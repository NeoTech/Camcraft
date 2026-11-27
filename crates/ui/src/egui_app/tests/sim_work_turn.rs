//! #4266 review findings 1-3 as scene tests: the work turn turns only
//! the steel that turns with the work (a fixed fixture stays in the
//! machine frame); one display frame shares the turn between the
//! marker, the cached toolpath lines, the model and the ground shadow;
//! and the turned geometry is cached under its actual inputs - a
//! paused playhead computes nothing again, a scrubbed one rebuilds.
//! No pixels: positions and counters only. Provenance: our own tests
//! over our own display transform; no vendor material consulted.

use std::sync::Arc;

use camcraft_app::commands::{ImportModel, SetFixtureFixed, TagFixture};
use camcraft_app::fixture;
use camcraft_app::geometry::{Mesh, Point3, cube};
use camcraft_app::model;
use camcraft_app::schema::Location;
use camcraft_app::sim::{Script, Seg, WorkTurn};

use super::*;
use crate::state::Sim;

/// An A rotary about +X through the origin, the mill-turn line.
fn work() -> WorkTurn {
    WorkTurn { origin: [0.0; 3], dir: [1.0, 0.0, 0.0] }
}

/// A standstill frame at the reviewer's pose: the tool axis stands at
/// -Y (the B turn's angles, held), the tip at `[0, 5, 0]`, the work
/// turn carried about `op`'s name.
fn posed_script(op: &str, work: Option<WorkTurn>) -> Script {
    let mut sc = Script { ops: vec![op.to_owned()], ..Default::default() };
    sc.segs.push(Seg {
        from: Point3::new(0.0, 5.0, 0.0),
        to: Point3::new(0.0, 5.0, 0.0),
        arc: None,
        secs: 1.0,
        dwell: 0.0,
        feed: Some(100.0),
        op: 0,
        abc: ([90.0, 0.0, 0.0], [90.0, 0.0, 0.0]),
        pl: 1.0,
        rl: 0.0,
        work,
    });
    sc.cum.push(1.0);
    sc.total = 1.0;
    sc
}

/// Play the transport paused on `sc`'s pose.
fn play(state: &mut State, sc: Script) {
    state.sim_script_cache = Some((state.doc_version, Arc::new(sc)));
    state.sim = Some(Sim { t: 0.5, playing: false, speed: 1.0 });
}

fn place(state: &mut State, name: &str, lo: [f64; 3], hi: [f64; 3]) {
    let (lo, hi) = (Point3::new(lo[0], lo[1], lo[2]), Point3::new(hi[0], hi[1], hi[2]));
    state.history.execute(&state.world, ImportModel::from_mesh(name, Mesh::from_facets(cube(lo, hi).facets.to_vec())));
    state.doc_version += 1;
}

fn tag(state: &mut State, name: &str) {
    state.history.execute(&state.world, TagFixture::new(&state.world, name, None).expect("tag"));
    state.doc_version += 1;
}

/// A part that holds a vertex within `tol` of `p`.
fn part_near(parts: &[crate::viewport::GpuPart], p: [f64; 3], tol: f32) -> bool {
    parts.iter().any(|g| {
        g.verts.iter().any(|v| v.iter().zip(p).all(|(a, b)| (f64::from(*a) - b).abs() < f64::from(tol)))
    })
}

/// The reviewed pose stands -Y upright with the -90ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â° turn: a point at
/// `[0, 5, 0]` is carried to `[0, 0, -5]` about the A line.
#[test]
fn the_reviewed_pose_carries_the_point_the_reviewer_wrote() {
    let out = camcraft_app::sim_machine_view::posed_tool([0.0, 5.0, 0.0], [0.0, -1.0, 0.0], &work());
    let (p, axis) = out.expect("the pose stands upright");
    assert!(
        p.iter().zip([0.0, 0.0, -5.0]).all(|(a, b)| (a - b).abs() < 1e-9),
        "the marker sits on the carried tip: {p:?}"
    );
    assert_eq!(axis, [0.0, 0.0, 1.0]);
}

/// Review finding 1: a turning fixture rides the turn, a fixture
/// declared fixed does not - the same membership the gate judges with
/// (`fixture::workholding_for`), applied before every scene transform.
#[test]
fn the_scene_turns_the_turning_fixture_and_keeps_the_fixed_one() {
    let mut state = State::new();
    place(&mut state, "TurnJaw", [40.0, 0.0, 0.0], [50.0, 10.0, 10.0]);
    place(&mut state, "FixedJaw", [0.0, 40.0, 0.0], [10.0, 50.0, 10.0]);
    tag(&mut state, "TurnJaw");
    tag(&mut state, "FixedJaw");
    state.history.execute(&state.world, SetFixtureFixed::new(&state.world, "FixedJaw", true).expect("fixed"));
    state.doc_version += 1;
    assert!(
        fixture::def_of(&state.world, &model::find_by_name(&state.world, "FixedJaw").expect("jaw"))
            .is_some_and(|d| d.fixed),
        "the gate keeps the fixed jaw fixed"
    );
    play(&mut state, posed_script("Nothing1", Some(work())));
    let scene = world::collect(&state);
    let steel: Vec<_> = scene.parts.iter().filter(|g| !g.stock && !g.fill).collect();
    assert_eq!(steel.len(), 2, "the two jaws, nothing else");
    // The turning jaw moved: its high corner (50, 10, 10) stands where
    // the turn carries it, (x, y, z) -> (x, z, -y).
    assert!(part_near(&scene.parts, [50.0, 10.0, -10.0], 1e-3), "the turning jaw rides the turn");
    assert!(!part_near(&scene.parts, [50.0, 10.0, 10.0], 1e-3), "no jaw keeps the turning jaw's original corner");
    // The fixed jaw did not move.
    assert!(part_near(&scene.parts, [10.0, 50.0, 10.0], 1e-3), "the fixed jaw stays in the machine frame");
    assert!(!part_near(&scene.parts, [10.0, 10.0, -50.0], 1e-3), "the fixed jaw is never carried");
}

/// Review finding 2: marker, tool body, cached path lines, model and
/// ground shadow share ONE display transform at a nonzero rotary pose.
#[test]
fn one_display_frame_shares_the_turn_between_marker_path_model_and_shadow() {
    let mut state = State::new();
    super::world::seed(&mut state);
    state.ground_shadow = true;
    let sc = posed_script("Contour1", Some(work()));
    // The marker: on the carried tip, never the original point.
    let (p, axis) = crate::egui_app::shell::sim_marker_pose(&sc, 0.5).expect("a pose on the timeline");
    assert!((p[0].abs() + p[1].abs() + (p[2] + 5.0).abs()) < 1e-9, "marker at [0, 0, -5]: {p:?}");
    assert_eq!(axis, [0.0, 0.0, 1.0], "the spindle stands upright");
    play(&mut state, sc);

    // The tool body: drawn whole from the carried tip.
    let scene = world::collect(&state);
    let solids: Vec<_> = scene.parts.iter().filter(|g| !g.stock && !g.fill).collect();
    let tool = solids.last().expect("the posed tool part");
    let zmin = tool.verts.iter().map(|v| f64::from(v[2])).fold(f64::INFINITY, f64::min);
    assert!((zmin + 5.0).abs() < 1e-3, "the tool starts at the carried tip [0, 0, -5]: {zmin}");

    // The model: the block's top-back edge rides the turn.
    assert!(part_near(&scene.parts, [0.0, 30.0, 0.0], 1e-3), "the model is drawn turned");
    assert!(!part_near(&scene.parts, [0.0, 0.0, 30.0], 1e-3), "no scene part keeps the model's original corner");

    // The shadow: built from the TURNED body - the footprint's y span
    // is the turned z span (0..30), not the original 0..60.
    assert!(!scene.shade.shadow.is_empty(), "the turned model casts a shadow");
    let ymax = scene.shade.shadow.iter().map(|v| v[1]).fold(f32::NEG_INFINITY, f32::max);
    // The turned footprint spans y 0..30 (the blur's margin adds a few
    // cells); the unturned block would reach past 60.
    assert!(ymax < 40.0, "the shadow follows the turned body (y max {ymax})");

    // The path: the cached lines read turned, exactly the unturned
    // cache's answer under the SAME turn (one display frame).
    let view = crate::work_turn::play_view(&state).expect("the play view");
    let turned = crate::path_overlay::toolpath_lines(&state);
    assert!(!turned.is_empty(), "the seed's Contour1 draws its path");
    state.sim = None;
    let plain = crate::path_overlay::toolpath_lines(&state);
    assert!(!plain.is_empty(), "the same cache answers the unturned frame too");
    assert_ne!(plain, turned, "the turn moved the path");
    let mut expected = plain;
    crate::work_turn::turn_lines(&mut expected, Some(&view));
    assert_eq!(turned, expected, "one transform, one frame: the path is the model's turn");
}

/// Review finding 3 at scene level: a paused playhead computes no
/// turned geometry again; a scrub to a new turn (or a new source)
/// builds again.
#[test]
fn a_paused_playhead_turns_no_geometry_again_and_a_scrub_does() {
    let mut state = State::new();
    place(&mut state, "Part", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let part = model::find_by_name(&state.world, "Part").expect("part");
    play(&mut state, posed_script("Nothing1", Some(work())));
    let _ = world::collect(&state);
    let builds = state.frame_cache.turned_verts.builds();
    assert_eq!(builds, 1, "the first frame turns the model once");
    for _ in 0..8 {
        let _ = world::collect(&state);
    }
    assert_eq!(state.frame_cache.turned_verts.builds(), builds, "a paused playhead reads the turned cache");
    // A scrub to a different turn: a changed input.
    let mut moved = work();
    moved.origin = [0.0, 0.0, 4.0];
    state.sim_script_cache = Some((state.doc_version, Arc::new(posed_script("Nothing1", Some(moved)))));
    let _ = world::collect(&state);
    assert_eq!(state.frame_cache.turned_verts.builds(), builds + 1, "a scrubbed turn builds again");
    // The transport down (the fallback draw): no turn work at all.
    state.sim = None;
    let _ = world::collect(&state);
    assert_eq!(state.frame_cache.turned_verts.builds(), builds + 1, "no turn, no turned cache work");
    // A new source at the same turn: a moved body builds again.
    model::set_transform(
        &state.world,
        &part,
        Location { pos: Point3::new(30.0, 0.0, 0.0), ..Default::default() },
    );
    state.sim = Some(Sim { t: 0.5, playing: false, speed: 1.0 });
    let _ = world::collect(&state);
    assert_eq!(state.frame_cache.turned_verts.builds(), builds + 2, "a moved source builds its turn again");
}
