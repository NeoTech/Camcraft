//! Provenance: #4031's cutter hand - the persistence contract and the
//! refusal law, using our own starter catalog (the shape #1453's
//! `tap_hand.rs` set for the tap's thread hand).
//! Gate map: covers app::project (tool JSON), app::tool_gate, app::posting,
//! app::tool_library, app::commands (tool, operation), app::operation
//! (stamp_tool), tools::tools.

use camcraft_app::commands::{
    AddTool, ChangeTool, CreateOperation, GenerateOperation, History, ImportModel, ImportTools,
    LoadMachine, SelectPost, SetStock,
};
use camcraft_app::drilling::{CycleChoice, Hole};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::schema::{Location, PostRef};
use camcraft_app::stock::StockShape;
use camcraft_app::tool_library::{export_tool_library, parse_tool_library};
use camcraft_app::world::new_world;
use camcraft_app::{model, posting, project, tools};
use flecs_ecs::prelude::World;

mod thread_support;

/// The posts that start a spindle M3 and never reverse it (the refusal
/// targets, #4031), and one that reverses, for the allow side.
const M3_ONLY: &[&str] = &["Tormach PathPilot", "Doosan DNM", "Grbl 1.1", "grblHAL", "Makera Carvera", "Heidenhain", "Heidenhain Klartext"];

fn drill(hand: Option<&str>) -> tools::Tool {
    let t = tools::catalog().into_iter().find(|(n, _)| *n == "DR-0300").expect("catalog drill").1;
    let mut v = project::tool_to_json(&t);
    if let Some(hand) = hand {
        v["cutter_hand"] = serde_json::json!(hand);
    }
    project::tool_from_json(&v).expect("drill JSON")
}

fn tool(world: &World) -> tools::Tool {
    model::find_by_name(world, "Drill").and_then(|e| model::tool_of(world, &e)).expect("Drill tool")
}

fn assert_left(t: &tools::Tool) {
    assert_eq!(project::tool_to_json(t)["cutter_hand"], "left");
}

fn flat_plate() -> Mesh {
    // The already-drilled puck of rigid_tapping.rs / tap_hand.rs: a flat
    // plate with a pilot hole, so no unrelated reach refusal.
    let meridian = [(0.0, 0.0), (20.0, 0.0), (20.0, 10.0), (3.1, 10.0), (3.1, 2.0), (0.0, 2.0)];
    let v = |j: usize, r: f64, z: f64| {
        let th = std::f64::consts::TAU * j as f64 / 16.0;
        Point3::new(r * th.cos(), r * th.sin(), z)
    };
    let mut facets = Vec::new();
    for seg in meridian.windows(2) {
        let ((ra, za), (rb, zb)) = (seg[0], seg[1]);
        for j in 0..16 {
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

fn drill_params() -> OpParams {
    OpParams::Drill {
        break_through: None,
        break_through_from: Default::default(),
        clearance_z: Height::fixed(25.0), retract_z: None, top_z: Height::fixed(10.0),
        bottom_z: None, cycle: CycleChoice::Drill, peck: 0.0, dwell: 0.0, feed: 100.0,
        plunge_feed: 100.0,
        holes: vec![Hole {
            xy: Point2::new(0.0, 0.0), radius: 3.05, entry_z: 10.0, floor_z: 2.0,
            through: false, stepped: false,
        }],
        cross: Vec::new(),
        picks: Default::default(),
    }
}

/// A world with a drilled plate and the named-hand drill, posting with
/// the document's own post (the built-in Fanuc unless selected away).
fn job(hand: Option<&str>) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, LoadMachine::builtin(&world, "VMC 3-axis").expect("machine"));
    h.execute(&world, SetStock::new(&world, StockShape::Cylinder { diameter: 40.0, height: 10.0 }, Location::default()).expect("stock"));
    h.execute(&world, ImportModel::from_mesh("plate", flat_plate()));
    h.execute(&world, AddTool::new(&world, "Drill", drill(hand)).expect("add drill"));
    (world, h)
}

fn output(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camcraft-4031-{tag}-{}.nc", std::process::id()))
}

#[test]
fn left_hand_cutter_survives_tool_json_roundtrip_and_legacy_defaults_right() {
    let left = drill(Some("left"));
    assert_left(&left);
    assert_eq!(drill(None), drill(Some("right")), "legacy tools default to right-hand");
    let mut definition = project::tool_to_json(&drill(None));
    for bad in [serde_json::json!("backwards"), serde_json::json!(7), serde_json::Value::Null] {
        definition["cutter_hand"] = bad.clone();
        let err = project::tool_from_json(&definition).expect_err("unknown hand must refuse");
        assert!(err.contains("cutter hand") || err.contains("cutter_hand"), "{bad}: {err}");
    }
}

#[test]
fn creation_refuses_a_left_hand_cutter_by_name_on_every_m3_only_post() {
    for post in M3_ONLY {
        let (world, _h) = job(Some("left"));
        let mut h = History::default();
        h.execute(&world, SelectPost::new(&world, PostRef::Named((*post).into())).expect("select post"));
        let err = match CreateOperation::new(&world, "Drill1", "Drill", SrcRef::Entity("plate".into()), drill_params()) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("{post} accepted a left-hand cutter"),
        };
        assert!(err.contains("left-hand cutter") && err.contains(post), "{post}: {err}");
    }
}

#[test]
fn creation_refuses_a_left_hand_drill_even_on_a_reversing_post() {
    // #4031 rework: the check is on the REQUEST, not the post. A
    // permitted post reverses nothing on its own - a Drill never asks
    // for M4, so even the built-in Fanuc refuses the left-hand cutter.
    for post in ["Fanuc", "Mazak", "Haas"] {
        let (world, _h) = job(Some("left"));
        let mut h = History::default();
        h.execute(&world, SelectPost::new(&world, PostRef::Named(post.into())).expect("select post"));
        let err = match CreateOperation::new(&world, "Drill1", "Drill", SrcRef::Entity("plate".into()), drill_params()) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("{post} accepted a left-hand cutter on a Drill"),
        };
        assert!(err.contains("left-hand cutter") && err.contains("Drill"), "{post}: {err}");
    }
}

#[test]
fn a_thread_operation_that_asks_to_reverse_posts_m4_with_a_left_hand_cutter() {
    // The one permitted path (#4031 rework): a Thread operation whose
    // passes ask the spindle to reverse. The library tool's hand rides
    // the JSON, the op's ask rides the passes - the posted program
    // turns M4, never M3.
    use thread_support::{MULTI, R_M8, world_with};

    let mill = {
        let t = tools::catalog().into_iter().find(|(n, _)| *n == MULTI).expect("catalog thread mill").1;
        let mut v = project::tool_to_json(&t);
        v["cutter_hand"] = serde_json::json!("left");
        project::tool_from_json(&v).expect("thread mill JSON")
    };
    let (world, mut history) = world_with(R_M8, &[(MULTI, mill)]);
    let mut params = thread_support::thread(R_M8, None, camcraft_app::cl::ThreadHand::Right, true);
    if let OpParams::Thread { passes, .. } = &mut params {
        passes.spindle_reverse = true;
        passes.cutter_left = true;
    }
    let op = CreateOperation::new(&world, "Thread1", MULTI, SrcRef::Entity("disc".into()), params)
        .expect("a Thread op that asks to reverse takes the left-hand mill");
    history.execute(&world, op);
    history.execute(&world, GenerateOperation::new(&world, "Thread1").expect("generate the M4 thread"));
    let path = output("m4");
    let posted = posting::post_setup(&world, &path).expect("the left-hand thread posts");
    assert!(
        posted.program.lines().any(|l| l.split_whitespace().any(|w| w == "M4")),
        "the program must turn M4: {}",
        posted.program
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn every_kind_without_a_reverse_request_refuses_a_left_hand_cutter_by_name() {
    // The law is one sentence for every kind (#4031 rework): a
    // left-hand cutter is refused unless the operation asks to reverse.
    // The loop walks the whole catalog table - Drill, Face, Contour,
    // Pocket, Adaptive, Bore, Circular, every multiaxis and turning
    // kind - with a left-hand tool of an accepted family in the
    // library, and holds the refusal by name. The Thread row also
    // proves the open door: with the reverse ask the tool is accepted.
    use camcraft_app::operation::{KindInfo, tool_gate};

    let (world, _h) = job(Some("left"));
    // One left-hand copy of every catalog tool, so each kind's family
    // has a candidate in the library.
    let mut h = History::default();
    for (name, t) in tools::catalog() {
        let mut v = project::tool_to_json(&t);
        v["cutter_hand"] = serde_json::json!("left");
        if let Ok(left) = project::tool_from_json(&v) {
            h.execute(&world, AddTool::new(&world, name, left).expect("add catalog tool"));
        }
    }
    let library: Vec<(String, tools::Tool)> = model::named_entities(&world)
        .into_iter()
        .filter_map(|(name, e)| model::tool_of(&world, &e).map(|t| (name, t)))
        .filter(|(_, t)| t.kind != tools::ToolKind::FormMill && !camcraft_tools::taper::is_unread_tip(t))
        .collect();
    assert!(library.len() > 10, "the catalog must offer families: {library:?}");

    let mut covered = 0usize;
    for info in KindInfo::ALL {
        if !info.mounts_tool() {
            continue;
        }
        let Some((name, _)) = library.iter().find(|(_, t)| info.accepts(t.kind)) else {
            continue;
        };
        covered += 1;
        let err = tool_gate::tool_refusal(&world, info, name, false)
            .unwrap_or_else(|| panic!("{} accepted a left-hand cutter", info.label));
        assert!(
            err.contains("left-hand cutter") && err.contains(info.label),
            "{}: {err}",
            info.label
        );
        if info.ty == camcraft_app::operation::OpType::Thread {
            assert!(
                tool_gate::tool_refusal(&world, info, name, true).is_none(),
                "a Thread op that asks to reverse takes the left-hand mill"
            );
        }
    }
    assert!(covered > 30, "the loop must cover the catalog's kinds, got {covered}");
}

#[test]
fn an_edit_to_left_hand_refuses_generation_and_post_until_the_operation_asks() {
    // The after-state door (M10.1 C) reads the same REQUEST law: the
    // tool edited to left-hand strands every operation that does not
    // ask to reverse - generation and post refuse by name, and no NC
    // file is written, on the reversing Fanuc as much as on an M3-only post.
    let (world, mut h) = job(None);
    h.execute(&world, CreateOperation::new(&world, "Drill1", "Drill", SrcRef::Entity("plate".into()), drill_params()).expect("drill op"));
    h.execute(&world, GenerateOperation::new(&world, "Drill1").expect("generate right-hand drill"));
    let path = output("right");
    let posted = posting::post_setup(&world, &path).expect("right-hand drill posts");
    assert!(!posted.program.is_empty());
    let _ = std::fs::remove_file(&path);

    // Edit the tool to left-hand: the Drill never asks for M4, so
    // generation and post refuse by name - the Fanuc post's ability to
    // reverse changes nothing.
    h.execute(&world, ChangeTool::new(&world, "Drill", drill(Some("left"))).expect("change hand"));
    assert_left(&tool(&world));
    let gen_err = match GenerateOperation::new(&world, "Drill1") {
        Err(e) => e.to_string(),
        Ok(_) => panic!("generation accepted a left-hand cutter on a Drill that asks no reverse"),
    };
    assert!(gen_err.contains("left-hand cutter") && gen_err.contains("Drill"), "{gen_err}");
    let path = output("fanuc");
    let _ = std::fs::remove_file(&path);
    let post_err = posting::post_setup(&world, &path).expect_err("posting refuses");
    assert!(post_err.contains("left-hand cutter") && post_err.contains("Drill1"), "{post_err}");
    assert!(!path.exists(), "a refusal writes no NC file");

    // The M3-only posts keep their own sentence, Thread op or not.
    // #1977: Haas reverses now (M4), so Tormach PathPilot is the M3-only post here.
    h.execute(&world, SelectPost::new(&world, PostRef::Named("Tormach PathPilot".into())).expect("select Tormach"));
    let gen_err = match GenerateOperation::new(&world, "Drill1") {
        Err(e) => e.to_string(),
        Ok(_) => panic!("generation accepted a left-hand cutter on Tormach PathPilot"),
    };
    assert!(gen_err.contains("left-hand cutter") && gen_err.contains("Tormach PathPilot"), "{gen_err}");
    let path = output("tormach");
    let post_err = posting::post_setup(&world, &path).expect_err("posting refuses");
    assert!(post_err.contains("left-hand cutter") && post_err.contains("Drill1"), "{post_err}");
    assert!(!path.exists(), "a refusal writes no NC file");
}

#[test]
fn hand_survives_project_and_library_and_the_library_steps_to_version_7() {
    let (world, _h) = job(Some("left"));
    let saved = project::save_text(&world).expect("save");
    let value: serde_json::Value = serde_json::from_str(&saved).expect("project JSON");
    assert!(value["version"].as_u64().expect("version") >= 83, "old builds must refuse left-hand projects");
    let loaded = new_world();
    project::open(&loaded, &saved).expect("load");
    assert_left(&tool(&loaded));

    let library = export_tool_library(&world).expect("export library");
    let value: serde_json::Value = serde_json::from_str(&library).expect("library JSON");
    assert_eq!(value["version"], 7, "old library readers must refuse a left-hand cutter");
    let lib = parse_tool_library(&library).expect("read the v7 library");
    let imported = new_world();
    let mut ih = History::default();
    ih.execute(&imported, ImportTools::new(&imported, lib).expect("import"));
    assert_left(&tool(&imported));

    // A library without a left-hand cutter keeps its older number.
    let (right_world, _h) = job(None);
    let legacy = export_tool_library(&right_world).expect("right-hand library");
    let value: serde_json::Value = serde_json::from_str(&legacy).expect("library JSON");
    assert_eq!(value["version"], 1, "right-hand-only libraries keep their format");
    assert_eq!(parse_tool_library(&legacy).expect("legacy library").len(), 1);
}
