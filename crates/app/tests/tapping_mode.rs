//! Issue #51: tapping intent must survive the command, generated cycle,
//! project cache and intermediate stream. `Tap` remains the rigid legacy
//! cycle; `FloatingTap` is explicit and keeps its own dwell.
//!
//! Gate map: covers app::commands (EditOperationParams + History),
//! app::operation (drill generation), app::project (params + cached cycle),
//! app::posting / post::cnc (dump + parse).
//!
//! Provenance: regression contract from issue #51 and our own toolpath and
//! stream laws; no vendor material consulted.

use flecs_ecs::prelude::World;

use camcraft_app::cl::{CycleKind, MoveKind, Toolpath};
use camcraft_app::commands::{
    AddTool, CreateOperation, EditOperationParams, GenerateOperation, History, ImportModel,
};
use camcraft_app::drilling::{CycleChoice, Hole};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle, cube};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::post::cnc::{self, Record};
use camcraft_app::posting;
use camcraft_app::project;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

const TOP: f64 = 20.0;
const FLOOR: f64 = -5.0;
const OP: &str = "Tap1";
const TOOL: &str = "TAP-M4";

fn flip(t: &Triangle) -> Triangle {
    Triangle::new(t.a, t.c, t.b)
}

/// A 40 x 40 block with a 6 mm square, 25 mm deep existing hole.
/// This is the established drill-reach fixture shape, kept local to the
/// tapping regression rather than expanding shared test fixtures.
fn block_with_hole(h: f64) -> Mesh {
    let quad = |a: Point3, b: Point3, c: Point3, d: Point3, f: &mut Vec<Triangle>| {
        f.push(Triangle::new(a, b, c));
        f.push(Triangle::new(a, c, d));
    };
    let mut f = Vec::new();
    let outer = cube(Point3::new(-20.0, -20.0, -10.0), Point3::new(20.0, 20.0, TOP));
    f.extend(outer.facets[4..12].iter().cloned());
    f.extend(outer.facets[0..2].iter().cloned());
    let o = [(-20.0, -20.0), (20.0, -20.0), (20.0, 20.0), (-20.0, 20.0)];
    let i = [(-h, -h), (h, -h), (h, h), (-h, h)];
    let p = |(x, y): (f64, f64)| Point3::new(x, y, TOP);
    for k in 0..4 {
        let n = (k + 1) % 4;
        quad(p(o[k]), p(o[n]), p(i[n]), p(i[k]), &mut f);
    }
    let inner = cube(Point3::new(-h, -h, FLOOR), Point3::new(h, h, TOP));
    f.extend(inner.facets[4..12].iter().map(flip));
    f.extend(inner.facets[0..2].iter().map(flip));
    Mesh::from_facets(f)
}

fn fresh_tap_world() -> (World, History) {
    let world = new_world();
    let mut history = History::default();
    let tap = catalog()
        .into_iter()
        .find(|(name, _)| *name == TOOL)
        .map(|(_, tool)| tool)
        .expect("catalog TAP-M4");
    history.execute(&world, AddTool::new(&world, TOOL, tap).expect("tap tool"));
    history.execute(&world, ImportModel::from_mesh("block", block_with_hole(3.0)));
    (world, history)
}

fn params(cycle: CycleChoice, dwell: f64) -> OpParams {
    OpParams::Drill {
        clearance_z: Height::fixed(TOP + 10.0),
        retract_z: None,
        top_z: Height::fixed(TOP),
        bottom_z: None,
        cycle,
        peck: 0.0,
        dwell,
        feed: 120.0,
        plunge_feed: 80.0,
        holes: vec![Hole {
            xy: Point2::new(0.0, 0.0),
            radius: 3.0,
            entry_z: TOP,
            floor_z: FLOOR,
            through: false,
            stepped: false,
        }],
        cross: vec![],
    }
}

fn create_and_generate(world: &World, history: &mut History, cycle: CycleChoice, dwell: f64) {
    history.execute(
        world,
        CreateOperation::new(world, OP, TOOL, SrcRef::Entity("block".into()), params(cycle, dwell))
            .expect("create tap operation"),
    );
    history.execute(world, GenerateOperation::new(world, OP).expect("generate tap operation"));
}

fn operation(world: &World) -> camcraft_app::schema::Operation {
    let entity = model::find_by_name(world, OP).expect("tap operation entity");
    model::operation_of(world, &entity).expect("operation definition")
}

fn path(world: &World) -> Toolpath {
    let entity = model::find_by_name(world, OP).expect("tap operation entity");
    model::toolpath_of(world, &entity).expect("generated toolpath")
}

fn assert_generated(world: &World, want: CycleKind, dwell: f64) -> Toolpath {
    let path = path(world);
    assert_eq!(path.cycles.len(), 1, "one record for the tapped hole");
    let cycle = path.cycles[0];
    assert_eq!(cycle.kind, want, "assert intent before dumping the path");
    assert_eq!(cycle.dwell, dwell, "cycle dwell");

    let tool_entity = model::find_by_name(world, TOOL).expect("tap tool entity");
    let tool = model::tool_of(world, &tool_entity).expect("tap tool definition");
    let pitch_feed = tool.pitch * tool.rpm;
    assert!(pitch_feed.is_finite() && pitch_feed > 0.0, "catalog tap has a positive pitch x rpm feed");
    let cycle_moves = &path.moves[cycle.first_move..];
    let feed_moves: Vec<_> = cycle_moves
        .iter()
        .take_while(|m| m.kind == MoveKind::Feed)
        .collect();
    assert_eq!(feed_moves.len(), 2, "tap feeds down and reverses out");
    assert!(
        feed_moves.iter().all(|m| m.feed == Some(pitch_feed)),
        "both tapping directions use pitch x rpm = {pitch_feed}: {feed_moves:?}"
    );
    path
}

/// Exercise the real operation-to-stream dump and the public stream parser.
/// The generated cycle kind is checked first, so a dump/parser agreement
/// cannot conceal an upstream mode collapse.
fn assert_stream_cycle(world: &World, want: CycleKind, marker: &str) -> String {
    let records = posting::stream_records(world).expect("generated operation dumps to records");
    let stream = cnc::serialize(&records);
    assert!(stream.contains(marker), "stream keeps its explicit cycle word: {stream}");
    let parsed = cnc::parse(&stream).expect("serialized operation stream parses");
    let cycle = parsed
        .into_iter()
        .find_map(|record| match record {
            Record::Cycle(cycle) => Some(cycle),
            _ => None,
        })
        .expect("stream has one canned cycle");
    assert_eq!(cycle.kind, want, "parser preserves the cycle word");
    stream
}

fn temp_project(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camcraft-tapping-{tag}-{}.camproj", std::process::id()))
}

#[test]
fn floating_mode_is_distinct_editable_undoable_and_round_trips_its_cached_cycle() {
    let (world, mut history) = fresh_tap_world();
    create_and_generate(&world, &mut history, CycleChoice::Tap, 1.5);

    // The established `Tap` choice remains rigid and preserves its
    // legacy no-dwell generator behavior even when a dwell was requested.
    let rigid = assert_generated(&world, CycleKind::Tap, 0.0);
    assert_stream_cycle(&world, CycleKind::Tap, "CYCLE TAP ");
    assert_eq!(rigid.cycles[0].kind, CycleKind::Tap);

    let floating_edit = EditOperationParams::new(&world, OP, params(CycleChoice::FloatingTap, 1.5))
        .expect("edit to floating tap");
    history.execute(&world, floating_edit);
    let OpParams::Drill { cycle, .. } = operation(&world).params else {
        panic!("drill parameters remain a drill operation");
    };
    assert_eq!(cycle, CycleChoice::FloatingTap, "edit stores the explicit mode");
    history.execute(&world, GenerateOperation::new(&world, OP).expect("generate floating tap"));

    assert_generated(&world, CycleKind::FloatingTap, 1.5);
    let floating_stream = assert_stream_cycle(&world, CycleKind::FloatingTap, "CYCLE FLOATINGTAP ");

    // Both the operation's choice and the independently cached cycle kind
    // survive the actual project serializer and loader.
    let project_path = temp_project("floating-roundtrip");
    project::save(&world, &project_path).expect("save floating tap project");
    let saved = std::fs::read_to_string(&project_path).expect("read saved project");
    assert!(saved.contains("\"cycle\": \"floating_tap\""), "explicit params spelling: {saved}");
    assert!(saved.contains("\"kind\": \"floating_tap\""), "cached CycleDto spelling: {saved}");
    let loaded = new_world();
    project::load(&loaded, &project_path).expect("load floating tap project");
    let _ = std::fs::remove_file(&project_path);
    let OpParams::Drill { cycle, .. } = operation(&loaded).params else {
        panic!("loaded parameters remain a drill operation");
    };
    assert_eq!(cycle, CycleChoice::FloatingTap);
    assert_generated(&loaded, CycleKind::FloatingTap, 1.5);
    assert_stream_cycle(&loaded, CycleKind::FloatingTap, "CYCLE FLOATINGTAP ");
    assert_eq!(floating_stream, assert_stream_cycle(&world, CycleKind::FloatingTap, "CYCLE FLOATINGTAP "));

    // Undo the regeneration, then the edit: history restores the previous
    // mode and its still-cached rigid path. Redo restores floating intent.
    assert!(history.undo(&world), "undo floating generation");
    assert!(history.undo(&world), "undo floating-mode edit");
    let OpParams::Drill { cycle, .. } = operation(&world).params else {
        panic!("undone parameters remain a drill operation");
    };
    assert_eq!(cycle, CycleChoice::Tap);
    assert_generated(&world, CycleKind::Tap, 0.0);
    assert!(history.redo(&world), "redo floating-mode edit");
    let OpParams::Drill { cycle, .. } = operation(&world).params else {
        panic!("redone parameters remain a drill operation");
    };
    assert_eq!(cycle, CycleChoice::FloatingTap);
    history.execute(&world, GenerateOperation::new(&world, OP).expect("regenerate after redo"));
    assert_generated(&world, CycleKind::FloatingTap, 1.5);
}

#[test]
fn legacy_tap_spelling_and_stream_are_stable_and_unknown_kinds_are_rejected() {
    let (world, mut history) = fresh_tap_world();
    create_and_generate(&world, &mut history, CycleChoice::Tap, 1.5);
    assert_generated(&world, CycleKind::Tap, 0.0);
    let stream = assert_stream_cycle(&world, CycleKind::Tap, "CYCLE TAP ");

    let project_path = temp_project("legacy-tap");
    project::save(&world, &project_path).expect("save legacy tap project");
    let saved = std::fs::read_to_string(&project_path).expect("read saved project");
    assert!(saved.contains("\"cycle\": \"tap\""), "legacy choice remains `tap`: {saved}");
    assert!(saved.contains("\"kind\": \"tap\""), "legacy cached kind remains `tap`: {saved}");
    let loaded = new_world();
    project::load(&loaded, &project_path).expect("load legacy tap project");
    let _ = std::fs::remove_file(&project_path);
    let OpParams::Drill { cycle, .. } = operation(&loaded).params else {
        panic!("loaded parameters remain a drill operation");
    };
    assert_eq!(cycle, CycleChoice::Tap);
    assert_generated(&loaded, CycleKind::Tap, 0.0);

    let mut unknown_choice: serde_json::Value = serde_json::from_str(&saved).expect("valid saved project JSON");
    let entities = unknown_choice["entities"].as_array_mut().expect("project entities");
    let entity = entities
        .iter_mut()
        .find(|entity| entity["name"] == OP)
        .expect("saved operation entity");
    entity["operation"]["params"]["cycle"] = serde_json::json!("unknown_tapping_mode");
    let bad_choice = serde_json::to_string(&unknown_choice).expect("serialize unknown choice");
    assert!(project::open(&new_world(), &bad_choice).is_err(), "unknown parameter mode is refused");

    let mut unknown_cache: serde_json::Value = serde_json::from_str(&saved).expect("valid saved project JSON");
    let entities = unknown_cache["entities"].as_array_mut().expect("project entities");
    let entity = entities
        .iter_mut()
        .find(|entity| entity["name"] == OP)
        .expect("saved operation entity");
    entity["operation"]["cache"]["cycles"][0]["kind"] = serde_json::json!("unknown_cycle_kind");
    let bad_cache = serde_json::to_string(&unknown_cache).expect("serialize unknown cache kind");
    assert!(project::open(&new_world(), &bad_cache).is_err(), "unknown cached cycle kind is refused");

    let unknown_stream = stream.replacen("CYCLE TAP ", "CYCLE UNKNOWNCYCLE ", 1);
    assert_ne!(unknown_stream, stream, "the unknown cycle word fixture changed");
    assert!(cnc::parse(&unknown_stream).is_err(), "unknown stream cycle words are refused");
}
