//! #1316: the overhang backstop of the mid-turn sweep (law c3) reads
//! the whole turn, not 4 stations x 5 poses of it. A holder 100 mm up
//! a long stack swings under a ledge, through stock the ledge hides
//! from above, BETWEEN two of the poses the backstop used to sample
//! (20 and 40 degrees of an 80-degree turn): each sampled disc stood
//! beside the hidden stock, the holder itself went through it.
//!
//! Provenance: our own fixture; the law is the Prime Directive's
//! ("never gouge, never collide"). No vendor material consulted.
//!
//! Gate map (#1316): covers camcraft_sim::verify (sweep, turn_cells,
//! hidden, reach).

use camcraft_sim::cl::{Move, MoveKind, Toolpath, angles_to_axis};
use camcraft_sim::geo2d::{Loop, Point2, Region};
use camcraft_sim::geometry::{Point3, Triangle, cube};
use camcraft_sim::verify::{Mode, ToolGeom, VerifyContext, Warning, verify};

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Region {
    let pts = vec![Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)];
    match Loop::new(pts).and_then(|l| Region::new(vec![l])) {
        Ok(r) => r,
        Err(e) => panic!("region: {e:?}"),
    }
}

fn path(moves: Vec<Move>) -> Toolpath {
    Toolpath { moves, cycles: Vec::new(), notes: Vec::new(), comps: Vec::new(), threads: Vec::new(), probes: Vec::new(), breaks: Vec::new(), plasma: None }
}

/// A post X [45, 58], Y [20, 30], Z [0, 100] carrying a ledge X [45, 58],
/// Y [-10, 30], Z [100, 102]: under the ledge at Y [-10, 20) the stock
/// is hidden from above. `ledge = false` is the bare post.
fn part(ledge: bool) -> Vec<Triangle> {
    let mut f: Vec<Triangle> = cube(Point3::new(45.0, 20.0, 0.0), Point3::new(58.0, 30.0, 100.0)).facets.to_vec();
    if ledge {
        f.extend(cube(Point3::new(45.0, -10.0, 100.0), Point3::new(58.0, 30.0, 102.0)).facets.iter().cloned());
    }
    f
}

/// A slim 6 mm cutter with 100 mm of flute and no shank band: the one
/// band the reach laws judge is the holder (r 4, 4 long), standing
/// 100 mm above the tip.
fn long_reach() -> ToolGeom {
    ToolGeom { radius: 3.0, nose_radius: 0.0, flute_len: 100.0, shank_radius: 3.0, shank_len: 0.0, holder_radius: 4.0, holder_len: 4.0 }
}

/// The band stack's reports for a one-move path turning from home to
/// B `to_deg` while the tip sinks from z 0 to z -1 at the origin. At
/// 30 degrees the holder stands at X ~51, Z ~87..90: under the ledge,
/// in the hidden stock. The stock's column samples (its corners and
/// edge midpoints, the post's walls) all stand far from its swing.
fn swing(to_deg: f64, ledge: bool) -> Vec<Warning> {
    let stock = rect(-40.0, -40.0, 160.0, 40.0);
    let facets = part(ledge);
    let tool = long_reach();
    let cx = VerifyContext {
        region: &stock,
        stock: Some(&stock),
        model: Some(&facets),
        tool: &tool,
        mode: Mode::Area,
        stock_top_z: 120.0,
        clearance_z: 130.0,
        gouge_tol: 0.01,
        z_tol: 1e-3,
        ..Default::default()
    };
    let mut m = Move::new(MoveKind::Feed, Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, -1.0), Some(300.0));
    m.axis = angles_to_axis([0.0, to_deg, 0.0]);
    assert!(m.axis.is_some());
    verify(&path(vec![m]), &cx)
        .into_iter()
        .filter(|w| matches!(w, Warning::ShankHit { .. } | Warning::SweepHit { .. }))
        .collect()
}

#[test]
fn a_holder_crossing_hidden_stock_between_two_sampled_poses_is_convicted() {
    // The backstop's old poses (0, 20, 40, 60, 80 degrees) put the
    // holder's disc at X <= 39.6 and X >= 60.3: beside the hidden stock
    // X [45, 58] both times. The swing between them crosses it.
    let w = swing(80.0, true);
    assert_eq!(w, vec![Warning::SweepHit { at: 0, part: false }], "{w:?}");
}

#[test]
fn the_same_swing_beside_a_bare_post_is_clean() {
    // No ledge, nothing hidden: the column samples see no stock in the
    // swing, and the backstop has nothing to read.
    let w = swing(80.0, false);
    assert!(w.is_empty(), "{w:?}");
}

#[test]
fn a_swing_that_stops_short_of_the_hidden_stock_is_clean() {
    // 15 degrees: the holder's disc reaches X ~31 at most.
    let w = swing(15.0, true);
    assert!(w.is_empty(), "{w:?}");
}
