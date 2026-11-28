//! #4081: the Laser Profile operation through the commands - the op on
//! a laser cutter only (the family door both ways), generation over a
//! holed plate (hole first, pierces in the scrap, every note in the
//! laser's words), the tool doors, undo / redo, the project round trip,
//! the laser machine check, and the post and sim refusals that hold
//! until the laser post exists.
//! Provenance: our own tests over our own fixtures; the sheet rules
//! they pin are cited in `crates/motion/src/laser/mod.rs`.

use flecs_ecs::prelude::World;

use camcraft_app::cl::MoveKind;
use camcraft_app::commands::{AddTool, CreateOperation, EditOperationParams, GenerateOperation, History, ImportModel, LoadMachine, SetStock};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machine::{self, Kind};
use camcraft_app::model;
use camcraft_app::operation::{Family, Height, JetLead, JetSide, JetTabsOp, KindInfo, LaserOp, OpParams, OpType, Section, SrcRef};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::world::new_world;
use camcraft_tools::laser::laser_tool;

const HEAD: &str = "Beam";

fn ring(h: f64, z: f64) -> [Point3; 4] {
    [Point3::new(-h, -h, z), Point3::new(h, -h, z), Point3::new(h, h, z), Point3::new(-h, h, z)]
}

fn quad(a: Point3, b: Point3, c: Point3, d: Point3) -> [Triangle; 2] {
    [Triangle::new(a, b, c), Triangle::new(a, c, d)]
}

fn annulus(outer: [Point3; 4], inner: [Point3; 4], up: bool, out: &mut Vec<Triangle>) {
    for i in 0..4 {
        let (o0, o1, i0, i1) = (outer[i], outer[(i + 1) % 4], inner[i], inner[(i + 1) % 4]);
        out.extend(if up { quad(o0, o1, i1, i0) } else { quad(o0, i0, i1, o1) });
    }
}

fn walls(r: f64, z0: f64, z1: f64, outward: bool, out: &mut Vec<Triangle>) {
    let (lo, hi) = (ring(r, z0), ring(r, z1));
    for i in 0..4 {
        let (a, b, c, d) = (lo[i], lo[(i + 1) % 4], hi[(i + 1) % 4], hi[i]);
        out.extend(if outward { quad(a, b, c, d) } else { quad(a, d, c, b) });
    }
}

/// A 40x40x3 plate with a 10x10 square hole through it.
fn holed_plate() -> Mesh {
    let mut f = Vec::new();
    annulus(ring(20.0, 3.0), ring(5.0, 3.0), true, &mut f);
    annulus(ring(20.0, 0.0), ring(5.0, 0.0), false, &mut f);
    walls(20.0, 0.0, 3.0, true, &mut f);
    walls(5.0, 0.0, 3.0, false, &mut f);
    Mesh::from_facets(f)
}

const LASER: &str = r#"{"general":{"model":"Laser test","kind":"laser"},"kinematics":{"default":{"parts":[{"name":"X","type":"linear","travel":{"min":-100,"max":360}},{"name":"Y","type":"linear","travel":{"min":-150,"max":150}},{"name":"Z","type":"linear","travel":{"min":-20,"max":60}},{"name":"laser head","type":"head"}]}},"controller":{"synced_configuration":{"parts":{"X":{"coordinate":0,"max_normal_speed":10000},"Y":{"coordinate":1,"max_normal_speed":10000},"Z":{"coordinate":2,"max_normal_speed":1000}}}}}"#;

fn laser_op() -> LaserOp {
    LaserOp {
        clearance_z: Height::fixed(10.0),
        top_z: Height::fixed(3.0),
        bottom_z: Height::fixed(0.0),
        kerf: None,
        control_comp: false,
        reversed: false,
        lead: JetLead::Arc,
        lead_in: 2.0,
        lead_out: 1.0,
        open_side: JetSide::Center,
        tabs: None,
        feed: 1200.0,
        power: 80.0,
        pierce_time: 0.4,
        linearize: None,
        picks: Default::default(),
    }
}

/// A laser document: the laser cutter mounted, the head, the plate.
fn laser_doc() -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let m = machine::parse_mch(LASER).expect("valid laser");
    h.execute(&world, LoadMachine::new(&world, "laser.mch".into(), Some(m)).expect("mounts"));
    h.execute(&world, AddTool::new(&world, HEAD, laser_tool(0.2)).expect("laser head"));
    h.execute(&world, ImportModel::from_mesh("plate", holed_plate()));
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 3.0 }, Location::default()).expect("stock"));
    (world, h)
}

fn cut(world: &World, h: &mut History, params: OpParams) {
    h.execute(world, CreateOperation::new(world, "Laser1", HEAD, SrcRef::Entity("plate".into()), params).expect("op"));
    h.execute(world, GenerateOperation::new(world, "Laser1").expect("generates"));
}

fn path(world: &World) -> camcraft_app::cl::Toolpath {
    let e = model::find_by_name(world, "Laser1").expect("op");
    model::toolpath_of(world, &e).expect("generated").path
}

#[test]
fn the_laser_family_lives_on_a_laser_cutter_only() {
    let info = OpType::LaserProfile.info();
    assert_eq!((info.label, info.section, info.family()), ("Laser Profile", Section::Laser, Family::Laser));
    assert!(KindInfo::ALL.iter().any(|k| k.ty == OpType::LaserProfile));
    assert!(info.accepts(camcraft_app::tools::ToolKind::Laser));
    for kind in [Kind::Mill, Kind::MillTurn, Kind::Lathe, Kind::Plasma, Kind::WireEdm, Kind::SinkerEdm, Kind::Waterjet] {
        assert!(Section::Laser.kinds_for(kind).is_empty(), "{kind:?} admits no laser work");
    }
    // A mill document refuses the op by name.
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, AddTool::new(&world, HEAD, laser_tool(0.2)).expect("laser head"));
    h.execute(&world, ImportModel::from_mesh("plate", holed_plate()));
    let err = CreateOperation::new(&world, "Laser1", HEAD, SrcRef::Entity("plate".into()), OpParams::LaserProfile(laser_op()))
        .err()
        .expect("a mill refuses laser work");
    assert!(format!("{err:?}").contains("laser cutting work") && format!("{err:?}").contains("mount a laser cutter first"), "{err:?}");
}

#[test]
fn a_holed_plate_cuts_its_hole_first_and_speaks_as_the_laser() {
    let (world, mut h) = laser_doc();
    cut(&world, &mut h, OpParams::LaserProfile(laser_op()));
    let p = path(&world);
    let ps: Vec<(f64, f64)> =
        p.moves.iter().filter(|m| m.kind == MoveKind::Rapid && m.to.z == 3.0 && m.from.z > 3.0).map(|m| (m.to.x, m.to.y)).collect();
    assert_eq!(ps.len(), 2, "{ps:?}");
    assert!(ps[0].0.abs() < 5.0 && ps[0].1.abs() < 5.0, "hole first: {ps:?}");
    assert!(ps[1].0.abs() > 20.0 || ps[1].1.abs() > 20.0, "outline last, outside: {ps:?}");
    assert!(p.notes.iter().any(|n| n == "laser: 2 contour(s) cut inner first (1 hole(s), 0 tab(s)), 2 pierce(s), kerf 0.200 mm offset in the computer"), "{:?}", p.notes);
    assert!(p.notes.iter().any(|n| n.contains("requested power 80.0 %")), "{:?}", p.notes);
    assert!(!p.notes.iter().any(|n| n.contains("waterjet") || n.contains("abrasive")), "{:?}", p.notes);
    // The laser machine passes its own moves: no family, tool, rotary,
    // feed or stroke problem.
    let e = model::find_by_name(&world, "Laser1").expect("op");
    let m = machine::parse_mch(LASER).expect("valid laser");
    let check = machine::check_ops(&world, &m, &[("Laser1".to_owned(), e)]).expect("the check runs");
    assert!(check.problems.is_empty(), "{:?}", check.problems);
}

#[test]
fn the_tool_doors_name_the_laser_head() {
    let (world, mut h) = laser_doc();
    let em = camcraft_app::tools::catalog().remove(0);
    h.execute(&world, AddTool::new(&world, em.0, em.1).expect("end mill"));
    assert!(
        CreateOperation::new(&world, "Laser1", em.0, SrcRef::Entity("plate".into()), OpParams::LaserProfile(laser_op())).is_err(),
        "a laser profile mounts a laser head only"
    );
    let jet = camcraft_app::operation::KindInfo::ALL.iter().find(|k| k.ty == OpType::WaterjetProfile).expect("waterjet row");
    assert!(!jet.accepts(camcraft_app::tools::ToolKind::Laser), "the waterjet never mounts a laser head");
}

#[test]
fn create_edit_undo_redo_and_the_power_door() {
    let (world, mut h) = laser_doc();
    let p0 = OpParams::LaserProfile(laser_op());
    h.execute(&world, CreateOperation::new(&world, "Laser1", HEAD, SrcRef::Entity("plate".into()), p0.clone()).expect("op"));
    let p1 = OpParams::LaserProfile(LaserOp { reversed: true, lead: JetLead::Straight, linearize: Some(0.01), power: 100.0, ..laser_op() });
    h.execute(&world, EditOperationParams::new(&world, "Laser1", p1.clone()).expect("edit"));
    let params = |w: &World| model::find_by_name(w, "Laser1").and_then(|e| model::operation_of(w, &e)).map(|o| o.params);
    assert_eq!(params(&world), Some(p1.clone()));
    assert!(h.undo(&world));
    assert_eq!(params(&world), Some(p0.clone()));
    assert!(h.redo(&world));
    assert_eq!(params(&world), Some(p1));
    h.execute(&world, GenerateOperation::new(&world, "Laser1").expect("generates"));
    let p = path(&world);
    assert!(!p.has_arcs(), "linearized");
    assert!(p.notes.iter().any(|n| n.starts_with("laser: cut direction REVERSED")), "{:?}", p.notes);
    for bad in [0.0, 101.0, f64::NAN] {
        let op = OpParams::LaserProfile(LaserOp { power: bad, ..laser_op() });
        assert!(EditOperationParams::new(&world, "Laser1", op).is_err(), "power {bad} is refused at the door");
    }
}

#[test]
fn the_op_round_trips_byte_identically() {
    let (world, mut h) = laser_doc();
    let tabs = JetTabsOp { count: 1, width: 1.5, at: vec![[20.0, 0.0]] };
    let op = LaserOp { kerf: Some(0.25), open_side: JetSide::Left, tabs: Some(tabs), power: 65.5, linearize: Some(0.02), ..laser_op() };
    cut(&world, &mut h, OpParams::LaserProfile(op.clone()));
    let file = std::env::temp_dir().join(format!("camcraft_laser_op_rt_{}.ccam", std::process::id()));
    camcraft_app::project::save(&world, &file).expect("save");
    let first = std::fs::read_to_string(&file).expect("read");
    let again = new_world();
    camcraft_app::project::load(&again, &file).expect("load");
    let e = model::find_by_name(&again, "Laser1").expect("op");
    assert_eq!(model::operation_of(&again, &e).map(|o| o.params), Some(OpParams::LaserProfile(op)));
    camcraft_app::project::save(&again, &file).expect("resave");
    let second = std::fs::read_to_string(&file).expect("reread");
    let _ = std::fs::remove_file(&file);
    assert!(first.contains("\"laser_profile\"") && first.contains("\"power\": 65.5"), "the words are written");
    assert_eq!(first, second, "byte-identical");
}

#[test]
fn the_post_and_the_stock_sim_refuse_laser_work_by_name() {
    let (world, mut h) = laser_doc();
    cut(&world, &mut h, OpParams::LaserProfile(laser_op()));
    let report = camcraft_app::posting::verification_report(&world);
    let refusal = report.problems.iter().find(|p| p.contains("Laser1") && p.contains("laser posting is unavailable"))
        .unwrap_or_else(|| panic!("the laser operation must be refused by name: {:?}", report.problems));
    for missing in ["beam on/off", "power records", "pierce dwell"] {
        assert!(refusal.contains(missing), "{refusal}");
    }
    let Err(sim) = camcraft_app::stock_sim::simulate_document(&world) else {
        panic!("the sim refuses laser removal");
    };
    assert!(sim.contains("Laser1: laser stock removal is not judged"), "{sim}");
}
