//! #650: lathe tool orientation numbers are read in the mounted
//! machine's turret frame. A rear-turret lathe (Tormach PathPilot's
//! frame) numbers the chart mirrored in X - its standard OD tool is 3,
//! its OD thread tip 8 - while the physical control point is the same.
//! So on a rear machine an OD tool numbered 3 posts exactly the path a
//! front machine's 2 posts, a facing with 3 reaches centre, refusals
//! and the tool note speak the machine's numbers, a front machine posts
//! byte for byte as before, and the side round-trips with the project.
//!
//! The PathPilot data points (behaviour reference only, nothing copied):
//! facing with orientation 3, an OD thread with 8, and a chamfer refusal
//! listing [1, 5, 6] / [2, 6, 7] - pinned below in the rear frame.
//!
//! Regenerate the rear golden after an intentional change:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-app --test lathe_turret_side`
//!
//! Gate map (M4.4): covers app::posting (the nose module's frame),
//! app::setup_mount (turret_of_setup), app::turret, app::orientation,
//! post::machine (general.turret), app::machine::validate.
//! Lane shape: seconds.

mod common;

use flecs_ecs::prelude::World;

use camcraft_app::commands::{
    AddTool, ChangeTool, CreateOperation, GenerateOperation, History, ImportModel, LoadMachine, SetStock,
};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machine::{parse_mch, validate, Coolant, Machine};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::orientation::Contact;
use camcraft_app::post::{cnc, engine};
use camcraft_app::posting;
use camcraft_app::project;
use camcraft_app::schema::Location;
use camcraft_app::tools::catalog;
use camcraft_app::turret::Turret;
use camcraft_app::world::new_world;

const TAU: f64 = std::f64::consts::TAU;
const LATHE_CPS: &str = include_str!("../../../assets/posts/fanuc-lathe.cps");
const TN: &str = "TN-1600-0R8";
const R: f64 = 0.8;
/// The lathe_post facing bar: wall r6 to z 2, neck r5 to the top.
const BAR: [(f64, f64); 6] = [(0.0, -16.0), (6.0, -16.0), (6.0, 2.0), (5.0, 2.0), (5.0, 16.0), (0.0, 16.0)];

fn revolve(meridian: &[(f64, f64)], segs: usize) -> Mesh {
    let v = |j: usize, r: f64, z: f64| {
        let th = TAU * j as f64 / segs as f64;
        Point3::new(r * th.cos(), r * th.sin(), z)
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

/// The lathe_post test bench, on the stated turret side (None = the
/// key unstated, every file before #650).
fn lathe(turret: Option<&str>) -> Machine {
    Machine {
        model: "TestBench lathe".into(),
        vendor: "Camcraft Testbench".into(),
        kind: Some("lathe".into()),
        spindle_min: Some(25.0),
        spindle_max: Some(12000.0),
        coolants: vec![Coolant { name: "Flood".into(), m_code: "M8".into() }],
        turret: turret.map(str::to_owned),
        ..Default::default()
    }
}

fn bar_world(turret: Option<&str>) -> (World, History) {
    let world = new_world();
    let mut history = History::default();
    let at = Location { pos: Point3::new(0.0, 0.0, -16.0), ..Location::default() };
    let cmd = SetStock::new(&world, camcraft_app::stock::StockShape::Cylinder { diameter: 12.0, height: 32.0 }, at);
    history.execute(&world, cmd.expect("bar stock"));
    history.execute(&world, LoadMachine::new(&world, "lathe.mch".into(), Some(lathe(turret))).expect("lathe"));
    let insert = catalog().into_iter().find(|(n, _)| *n == TN).map(|(_, t)| t).expect("catalog insert");
    history.execute(&world, AddTool::new(&world, TN, insert).expect("insert"));
    history.execute(&world, ImportModel::from_mesh("bar", revolve(&BAR, 16)));
    (world, history)
}

fn create(world: &World, history: &mut History, name: &str, params: OpParams) {
    let cmd = CreateOperation::new(world, name, TN, SrcRef::Entity("bar".into()), params).expect("create");
    history.execute(world, cmd);
    history.execute(world, GenerateOperation::new(world, name).expect("generate"));
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
    }
}

fn finish() -> OpParams {
    OpParams::ProfileFinish {
        tolerance: 0.05,
        clearance_z: Height::fixed(25.0),
        top_z: Height::fixed(16.0),
        bottom_z: Height::fixed(-2.0),
        feed: 120.0,
        plunge_feed: 60.0,
    }
}

/// Type `shown` into the tool the way the editor does: read in the
/// active machine's frame, stored canonical, through `ChangeTool`.
fn type_orientation(world: &World, history: &mut History, shown: u8) {
    let e = model::find_by_name(world, TN).expect("tool");
    let mut t = model::tool_of(world, &e).expect("a tool");
    t.orientation = Some(camcraft_app::setup_mount::turret_of_active(world).canonical(shown));
    history.execute(world, ChangeTool::new(world, TN, t).expect("a sane orientation"));
}

fn posted_moves(world: &World) -> Vec<[f64; 3]> {
    posting::stream_records(world)
        .expect("posts")
        .into_iter()
        .filter_map(|r| match r {
            cnc::Record::Move(m) => Some([m.x, m.y, m.z]),
            _ => None,
        })
        .collect()
}

fn program(world: &World) -> String {
    let records = posting::stream_records(world).expect("posts");
    engine::run(LATHE_CPS, &posting::machine_value(world), &cnc::serialize(&records))
        .expect("the lathe script runs")
        .program
}

#[test]
fn a_rear_od_tool_numbered_3_posts_the_front_2_control_point() {
    let (front, mut fh) = bar_world(None);
    create(&front, &mut fh, "S1", finish());
    type_orientation(&front, &mut fh, 2);
    let (rear, mut rh) = bar_world(Some("rear"));
    create(&rear, &mut rh, "S1", finish());
    let unstated = posted_moves(&rear);
    type_orientation(&rear, &mut rh, 3);
    assert_eq!(posted_moves(&rear), posted_moves(&front), "rear 3 is front 2: the same control point");
    assert_eq!(unstated, posted_moves(&front), "the rear default is the same OD tool");
    assert!(program(&rear).contains("ORIENTATION 3)"), "the rear tool note names 3: {}", program(&rear));
    assert!(program(&front).contains("ORIENTATION 2)"), "the front note still names 2");
    // The tool stores the canonical code: the same tool on either side.
    let e = model::find_by_name(&rear, TN).expect("tool");
    assert_eq!(model::tool_of(&rear, &e).and_then(|t| t.orientation), Some(2));
}

#[test]
fn a_rear_facing_with_3_reaches_centre_and_pins_the_rear_golden() {
    let (world, mut history) = bar_world(Some("rear"));
    create(&world, &mut history, "F1", facing());
    type_orientation(&world, &mut history, 3);
    let text = program(&world);
    assert!(text.contains("X-1.6 Z14 "), "the facing reaches centre like front 2's: {text}");
    assert!(text.contains("ORIENTATION 3"), "{text}");
    // Byte for byte the front golden, but for the number the note gives.
    let front = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/nc/fanuc-lathe/facing.nc"),
    )
    .expect("the front facing golden")
    .replace("\r\n", "\n");
    assert_eq!(text.replace("ORIENTATION 3)", "ORIENTATION 2)"), front, "only the tool note's number differs");
    common::write_golden("nc/fanuc-lathe/facing_rear.nc", &text);
}

#[test]
fn a_rear_od_thread_tip_is_8_and_a_rear_6_points_away_from_the_axis() {
    let (world, mut history) = bar_world(Some("rear"));
    create(&world, &mut history, "S1", finish());
    let e = model::find_by_name(&world, "S1").expect("op");
    let cl: Vec<Point3> = model::toolpath_of(&world, &e).expect("cache").path.moves.iter().map(|m| m.to).collect();
    // 8 on a rear turret: the neutral OD tip pointing at the axis (-X, 0).
    type_orientation(&world, &mut history, 8);
    let posted = posted_moves(&world);
    assert_eq!(posted.len(), cl.len());
    for (p, c) in posted.iter().zip(&cl) {
        assert!((p[1] - (c.y - R)).abs() < 1e-3 && (p[2] - c.z).abs() < 1e-3, "rear 8 is (-r, 0): {p:?} for {c:?}");
    }
    assert!(program(&world).contains("ORIENTATION 8)"));
    // 6 on a rear turret points +X: an ID tip, refused in rear numbers.
    type_orientation(&world, &mut history, 6);
    let e = posting::stream_records(&world).expect_err("an ID tip on an OD turn");
    assert!(e.contains("orientation [3, 4, 5, 7, 8] (rear turret numbering)") && e.contains("is orientation 6"), "{e}");
}

#[test]
fn refusals_name_the_numbers_of_the_machines_frame() {
    // A boring-bar code on a facing: rear 2 lists the rear [3, 7, 8].
    let (world, mut history) = bar_world(Some("rear"));
    create(&world, &mut history, "F1", facing());
    type_orientation(&world, &mut history, 2);
    let e = posting::stream_records(&world).expect_err("an ID tip on a face");
    assert!(e.contains("orientation [3, 7, 8] (rear turret numbering)") && e.contains("is orientation 2"), "{e}");
    // The same physical tool on a front machine: front 3, front list.
    let (world, mut history) = bar_world(None);
    create(&world, &mut history, "F1", facing());
    type_orientation(&world, &mut history, 3);
    let e = posting::stream_records(&world).expect_err("an ID tip on a face");
    assert!(e.contains("orientation [2, 6, 7] (front turret numbering)") && e.contains("is orientation 3"), "{e}");
}

#[test]
fn the_pathpilot_chamfer_listing_is_the_rear_bore_mouth_pair() {
    // PathPilot (rear) refused its OD tool 3 on a chamfer, listing
    // [1, 5, 6] or [2, 6, 7]. In the rear frame those are the cuts
    // needing the +X contact with +Z or -Z: an ID (bore-mouth) chamfer
    // - an OD tool cannot present +X. The OD chamfer an OD tool cuts
    // (-X) lists the rear OD family, which holds 3.
    assert_eq!(Turret::Rear.fitting(&[Contact::PlusX, Contact::PlusZ]), vec![1, 5, 6]);
    assert_eq!(Turret::Rear.fitting(&[Contact::PlusX, Contact::MinusZ]), vec![2, 6, 7]);
    assert!(!Turret::Rear.fitting(&[Contact::PlusX]).contains(&3), "rear 3 is no ID tool");
    let chamfer = OpParams::Chamfering {
        tolerance: 0.05,
        clearance_z: Height::fixed(25.0),
        top_z: Height::fixed(16.0),
        bottom_z: Height::fixed(14.0),
        angle_deg: 45.0,
        feed: 120.0,
        plunge_feed: 60.0,
    };
    let mut od = catalog().into_iter().find(|(n, _)| *n == TN).map(|(_, t)| t).expect("insert");
    od.orientation = Some(Turret::Rear.canonical(3));
    assert_eq!(posting::nose::checked_orientation(&chamfer, TN, &od, Turret::Rear), Ok(Some(2)), "an OD chamfer takes rear 3");
}

#[test]
fn the_mch_states_its_side_and_a_misspelt_one_is_refused_by_name() {
    let text = include_str!("../../../assets/machines/lathe.mch");
    let m = parse_mch(text).expect("the built-in lathe");
    assert_eq!(camcraft_app::setup_mount::turret_of_machine(Some(&m)), Turret::Front, "the built-in Lathe is front");
    let rear = text.replace("\"turret\": \"front\"", "\"turret\": \"rear\"");
    let m = parse_mch(&rear).expect("a rear lathe");
    assert_eq!(camcraft_app::setup_mount::turret_of_machine(Some(&m)), Turret::Rear);
    let bad = text.replace("\"turret\": \"front\"", "\"turret\": \"left\"");
    assert_ne!(bad, text, "the built-in lathe states its side");
    let e = parse_mch(&bad).expect_err("left is no side");
    assert!(e.contains("turret") && e.contains("left"), "{e}");
    assert!(validate::validate_text(&bad).iter().any(|r| r.key == "general.turret"), "the editor names the field");
    // Unstated is front: every machine file before #650.
    let old = text.replace(", \"turret\": \"front\"", "");
    assert_ne!(old, text);
    let m = parse_mch(&old).expect("an older lathe");
    assert_eq!(m.turret, None);
    assert_eq!(camcraft_app::setup_mount::turret_of_machine(Some(&m)), Turret::Front);
    assert_eq!(camcraft_app::setup_mount::turret_of_machine(None), Turret::Front);
}

#[test]
fn a_rear_lathe_and_its_tools_round_trip() {
    let path = std::env::temp_dir().join(format!("camcraft_proj_turret_{}.camproj", std::process::id()));
    let (world, mut history) = bar_world(Some("rear"));
    type_orientation(&world, &mut history, 3);
    project::save(&world, &path).expect("save");
    let text = std::fs::read_to_string(&path).expect("read");
    assert!(text.contains("\"turret\": \"rear\""), "the side is saved");
    assert!(text.contains("\"orientation\": 2"), "the tool keeps its canonical code");
    let back = new_world();
    project::load(&back, &path).expect("load");
    assert_eq!(camcraft_app::setup_mount::turret_of_active(&back), Turret::Rear);
    project::save(&back, &path).expect("resave");
    let again = std::fs::read_to_string(&path).expect("reread");
    let _ = std::fs::remove_file(&path);
    assert_eq!(again, text, "byte-identical round trip");
}
