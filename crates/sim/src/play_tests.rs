//! The playback built and driven from plain inputs, with no world
//! (#2268): `FieldPlay`, `PlayIndex` and `PlayWorker` are the sim
//! crate's, and the app only gathers the inputs.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use super::{FieldPlay, PlayInputs, PlayOpPlan, Progress};
use crate::cl::{Move, MoveKind};
use crate::geometry::{Point3, Triangle};
use crate::play_worker::PlayWorker;

/// A test's monotonic clock: time since the first read.
fn test_clock() -> Duration {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    ORIGIN.get_or_init(Instant::now).elapsed()
}

/// A solid box as facet soup, outward CCW.
fn box_facets(min: [f64; 3], max: [f64; 3]) -> Vec<Triangle> {
    let v = |i: usize| Point3::new([min[0], max[0]][i & 1], [min[1], max[1]][(i >> 1) & 1], [min[2], max[2]][(i >> 2) & 1]);
    let quad = |a: usize, b: usize, c: usize, d: usize| [Triangle::new(v(a), v(b), v(c)), Triangle::new(v(a), v(c), v(d))];
    let mut t = Vec::with_capacity(12);
    for q in [quad(4, 5, 7, 6), quad(0, 2, 3, 1), quad(0, 1, 5, 4), quad(2, 6, 7, 3), quad(1, 3, 7, 5), quad(2, 0, 4, 6)] {
        t.extend(q);
    }
    t
}

fn feed(from: Point3, to: Point3) -> Move {
    Move { rotary: None, kind: MoveKind::Feed, from, to, feed: Some(500.0), arc: None, axis: None }
}

fn op(name: &str, moves: Vec<Move>) -> PlayOpPlan {
    PlayOpPlan {
        name: name.into(),
        moves,
        radius: 2.0,
        ball: false,
        cone: None,
        corner: None,
        drill: false,
        rotary: None,
        lathe: None,
        undercut: None,
    }
}

/// A 20 x 20 x 10 block and one flat 4 mm slot across its top, 2 mm deep.
fn inputs() -> PlayInputs {
    let slot = feed(Point3::new(-5.0, 10.0, 8.0), Point3::new(25.0, 10.0, 8.0));
    PlayInputs {
        stock_facets: box_facets([0.0; 3], [20.0, 20.0, 10.0]),
        ops: vec![op("Slot", vec![slot])],
        cell: 0.5,
        part: None,
        tol: Ok(crate::stock_sim::DEV_TOL),
        rest: Vec::new(),
        clock: test_clock,
    }
}

/// Inside the slot, below the block's top.
const IN_SLOT: Point3 = Point3 { x: 10.0, y: 10.0, z: 9.0 };

#[test]
fn field_play_cuts_plain_inputs_with_no_world() {
    let mut play = FieldPlay::from_seed(inputs()).expect("playback builds");
    assert_eq!((play.total(), play.at()), (1, 0));
    assert!(!play.judged(), "no part: the picture is unjudged material");
    assert!(play.view().stock.distance(IN_SLOT) < 0.0, "uncut: the slot is still material");
    assert!(play.seek(1).expect("cut"), "the picture changed");
    assert_eq!(play.at(), 1);
    assert!(play.view().stock.distance(IN_SLOT) > 0.0, "cut: the slot is air");
    // Back to the start restores the uncut stock from the first keyframe.
    assert!(play.seek(0).expect("restore"));
    assert!(play.view().stock.distance(IN_SLOT) < 0.0);
}

#[test]
fn play_worker_lands_on_the_end_from_plain_inputs() {
    let w = PlayWorker::spawn(inputs()).expect("worker");
    assert_eq!(w.index().total(), 1);
    w.request(1);
    let mut seen = 0;
    let deadline = test_clock() + Duration::from_secs(30);
    let frame = loop {
        seen = w.wait_published(seen, Duration::from_millis(200));
        if let Some(r) = w.take() {
            let f = r.expect("no refusal");
            if f.at == 1 {
                break f;
            }
        }
        assert!(test_clock() < deadline, "the worker never landed on move 1");
    };
    assert!(frame.view.stock.distance(IN_SLOT) > 0.0, "the landed picture holds the cut");
}

#[test]
fn play_index_maps_a_playhead_by_name() {
    let a = feed(Point3::new(0.0, 0.0, 12.0), Point3::new(1.0, 0.0, 12.0));
    let mut seed = inputs();
    seed.ops = vec![op("A", vec![a; 3]), op("B", vec![a; 2])];
    let ix = seed.index();
    assert_eq!(ix.total(), 5);
    let at = |op: &str, segs_done, span| ix.index_at(&Progress { op: op.into(), segs_done, span });
    assert_eq!(at("B", 1, 2), Some(4));
    // A span one longer than the moves carries a connector rapid.
    assert_eq!(at("B", 1, 3), Some(3));
    assert_eq!(at("C", 0, 1), None);
}
