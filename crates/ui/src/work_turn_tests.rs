//! Unit pins for the work-turn display read (#4266): the membership
//! rule (a fixture declared fixed stays in the machine frame, the
//! default rides the turn), the play view the transport carries, and
//! the line transform that keeps each line's colour. Provenance: our
//! own tests over our own display math; no vendor material consulted.

use camcraft_app::commands::{ImportModel, SetFixtureFixed, TagFixture};
use camcraft_app::geometry::{Mesh, Point3, cube};
use camcraft_app::model;
use camcraft_app::sim::{Script, Seg, WorkTurn};

use super::play_view;
use crate::state::{Sim, State};

fn work() -> WorkTurn {
    WorkTurn { origin: [0.0; 3], dir: [1.0, 0.0, 0.0] }
}

fn place(state: &mut State, name: &str) {
    let lo = Point3::new(40.0, 0.0, 0.0);
    let hi = Point3::new(50.0, 10.0, 10.0);
    state.history.execute(&state.world, ImportModel::from_mesh(name, Mesh::from_facets(cube(lo, hi).facets.to_vec())));
}

fn tag(state: &mut State, name: &str) {
    state.history.execute(&state.world, TagFixture::new(&state.world, name, None).expect("tag"));
}

fn posed() -> Script {
    let mut sc = Script { ops: vec!["Nothing1".into()], ..Default::default() };
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
        work: Some(work()),
    });
    sc.cum.push(1.0);
    sc.total = 1.0;
    sc
}

#[test]
fn the_membership_rule_follows_the_gate() {
    let mut state = State::new();
    place(&mut state, "Part");
    let part = model::find_by_name(&state.world, "Part").expect("part");
    assert!(super::turns_with_work(&state, &part), "a plain model is the work: it rides the turn");
    tag(&mut state, "Part");
    assert!(super::turns_with_work(&state, &part), "a fixture is turning steel unless declared fixed");
    state.history.execute(&state.world, SetFixtureFixed::new(&state.world, "Part", true).expect("fixed"));
    assert!(!super::turns_with_work(&state, &part), "a fixture declared fixed stays in the machine frame");
}

#[test]
fn the_play_view_is_the_transport_s_turn_and_the_fallback_draw_has_none() {
    let mut state = State::new();
    assert!(play_view(&state).is_none(), "no transport, no turn");
    state.sim_script_cache = Some((state.doc_version, std::sync::Arc::new(posed())));
    state.sim = Some(Sim { t: 0.5, playing: false, speed: 1.0 });
    let view = play_view(&state).expect("the pose stands upright under the A line");
    assert_eq!(view.angle.to_degrees(), -90.0, "the -Y pose stands upright with the -90Ãƒâ€šÃ‚Â° turn");
    assert_eq!(view.bits[0], 1, "the key says the turn is present");
    // A pose the rotary alone cannot stand: no turn, the fallback draw.
    let mut sc = posed();
    sc.segs[0].abc = ([0.0, 90.0, 0.0], [0.0, 90.0, 0.0]); // the axis stays +X, never upright
    state.sim_script_cache = Some((state.doc_version, std::sync::Arc::new(sc)));
    assert!(play_view(&state).is_none(), "a leaning pose draws today's picture");
}

#[test]
fn turning_lines_moves_their_points_and_keeps_their_colours() {
    let mut lines = vec![[0.0, 0.0, 0.0, 1.0, 0.5, 0.25], [0.0, 5.0, 0.0, 0.0, 1.0, 0.0]];
    super::turn_lines(&mut lines, None);
    assert_eq!(lines[0], [0.0, 0.0, 0.0, 1.0, 0.5, 0.25], "no turn, no change");
    let view = super::TurnView::key(work().origin, work().dir, -std::f64::consts::FRAC_PI_2);
    super::turn_lines(&mut lines, Some(&view));
    assert_eq!(lines[0][3], 1.0, "the colour rides along");
    assert_eq!(lines[0][5], 0.25);
    // (0, 5, 0) is carried to (0, 0, -5) - the reviewer's pose.
    assert!((lines[1][0].abs() + lines[1][1].abs() + (lines[1][2] + 5.0).abs()) < 1e-4, "{:?}", lines[1]);
}
