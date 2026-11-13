//! Pins of the overhang backstop's reading (#1313 review of the #1303
//! port, carried onto #1316's swept cells): the backstop reads main's
//! #1303 column - the rim's exact dip, lowered by the span floor - and a
//! hidden-stock probe asserts the exact verdict on both sides of that
//! reading. Our own fixture; no vendor material.

use super::*;
use crate::geometry::Point3;

/// A stand-in for [`super::super::hidden`]'s `hidden_in_cover`: one
/// column of stock hidden under a ledge at `p`, its top at `top`. An ask
/// meets it when `p` is in the ask's box and footprint (the kerf
/// excluded by the footprint's own test) and the ask's plane `h` stands
/// under its top.
fn hidden_post(p: Point2, top: f64) -> impl Fn((Point2, Point2), f64, &dyn Fn() -> Vec<Point2>, &dyn Fn(Point2) -> bool, usize) -> bool {
    move |(lo, hi), h, _probes, covers, _upto| {
        p.x >= lo.x && p.x <= hi.x && p.y >= lo.y && p.y <= hi.y && covers(p) && h < top
    }
}

/// A 100 -> 260 passes the tool straight down (A 180, the middle sampled
/// pose). Band 20 up, radius 10, 30 long (L = 50); cutter radius 2. The
/// hidden post at (5, 0) sits inside the A 180 column (radius 10 about
/// the tip) and outside the reach of the poses whose span floors run
/// lower (A 140 / A 220 lean 22 mm or more off the tip along Y), so the
/// lowest reading over it is A 180's:
/// - its own exact column: -L, the rim dips R·sin 180 = 0 (the cells
///   about it read a little lower, by their sag and kappa bound);
/// - the span floor toward A 140 / A 220: -L - R·sin 40 = -56.43 (the
///   lerp between the samples leans as far as 40 degrees off down);
/// - the whole radius (the reading before #1303): -L - R = -60.
///
/// The backstop must read -56.43: the post topped at -53 is convicted
/// (only the span floor reaches it - deleting the floor reads about
/// -51.5 and clears it), the post topped at -58 is clean (the whole
/// radius would read -60 and convict it). A revert either way is caught.
#[test]
fn the_backstop_reads_the_exact_dip_lowered_by_the_span_floor() {
    let (from, to) = ([100.0, 0.0, 0.0], [260.0, 0.0, 0.0]);
    let (above, band_r, band_len) = (20.0, 10.0, 30.0);
    let bands = [(above, band_r, band_len)];
    let (r, tol) = (2.0, 0.01);
    let o = Point3::new(0.0, 0.0, 0.0);
    let m = Move::new(MoveKind::Feed, o, o, Some(300.0));
    let p = Point2::new(5.0, 0.0);
    let l = above + band_len;
    let floor = -l - band_r * 40f64.to_radians().sin();

    let verdict = |top: f64| {
        let mut memo = std::collections::HashMap::new();
        let none_in = |_: Point2| false;
        // No sample columns: the column law reads nothing, the backstop is asked.
        let flat = |_: Point2| -> Steps { unreachable!("no sample column to read") };
        swing_hit(&m, 1, from, to, &bands, r, (tol, 0.0), 0.01, &[], &mut memo, &none_in, &flat, &none_in, &hidden_post(p, top))
    };
    assert_eq!(verdict(-53.0), Some(false), "only the span floor reaches the post at -53");
    assert_eq!(verdict(-58.0), None, "the exact dip stands clear of the post at -58 (the whole radius would not)");

    // The reading itself: the lowest plane of any ask whose footprint holds the post.
    let kerf2 = (r + tol) * (r + tol);
    let asks = super::super::turn_hidden::backstop_asks(&m.segments3(0.01), from, to, &bands, r, 0.0);
    let read = asks.iter().filter(|a| a.covers(p, kerf2)).map(|a| a.plane).fold(f64::INFINITY, f64::min);
    assert!((read - floor).abs() < 1e-9, "the backstop reads {read} over the post, the exact dip and span floor {floor}");
}
