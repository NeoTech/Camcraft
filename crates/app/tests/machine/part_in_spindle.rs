//! Part-in-spindle mode on a mill (#3406): the mill spindle holds the
//! bar in a collet and stationary lathe tools sit on the table, each at
//! its own work offset. The setup states the mode (the stock's length
//! offset register `H`, the collet's stick-out and diameter) and its
//! table stations through undoable commands; the family door then
//! admits turning work and an axial Drill and refuses milling by name;
//! two tools never share one work offset; the turning op generates and
//! the lathe field eye cuts it; the collet's workholding rpm is required
//! and enforced at the post gate; the project file keeps it all.
//!
//! Gate map (M4.4): covers app::part_in_spindle, app::commands
//! (SetPartInSpindle, SetTableStation, the family door), app::posting
//! (the program gate), app::setup_workholding (the mill's collet),
//! app::stock_view (the lathe field eye), app::project (the additive key).

use flecs_ecs::prelude::World;

use camcraft_app::commands::{
    AddTool, CreateOperation, CreateSetup, GenerateOperation, History, ImportModel, LoadMachine, SetPartInSpindle,
    SetSetupWorkholdingRpm, SetStock, SetTableStation,
};
use camcraft_app::drilling::{CrossHole, CycleChoice, Hole};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machine::Machine;
use camcraft_app::operation::{Height, OpParams, OpStatus, SrcRef};
use camcraft_app::part_in_spindle::{self, ApproachSide, Holding, SpindleDir, TableStation};
use camcraft_app::schema::Location;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use camcraft_app::{model, posting, project};

const TAU: f64 = std::f64::consts::TAU;
const INSERT: &str = "TN-1600-0R8";
const DRILL: &str = "DR-0400";
const TORMACH_MILL: &str = include_str!("../../../../assets/posts/tormach.cps");

fn revolve(meridian: &[(f64, f64)], segs: usize, cx: f64) -> Mesh {
    let v = |j: usize, r: f64, z: f64| {
        let th = TAU * j as f64 / segs as f64;
        Point3::new(cx + r * th.cos(), r * th.sin(), z)
    };
    let mut facets = Vec::new();
    for seg in meridian.windows(2) {
        let ((ra, za), (rb, zb)) = (seg[0], seg[1]);
        for j in 0..segs {
            let (a1, a2, b1, b2) = (v(j, ra, za), v(j + 1, ra, za), v(j, rb, zb), v(j + 1, rb, zb));
            if ra <= 1e-12 {
                facets.push(Triangle::new(a1, b2, b1));
            } else if rb <= 1e-12 {
                facets.push(Triangle::new(a1, a2, b2));
            } else {
                facets.push(Triangle::new(a1, a2, b2));
                facets.push(Triangle::new(a1, b2, b1));
            }
        }
    }
    Mesh::from_facets(facets)
}

fn holding() -> Holding {
    Holding { h_register: 255, stick_out: 40.0, collet_od: 30.0 }
}

fn station(tool: &str, work_offset: usize) -> TableStation {
    TableStation { tool: tool.into(), work_offset, side: ApproachSide::MinusX, spindle: SpindleDir::M3 }
}

/// A 12 mm bar in its own-size stock on a 10000 rpm MILL, declared as
/// Setup1, with a carbide insert and a 4 mm drill.
fn mill_bar() -> (World, History) {
    mill_bar_at(0.0)
}

/// [`mill_bar`] with the bar's axis at X `cx`; the work offset stays at the origin.
fn mill_bar_at(cx: f64) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let at = Location { pos: Point3::new(cx, 0.0, -16.0), ..Location::default() };
    let stock = SetStock::new(&world, camcraft_app::stock::StockShape::Cylinder { diameter: 12.0, height: 32.0 }, at);
    h.execute(&world, stock.expect("bar stock"));
    let mill = Machine {
        model: "TestBench mill".into(),
        vendor: "Camcraft Testbench".into(),
        kind: Some("mill".into()),
        spindle_min: Some(100.0),
        spindle_max: Some(10000.0),
        ..Default::default()
    };
    h.execute(&world, LoadMachine::new(&world, "mill.mch".into(), Some(mill)).expect("mill"));
    for name in [INSERT, DRILL] {
        let tool = catalog().into_iter().find(|(n, _)| *n == name).map(|(_, t)| t).expect("catalog entry");
        h.execute(&world, AddTool::new(&world, name, tool).expect("catalog tool"));
    }
    let bar = revolve(&[(0.0, -16.0), (6.0, -16.0), (6.0, 2.0), (5.0, 2.0), (5.0, 16.0), (0.0, 16.0)], 16, cx);
    h.execute(&world, ImportModel::from_mesh("bar", bar));
    h.execute(&world, CreateSetup::new(&world, None, None).expect("Setup1"));
    (world, h)
}

fn facing() -> OpParams {
    OpParams::Facing {
        tolerance: 0.05,
        clearance_z: Height::fixed(25.0),
        top_z: Height::fixed(16.0),
        bottom_z: Height::fixed(14.0),
        stepdown: 0.5,
        feed: 150.0,
        plunge_feed: 80.0,
        picks: Default::default(),
    }
}

fn drill(cross: Vec<CrossHole>) -> OpParams {
    drill_at(0.0, cross)
}

fn drill_at(x: f64, cross: Vec<CrossHole>) -> OpParams {
    let hole = Hole { xy: Point2::new(x, 0.0), radius: 2.0, entry_z: 16.0, floor_z: 6.0, through: false, stepped: false };
    OpParams::Drill {
        break_through: None,
        break_through_from: Default::default(),
        clearance_z: Height::fixed(30.0),
        retract_z: None,
        top_z: Height::fixed(16.0),
        bottom_z: None,
        cycle: CycleChoice::Auto,
        peck: 0.0,
        dwell: 0.0,
        feed: 120.0,
        plunge_feed: 80.0,
        holes: if cross.is_empty() { vec![hole] } else { vec![] },
        cross,
        picks: Default::default(),
    }
}

fn create(world: &World, name: &str, tool: &str, params: OpParams) -> Result<CreateOperation, String> {
    CreateOperation::new(world, name, tool, SrcRef::Entity("bar".into()), params).map_err(|e| e.to_string())
}

fn gate(world: &World) -> Vec<String> {
    posting::verification_gate(world)
}

#[test]
fn turning_is_admitted_only_in_part_in_spindle_mode_and_generates_and_cuts_on_the_field() {
    let (world, mut h) = mill_bar();
    let why = create(&world, "F1", INSERT, facing()).err().unwrap_or_default();
    assert!(why.contains("turning"), "a plain mill refuses turning work: {why}");

    h.execute(&world, SetPartInSpindle::new(&world, "Setup1", Some(holding())).expect("mode on"));
    h.execute(&world, create(&world, "F1", INSERT, facing()).expect("turning is admitted"));
    h.execute(&world, GenerateOperation::new(&world, "F1").expect("generate"));
    let e = model::find_by_name(&world, "F1").expect("op exists");
    assert_eq!(camcraft_app::operation::status_of(&world, &e), OpStatus::Ok, "F1 generated");
    // The lathe field eye cuts the faced end: z 14..16 is air, below it steel.
    let view = camcraft_app::stock_view::field_view_with(&world, 512_000.0).expect("the field eye cuts the turning program");
    assert!(view.stock.distance(Point3::new(2.0, 0.0, 15.0)) > 0.0, "the faced end is cut");
    assert!(view.stock.distance(Point3::new(2.0, 0.0, 10.0)) < 0.0, "below the face is steel");

    // An axial Drill is the spindle's own hole; a cross hole needs live
    // tooling a mill table has not.
    assert!(create(&world, "D1", DRILL, drill(vec![])).is_ok(), "an on-centre drill is admitted");
    // The part turns: a hole off the spindle axis is refused by name at
    // generation (the lathe's axial door), never drilled as a mill hole.
    h.execute(&world, create(&world, "D3", DRILL, drill_at(3.0, vec![])).expect("the door admits a Drill"));
    let why = GenerateOperation::new(&world, "D3").err().map(|e| e.to_string()).unwrap_or_default();
    assert!(why.contains("D3") && why.contains("off the spindle axis"),"an off-centre hole in a turning part is refused by name: {why}");
    let cross = CrossHole {
        origin: Point3::new(6.0, 0.0, 0.0),
        dir: Point3::new(-1.0, 0.0, 0.0),
        radius: 2.0,
        entry_d: 0.0,
        floor_d: -3.0,
        through: false,
    };
    let why = create(&world, "D2", DRILL, drill(vec![cross])).err().unwrap_or_default();
    assert!(why.contains("part-in-spindle") && why.contains("Setup1"), "a cross drill is refused by name: {why}");

    // The mode cannot be cleared under its turning op.
    let why = SetPartInSpindle::new(&world, "Setup1", None).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(why.contains("F1"), "clearing names the turning op: {why}");
}

#[test]
fn an_axial_drill_off_the_spindle_line_is_refused_by_the_spindle_law() {
    // The bar stands 2 mm off the work offset: the spindle turns about
    // world Z through the origin, so a hole on the bar's own axis would
    // be drilled about a line the part is not on.
    let (world, mut h) = mill_bar_at(2.0);
    h.execute(&world, SetPartInSpindle::new(&world, "Setup1", Some(holding())).expect("mode on"));
    h.execute(&world, create(&world, "D1", DRILL, drill_at(2.0, vec![])).expect("the door admits a Drill"));
    let why = GenerateOperation::new(&world, "D1").err().map(|e| e.to_string()).unwrap_or_default();
    assert!(why.contains("D1") && why.contains("not the lathe spindle"), "refused by the spindle law: {why}");
}

#[test]
fn the_mode_and_stations_are_validated_undoable_commands() {
    let (world, mut h) = mill_bar();
    assert!(SetTableStation::put(&world, "Setup1", station(INSERT, 2)).is_err(), "no mode, no station");
    for bad in [
        Holding { h_register: 0, ..holding() },
        Holding { stick_out: 0.0, ..holding() },
        Holding { stick_out: f64::NAN, ..holding() },
        Holding { collet_od: -1.0, ..holding() },
    ] {
        assert!(SetPartInSpindle::new(&world, "Setup1", Some(bad)).is_err(), "{bad:?} is refused");
    }
    assert!(SetPartInSpindle::new(&world, "Setup1", None).is_err(), "clearing nothing is a no-op");
    assert!(SetPartInSpindle::new(&world, "NoSuch", Some(holding())).is_err());
    h.execute(&world, SetPartInSpindle::new(&world, "Setup1", Some(holding())).expect("mode on"));
    assert!(SetPartInSpindle::new(&world, "Setup1", Some(holding())).is_err(), "already stated");

    h.execute(&world, SetTableStation::put(&world, "Setup1", station(INSERT, 2)).expect("insert at offset 2"));
    let why = SetTableStation::put(&world, "Setup1", station(DRILL, 2)).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(why.contains("work offset 2") && why.contains(INSERT), "a shared offset is refused by name: {why}");
    assert!(SetTableStation::put(&world, "Setup1", station(DRILL, 0)).is_err(), "offsets are 1-based");
    assert!(SetTableStation::put(&world, "Setup1", station("NoTool", 3)).is_err(), "a tool must exist");
    h.execute(&world, SetTableStation::put(&world, "Setup1", station(DRILL, 3)).expect("drill at offset 3"));
    let both = part_in_spindle::of(&world, Some("Setup1")).expect("the record");
    assert_eq!(both.stations.len(), 2);
    assert_eq!(both.holding, holding());

    // Moving the drill to the insert's offset is refused; moving it to a free one replaces it.
    let moved = TableStation { side: ApproachSide::PlusX, spindle: SpindleDir::M4, ..station(DRILL, 4) };
    h.execute(&world, SetTableStation::put(&world, "Setup1", moved.clone()).expect("move the drill"));
    let now = part_in_spindle::of(&world, Some("Setup1")).expect("the record");
    assert_eq!(now.stations.iter().find(|s| s.tool == DRILL), Some(&moved));
    // Changing the holding keeps the stations.
    let wider = Holding { collet_od: 32.0, ..holding() };
    h.execute(&world, SetPartInSpindle::new(&world, "Setup1", Some(wider)).expect("a wider collet"));
    assert_eq!(part_in_spindle::of(&world, Some("Setup1")).map(|p| p.stations.len()), Some(2));
    h.execute(&world, SetTableStation::remove(&world, "Setup1", DRILL).expect("remove the drill"));
    assert!(SetTableStation::remove(&world, "Setup1", DRILL).is_err(), "no station to remove");

    for _ in 0..3 {
        assert!(h.undo(&world));
    }
    assert_eq!(part_in_spindle::of(&world, Some("Setup1")), Some(both), "undo restores the record exactly");
    for _ in 0..3 {
        assert!(h.undo(&world));
    }
    assert_eq!(part_in_spindle::of(&world, Some("Setup1")), None, "undo clears the mode");
    assert!(h.redo(&world));
    assert_eq!(part_in_spindle::of(&world, Some("Setup1")).map(|p| p.stations.len()), Some(0));
}

#[test]
fn a_lathe_setup_refuses_the_mode() {
    let world = new_world();
    let mut h = History::default();
    let lathe = Machine { model: "TestBench lathe".into(), kind: Some("lathe".into()), ..Default::default() };
    h.execute(&world, LoadMachine::new(&world, "lathe.mch".into(), Some(lathe)).expect("lathe"));
    h.execute(&world, CreateSetup::new(&world, None, None).expect("Setup1"));
    let why = SetPartInSpindle::new(&world, "Setup1", Some(holding())).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(why.contains("mill") && why.contains("lathe"), "{why}");
}

#[test]
fn the_collets_rpm_is_required_and_enforced_and_the_post_is_refused_by_name() {
    let (world, mut h) = mill_bar();
    h.execute(&world, SetPartInSpindle::new(&world, "Setup1", Some(holding())).expect("mode on"));
    // A G97 turning op: 3000 rpm, no constant surface speed.
    let mut slow = catalog().into_iter().find(|(n, _)| *n == INSERT).map(|(_, t)| t).expect("insert");
    slow.rpm = 3000.0;
    slow.turning = None;
    h.execute(&world, AddTool::new(&world, "T97", slow).expect("G97 insert"));
    h.execute(&world, create(&world, "F1", "T97", facing()).expect("turning is admitted"));
    h.execute(&world, GenerateOperation::new(&world, "F1").expect("generate"));
    let unset = gate(&world);
    assert!(
        unset.iter().any(|l| l.contains("Setup1") && l.contains("workholding maximum rpm")),
        "an unset collet rating blocks the post: {unset:?}"
    );
    let out = std::env::temp_dir().join(format!("camcraft-3406-{}.nc", std::process::id()));
    let why = posting::post_script(&world, &out, TORMACH_MILL).err().unwrap_or_default();
    let _ = std::fs::remove_file(&out);
    assert!(why.contains("post blocked") && why.contains("workholding maximum rpm"), "{why}");

    // Over the collet's rating, the G97 op is refused, as on a lathe.
    h.execute(&world, SetSetupWorkholdingRpm::new(&world, "Setup1", Some(2000.0)).expect("collet rating"));
    let lines = gate(&world);
    assert!(!lines.iter().any(|l| l.contains("workholding maximum rpm")), "the rating is stated: {lines:?}");
    assert!(
        lines.iter().any(|l| l.contains("F1: turns the work at 3000 rpm (G97)") && l.contains("2000 rpm")),
        "the collet's rating is enforced on the mill: {lines:?}"
    );
    // No post section voices the table stations yet: refused by name, never milled.
    assert!(lines.iter().any(|l| l.contains("Setup1") && l.contains("part-in-spindle")), "{lines:?}");
}

#[test]
fn the_mode_and_stations_round_trip_in_the_project_file() {
    let (world, mut h) = mill_bar();
    h.execute(&world, SetPartInSpindle::new(&world, "Setup1", Some(holding())).expect("mode on"));
    h.execute(&world, SetTableStation::put(&world, "Setup1", station(INSERT, 2)).expect("station"));
    let path = std::env::temp_dir().join(format!("camcraft-3406-{}.camproj", std::process::id()));
    project::save(&world, &path).expect("save");
    let text = std::fs::read_to_string(&path).expect("read");
    assert_eq!(text.matches("\"part_in_spindle\"").count(), 1, "{text}");
    let back = new_world();
    project::open(&back, &text).expect("load");
    assert_eq!(part_in_spindle::of(&back, Some("Setup1")), part_in_spindle::of(&world, Some("Setup1")));
    project::save(&back, &path).expect("resave");
    let again = std::fs::read_to_string(&path).expect("reread");
    let _ = std::fs::remove_file(&path);
    assert_eq!(again, text, "byte-identical round trip");
    // A record no command would accept is a corrupt file, refused by name.
    let bad = text.replace("\"h_register\": 255", "\"h_register\": 0");
    assert_ne!(bad, text, "fixture: the key is in the file");
    let why = project::open(&new_world(), &bad).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(why.contains("Setup1") && why.contains("register"), "refused by name: {why}");
    // A document without the mode writes no key.
    let (plain, _h) = mill_bar();
    project::save(&plain, &path).expect("save plain");
    let plain_text = std::fs::read_to_string(&path).expect("read plain");
    let _ = std::fs::remove_file(&path);
    assert!(!plain_text.contains("part_in_spindle"));
}
