//! #793 acceptance: a built-in taper holder (BT30 / BT40 / CAT40, ER
//! chuck or Weldon) drops into a tool's holder through one undoable
//! command, lands as an editable drawn profile, and is judged by the
//! fixture laws as the shape it is: the BT30 ER20 flange strikes a wall
//! the plain holder cylinder misses, a long-gauge Weldon clears where a
//! short one collides, undo restores the previous holder, and nonsense
//! picks are refused by name.
//!
//! Gate map (M4.4): covers app::commands, app::tool_body, app::tools.

use camcraft_app::commands::{AddTool, CommandError, CreateOperation, GenerateOperation, History, SetStock, SetToolBody, TagFixture};
use camcraft_app::geometry::{Point3, Triangle};
use camcraft_app::holder_library;
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tool_body;
use camcraft_app::tools::{catalog, Tool};
use camcraft_app::verify::{FixturePart, Warning};
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;

/// A 4 mm end mill on a 6 mm shank: flutes 12, shank 40 - the holder's
/// nose face stands 52 above the tip.
const TOOL: &str = "EM-0400-2FL";
const CUT_Z: f64 = 19.5;
const CLEAR_Z: f64 = 25.0;
const NOSE: f64 = 52.0;

fn catalog_tool(name: &str) -> Tool {
    catalog().into_iter().find(|(n, _)| *n == name).map(|(_, t)| t).expect("catalog entry")
}

fn face_world() -> (World, History) {
    let world = new_world();
    let mut history = History::default();
    history.execute(
        &world,
        SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Location::default()).expect("stock"),
    );
    history.execute(&world, AddTool::new(&world, TOOL, catalog_tool(TOOL)).expect("tool"));
    history.execute(
        &world,
        CreateOperation::new(
            &world,
            "Face1",
            TOOL,
            SrcRef::Stock,
            OpParams::Face {
                holes: Default::default(),
                tilt: None,
                blend_turns: false,
                stepover: 3.0,
                clearance_z: Height::fixed(CLEAR_Z),
                cut_z: Height::fixed(CUT_Z),
                top_z: None,
                stepdown: None,
                feed: 1200.0,
                plunge_feed: 300.0,
                climb: true,
            },
        )
        .expect("face op"),
    );
    history.execute(&world, GenerateOperation::new(&world, "Face1").expect("generate"));
    (world, history)
}

/// A vertical wall `dx` off the path's +X edge (radially, from the
/// tool axis at its outermost pass), from `zb` up `h`, along the
/// path's whole Y span.
fn plant_wall(world: &World, history: &mut History, dx: f64, zb: f64, h: f64) {
    let op = model::find_by_name(world, "Face1").expect("Face1");
    let cache = model::toolpath_of(world, &op).expect("generated");
    let xmax = cache.path.moves.iter().flat_map(|m| [m.from.x, m.to.x]).fold(f64::NEG_INFINITY, f64::max);
    let (y0, y1) = cache
        .path
        .moves
        .iter()
        .flat_map(|m| [m.from.y, m.to.y])
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), y| (a.min(y), b.max(y)));
    let x = xmax + dx;
    let (a, b, c, d) =
        (Point3::new(x, y0, zb), Point3::new(x, y1, zb), Point3::new(x, y1, zb + h), Point3::new(x, y0, zb + h));
    let e = model::spawn(world, "Wall1");
    model::set_facets(world, &e, vec![Triangle::new(a, b, c), Triangle::new(a, c, d)].into());
    model::set_rev(world, &e, 1);
    history.execute(world, TagFixture::new(world, "Wall1", None).expect("tagged"));
    history.execute(world, GenerateOperation::new(world, "Face1").expect("regenerate"));
}

fn holder_hits(world: &World) -> usize {
    let op = model::find_by_name(world, "Face1").expect("Face1");
    let cache = model::toolpath_of(world, &op).expect("generated");
    cache.warnings.iter().filter(|w| matches!(w, Warning::FixtureHit { fixture_part: FixturePart::Holder, .. })).count()
}

fn regenerate(world: &World, history: &mut History) {
    history.execute(world, GenerateOperation::new(world, "Face1").expect("regenerate"));
}

fn drawn(world: &World) -> Option<Vec<(f64, f64)>> {
    let e = model::find_by_name(world, TOOL).expect("tool");
    tool_body::body_of(world, &e).and_then(|b| b.spun).map(|s| s.points().iter().map(|p| (p.z, p.r)).collect())
}

#[test]
fn a_library_holder_lands_as_its_drawn_profile() {
    let (world, mut history) = face_world();
    history.execute(&world, SetToolBody::library(&world, TOOL, "BT30 ER20", None, None).expect("a library pick"));
    let want = holder_library::find("BT30 ER20").and_then(|e| e.points(NOSE, e.gauge_len)).expect("draws");
    assert_eq!(drawn(&world), Some(want.iter().map(|p| (p.z, p.r)).collect()), "nose at the shank top, default gauge 60");
    let (judged, body) = tool_body::judged(&world, &model::find_by_name(&world, TOOL).expect("tool"), catalog_tool(TOOL)).expect("judged");
    assert!(body.is_some(), "the laws read its bands");
    assert!((judged.holder_d - 46.0).abs() < 1e-6, "the enclosing cylinder is the BT30 flange: Ø{}", judged.holder_d);
    assert!((judged.holder_len - 60.0).abs() < 1e-6, "nose face to gauge line: {}", judged.holder_len);
    // Gauge length and projection are the caller's to set.
    history.execute(&world, SetToolBody::library(&world, TOOL, "BT30 ER20", Some(80.0), Some(60.0)).expect("set"));
    let top = drawn(&world).and_then(|p| p.last().map(|q| q.0));
    assert_eq!(top, Some(140.0));
}

/// The user's 770MX case: a wall beside the BT30 flange (22.5 mm off
/// the tool axis, at the flange's height) is missed by the catalog's
/// Ø44 × 35 holder cylinder - it tops out below the wall - and struck
/// by the Ø46 flange; raised clear of the gauge line it is clear again.
#[test]
fn the_bt30_er20_flange_strikes_a_wall_the_cylinder_misses() {
    let (world, mut history) = face_world();
    // Flange: 90-110 above the tip at the default gauge of 60.
    plant_wall(&world, &mut history, 22.5, CUT_Z + 95.0, 10.0);
    assert_eq!(holder_hits(&world), 0, "the holder cylinder misses the wall");
    history.execute(&world, SetToolBody::library(&world, TOOL, "BT30 ER20", None, None).expect("pick"));
    regenerate(&world, &mut history);
    assert!(holder_hits(&world) > 0, "the BT30 flange strikes it");

    let (world, mut history) = face_world();
    plant_wall(&world, &mut history, 22.5, CLEAR_Z + NOSE + 60.0 + 5.0, 10.0);
    history.execute(&world, SetToolBody::library(&world, TOOL, "BT30 ER20", None, None).expect("pick"));
    regenerate(&world, &mut history);
    assert_eq!(holder_hits(&world), 0, "a wall above the gauge line is clear");
}

/// A wall 15 mm off the axis at 95-105 above the tip: the 60 mm Weldon
/// puts its Ø46 flange there and collides; a 100 mm one reaches past
/// it with its Ø25 body and clears.
#[test]
fn a_long_gauge_weldon_clears_where_a_short_one_collides() {
    let (world, mut history) = face_world();
    plant_wall(&world, &mut history, 15.0, CUT_Z + 95.0, 10.0);
    history.execute(&world, SetToolBody::library(&world, TOOL, "BT30 Weldon 6 mm", Some(60.0), None).expect("short"));
    regenerate(&world, &mut history);
    assert!(holder_hits(&world) > 0, "the short holder's flange collides");
    history.execute(&world, SetToolBody::library(&world, TOOL, "BT30 Weldon 6 mm", Some(100.0), None).expect("long"));
    regenerate(&world, &mut history);
    assert_eq!(holder_hits(&world), 0, "the long holder's Ø25 body clears");
}

#[test]
fn undo_restores_the_previous_holder() {
    let (world, mut history) = face_world();
    history.execute(&world, SetToolBody::library(&world, TOOL, "BT40 ER20", None, None).expect("first"));
    let bt40 = drawn(&world);
    history.execute(&world, SetToolBody::library(&world, TOOL, "BT30 ER20", None, None).expect("second"));
    assert_ne!(drawn(&world), bt40);
    assert!(history.undo(&world));
    assert_eq!(drawn(&world), bt40, "undo: the BT40 again");
    assert!(history.undo(&world));
    assert_eq!(drawn(&world), None, "undo: the holder cylinder again");
    assert!(history.redo(&world));
    assert_eq!(drawn(&world), bt40);
}

#[test]
fn nonsense_picks_are_refused_by_name() {
    let (world, _history) = face_world();
    let refuse = |entry: &str, gauge: Option<f64>, proj: Option<f64>, says: &str| {
        match SetToolBody::library(&world, TOOL, entry, gauge, proj) {
            Err(CommandError::Invalid(m)) => assert!(m.contains(says), "{m:?} should say {says:?}"),
            Err(e) => panic!("wrong refusal {e}"),
            Ok(_) => panic!("accepted a pick that should say {says:?}"),
        }
    };
    refuse("BT35 ER20", None, None, "no holder 'BT35 ER20'");
    refuse("BT30 Weldon 12 mm", None, None, "does not fit");
    refuse("BT30 ER20", Some(20.0), None, "gauge length is shorter");
    refuse("BT30 ER20", Some(f64::NAN), None, "finite");
    refuse("BT30 ER20", None, Some(5.0), "reaches into the flutes");
    assert!(matches!(SetToolBody::library(&world, "nope", "BT30 ER20", None, None), Err(CommandError::NotFound(_))));
}
