//! Pins of the overhang backstop's column (#1313 review of the #1303
//! port): `backstop_columns` reads main's #1303 column - the rim's exact
//! dip, lowered by the span floor - and a hidden-stock probe asserts the
//! exact verdict on both sides of that reading. Our own fixture; no
//! vendor material.

use super::*;

/// A stand-in for [`super::super::hidden`]'s annulus law: one column of
/// stock hidden under a ledge at `p`, its top at `top`. The band's disc
/// `|q - pc| <= r_out` outside the kerf `|q - tip| <= r_in` meets it when
/// the band's bottom `h` stands under its top.
fn hidden_post(p: Point2, top: f64) -> impl Fn(Point2, Point2, f64, f64, f64, usize) -> bool {
    move |pc, tip, r_in, r_out, h, _upto| {
        (p.x - pc.x).hypot(p.y - pc.y) <= r_out + 1e-9 && (p.x - tip.x).hypot(p.y - tip.y) > r_in && h < top
    }
}

/// A 100 -> 260 passes the tool straight down (A 180, the middle sampled
/// pose). Band 20 up, radius 10, 30 long (L = 50); cutter radius 2. The
/// hidden post at (5, 0) sits inside the A 180 column (radius 10 about
/// the tip) and outside every other pose's column (they lean 22 mm or
/// more off the tip along Y), so only A 180's reading reaches it:
/// - its own exact column: -L, the rim dips R·sin 180 = 0;
/// - the span floor toward A 140 / A 220: -L - R·sin 40 = -56.43 (the
///   lerp between the samples leans as far as 40 degrees off down);
/// - the whole radius (the reading before #1303): -L - R = -60.
///
/// The backstop must read -56.43: the post topped at -53 is convicted
/// (only the span floor reaches it - deleting the floor's line reads -50
/// and clears it), the post topped at -58 is clean (the whole radius
/// would read -60 and convict it). A revert either way is caught.
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

    // The reading itself: the lowest column bottom over the post.
    let cols = backstop_columns(&m, from, to, &bands, r, 0.0, 0.01);
    let over = |&&(pc, tip, rb, _): &&(Point2, Point2, f64, f64)| {
        (p.x - pc.x).hypot(p.y - pc.y) <= rb + 1e-9 && (p.x - tip.x).hypot(p.y - tip.y) > r + tol
    };
    let read = cols.iter().filter(over).map(|c| c.3).fold(f64::INFINITY, f64::min);
    assert!((read - floor).abs() < 1e-9, "the backstop reads {read} over the post, the exact dip and span floor {floor}");
}
