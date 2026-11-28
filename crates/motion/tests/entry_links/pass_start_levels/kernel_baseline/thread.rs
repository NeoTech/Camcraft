//! #4153 Thread baseline: an M10 x 1.5 internal thread and an external
//! one with a Ø6 thread mill, right and left hand, climb and
//! conventional, one row and two rows, and two holes.

use super::pin;
use camcraft_motion::cl::ThreadHand;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::machining2d::{ThreadHole, ThreadParams, ThreadPasses, thread_mill};

fn hole(x: f64, y: f64) -> ThreadHole {
    ThreadHole { xy: Point2::new(x, y), radius: 5.0, top_z: 0.0, floor_z: -6.0, stepped: false, through: false }
}

fn params(external: bool, hand: ThreadHand, climb: bool, rows: u32) -> ThreadParams {
    ThreadParams {
        through_run: None,
        holes: vec![hole(10.0, 20.0)],
        pitch: 1.5,
        tool_diameter: 6.0,
        rows,
        infeed: 0.3,
        external,
        hand,
        climb,
        passes: ThreadPasses::default(),
        clearance_z: 10.0,
        r_z: None,
        retract_z: None,
        feed: 400.0,
        plunge_feed: 120.0,
    }
}

#[test]
fn kernel_baseline_thread() {
    let mut two = params(false, ThreadHand::Right, true, 1);
    two.holes.push(hole(40.0, 20.0));
    let cases: [(&str, ThreadParams); 7] = [
        ("internal_right_climb", params(false, ThreadHand::Right, true, 1)),
        ("internal_right_conventional", params(false, ThreadHand::Right, false, 1)),
        ("internal_left_climb", params(false, ThreadHand::Left, true, 1)),
        ("internal_right_two_rows", params(false, ThreadHand::Right, true, 2)),
        ("external_right_climb", params(true, ThreadHand::Right, true, 1)),
        ("external_left_conventional", params(true, ThreadHand::Left, false, 1)),
        ("internal_two_holes", two),
    ];
    for (case, p) in cases {
        pin("thread", case, &thread_mill(&p).unwrap_or_else(|e| panic!("{case}: {e:?}")));
    }
}
