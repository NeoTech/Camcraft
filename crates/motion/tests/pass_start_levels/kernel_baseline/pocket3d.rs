//! #4153 3D Pocket baseline: one level of a 40 mm square pocket round a
//! 10 mm island (the #3361 fixture), for Waterline and Offset rings with
//! Auto, Inside and Outside, Zigzag, the Adaptive, and a Helix entry on
//! two levels.

use super::pin;
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::geometry::{Point3, Triangle};
use camcraft_motion::links::Links;
use camcraft_motion::load::DEFAULT_OPTIMAL_LOAD;
use camcraft_motion::machining2d::{PassStart, PocketStrategy};
use camcraft_motion::machining3d::{Pocket3DParams, pocket3d};

fn square(x0: f64, y0: f64, s: f64) -> Region {
    let l = Loop::new(vec![
        Point2::new(x0, y0),
        Point2::new(x0 + s, y0),
        Point2::new(x0 + s, y0 + s),
        Point2::new(x0, y0 + s),
    ]);
    Region::new(vec![l.expect("loop")]).expect("region")
}

fn cuboid(lo: [f64; 3], hi: [f64; 3]) -> Vec<Triangle> {
    let v = |i: usize| Point3::new([lo[0], hi[0]][i & 1], [lo[1], hi[1]][(i >> 1) & 1], [lo[2], hi[2]][(i >> 2) & 1]);
    let quad = |a: usize, b: usize, c: usize, d: usize| [Triangle::new(v(a), v(b), v(c)), Triangle::new(v(a), v(c), v(d))];
    [quad(4, 5, 7, 6), quad(0, 2, 3, 1), quad(0, 1, 5, 4), quad(2, 6, 7, 3), quad(1, 3, 7, 5), quad(2, 0, 4, 6)]
        .into_iter()
        .flatten()
        .collect()
}

/// A floor plate under the pocket and an island through every level.
fn model() -> Vec<Triangle> {
    let mut f = cuboid([-30.0, -30.0, -5.0], [70.0, 70.0, -1.0]);
    f.extend(cuboid([15.0, 15.0, -5.0], [25.0, 25.0, 20.0]));
    f
}

fn params(strategy: PocketStrategy, pass_start: PassStart) -> Pocket3DParams {
    Pocket3DParams {
        links: Links::ROUGH3D,
        tool_diameter: 6.0,
        stepover: 2.4,
        stock_to_leave: 0.0,
        clearance_z: 20.0,
        top_z: 13.0,
        bottom_z: 0.0,
        stepdown: 50.0,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        strategy,
        retract_z: None,
        entry: Entry3D::Plunge,
        helix_radius: 1.5,
        optimal_load: DEFAULT_OPTIMAL_LOAD,
        pass_start,
    }
}

#[test]
fn kernel_baseline_pocket3d() {
    use PassStart::{Auto, Inside, Outside};
    use PocketStrategy::{Adaptive, OffsetRings, Waterline, Zigzag};
    let mut cases: Vec<(String, Pocket3DParams)> = Vec::new();
    for (s, strategy) in [("waterline", Waterline), ("rings", OffsetRings)] {
        for (n, start) in [("auto", Auto), ("inside", Inside), ("outside", Outside)] {
            cases.push((format!("{s}_{n}"), params(strategy, start)));
        }
    }
    cases.push(("zigzag".into(), params(Zigzag, Auto)));
    cases.push(("adaptive".into(), params(Adaptive, Auto)));
    let two_levels = Pocket3DParams { stepdown: 6.0, entry: Entry3D::Helix, ..params(Waterline, Auto) };
    cases.push(("waterline_helix_levels".into(), two_levels));
    for (case, p) in cases {
        pin("pocket3d", &case, &pocket3d(&model(), &square(0.0, 0.0, 40.0), &p).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
}
