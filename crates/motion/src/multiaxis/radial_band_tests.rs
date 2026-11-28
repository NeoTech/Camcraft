//! The radial band's edge law (#2774, review of #3490): the band edges are
//! a CLOSED interval, so a span that starts or ends ON a band cylinder is
//! kept whole - never an edge lost because another span forces the trim -
//! and each supported one-sided band windows the path by itself. Driven
//! through [`super::band_trim`] and [`super::band_select`] on synthetic
//! planning-frame paths with an exact analytic radius read, so every edge
//! point is pinned to the solver's own tolerance.
//!
//! Provenance: our own fixtures; elementary geometry. No vendor material
//! consulted.

use super::*;
use crate::cl::PathBuilder;

/// The radius read the tests state: the plain distance in the y-z radial
/// plane (the laid frame's radial one, without the retip offset - the edge
/// law is the same read for any convex rho).
fn rho(_x: f64, y: f64, z: f64) -> f64 {
    (y * y + z * z).sqrt()
}

fn pt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn same(a: Point3, b: Point3) -> bool {
    (a - b).dot(a - b) < 1e-12
}

/// A run of chained spans (the raster's pass shape): each span a Feed, the
/// boundary Rapid between unchained spans marking the run edge the trim's
/// link law reads.
fn path(moves: Vec<(Point3, Point3)>) -> Toolpath {
    let mut b = PathBuilder::new(moves[0].0);
    for (k, (a, e)) in moves.into_iter().enumerate() {
        if k > 0 {
            b.push(MoveKind::Rapid, a, None);
        }
        b.push(MoveKind::Feed, e, None);
    }
    b.finish()
}

/// The kept cut points of a trimmed path, in order: every Feed's from and
/// to. The trim may split a chord at an edge into touching records, so the
/// pins read the polyline the records carry, never a record count.
fn cut(path: &Toolpath) -> Vec<Point3> {
    let mut out = Vec::new();
    for m in &path.moves {
        if m.kind == MoveKind::Feed {
            out.push(m.from);
            out.push(m.to);
        }
    }
    out
}

fn in_band(points: &[Point3], lo: f64, hi: f64) {
    for p in points {
        let r = rho(p.x, p.y, p.z);
        assert!(r >= lo - 1e-9 && r <= hi + 1e-9, "{p:?} stands at r{r:.6}, out of the band {lo}..{hi}");
    }
}

/// The #3490 review's probe: a span ON the bottom cylinder, beside a span
/// that forces the trim. The constant-radius span `(0,1,0) -> (2,1,0)` is
/// within the inclusive band R1..R2: the trim starts at the first span's
/// R2 edge, the corner survives, and the last span's END is reached - the
/// old strict comparisons dropped that whole span.
#[test]
fn a_constant_span_on_the_bottom_cylinder_is_kept_to_its_end() {
    let base = path(vec![(pt(0.0, 3.0, 0.0), pt(0.0, 1.0, 0.0)), (pt(0.0, 1.0, 0.0), pt(2.0, 1.0, 0.0))]);
    let got = band_trim(&base, (1.0, 2.0), &rho, "Rotary Parallel").expect("the banded trim");
    let got = cut(&got);
    assert!(got.len() >= 4, "both spans stand in the band: {got:?}");
    in_band(&got, 1.0, 2.0);
    assert!(same(got[0], pt(0.0, 2.0, 0.0)), "the trim starts at the R2 edge: {:?}", got[0]);
    assert!(got.iter().any(|p| same(*p, pt(0.0, 1.0, 0.0))), "the corner on the bottom cylinder stands: {got:?}");
    assert!(same(got[got.len() - 1], pt(2.0, 1.0, 0.0)), "the last span's end is reached: {got:?}");
}

/// A span that starts exactly on the bottom cylinder and rises into the
/// band keeps its start edge (the old `ra > lo` dropped it) and is cut at
/// the R2 crossing.
#[test]
fn a_span_starting_on_the_bottom_edge_keeps_its_start() {
    let base = path(vec![(pt(0.0, 1.0, 0.0), pt(0.0, 3.0, 0.0))]);
    let got = band_trim(&base, (1.0, 2.0), &rho, "Rotary Parallel").expect("the banded trim");
    let got = cut(&got);
    in_band(&got, 1.0, 2.0);
    assert!(same(got[0], pt(0.0, 1.0, 0.0)), "the start on the R1 cylinder stands: {:?}", got[0]);
    assert!(same(got[got.len() - 1], pt(0.0, 2.0, 0.0)), "the R2 exit edge is solved: {got:?}");
}

/// A tangent span at the top cylinder - constant radius `hi` - is inside
/// the closed band and kept whole (the old `rm < hi` dropped its left
/// half): the whole-band hand-off takes it, unchanged.
#[test]
fn a_span_tangent_at_the_top_edge_is_kept_whole() {
    let base = path(vec![(pt(0.0, 2.0, 0.0), pt(4.0, 2.0, 0.0))]);
    let got = band_trim(&base, (1.0, 2.0), &rho, "Rotary Parallel").expect("the banded trim");
    assert_eq!(got.moves, base.moves, "a tangent span is inside the closed band");
}

/// A chord that enters and leaves the band: the kept cut runs the whole
/// in-band stretch edge to edge, from the R2 entry through the R1 low
/// point to the R2 exit, and nothing outside the band stands.
#[test]
fn a_mixed_path_enters_and_leaves_the_band_at_exact_edges() {
    // rho falls from 3 to 1 and rises back to 3 (a V in the radial plane),
    // band R1..R2.
    let base = path(vec![(pt(0.0, 3.0, 0.0), pt(2.0, 1.0, 0.0)), (pt(2.0, 1.0, 0.0), pt(4.0, 3.0, 0.0))]);
    let got = band_trim(&base, (1.0, 2.0), &rho, "Rotary Parallel").expect("the banded trim");
    let got = cut(&got);
    in_band(&got, 1.0, 2.0);
    assert!(same(got[0], pt(1.0, 2.0, 0.0)), "the entry edge: {:?}", got[0]);
    assert!(got.iter().any(|p| same(*p, pt(2.0, 1.0, 0.0))), "the R1 low point stands: {got:?}");
    assert!(same(got[got.len() - 1], pt(3.0, 2.0, 0.0)), "the exit edge: {got:?}");
}

/// The whole-band hand-off agrees with the per-span rule: a path that
/// never leaves the closed band is handed on unchanged, corner moves
/// included - the constant-at-bottom path above must take the hand-off,
/// and the touching-span join must keep it one cut.
#[test]
fn a_path_wholly_inside_the_band_is_handed_on_unchanged() {
    let base = path(vec![(pt(0.0, 1.0, 0.0), pt(2.0, 1.0, 0.0)), (pt(2.0, 1.0, 0.0), pt(4.0, 1.5, 0.0))]);
    let got = band_trim(&base, (1.0, 2.0), &rho, "Rotary Parallel").expect("the banded trim");
    assert_eq!(got.moves, base.moves, "a band that trims nothing changes nothing");
}

/// A Bottom radius alone windows the path from its floor cylinder out
/// (#2774 review): everything under it goes, everything above stays.
#[test]
fn a_bottom_only_band_cuts_everything_under_its_floor() {
    let base = path(vec![(pt(0.0, 0.5, 0.0), pt(2.0, 0.5, 0.0)), (pt(2.0, 0.5, 0.0), pt(4.0, 3.0, 0.0))]);
    let got = band_trim(&base, (1.0, f64::INFINITY), &rho, "Rotary Parallel").expect("the bottom-only band");
    let got = cut(&got);
    in_band(&got, 1.0, f64::INFINITY);
    assert_eq!(got.len(), 2, "only the part at or above R1 stands: {got:?}");
    assert!(same(got[0], pt(2.4, 1.0, 0.0)) && same(got[1], pt(4.0, 3.0, 0.0)), "{got:?}");
}

/// A Top radius alone windows the path from the axis out to its ceiling
/// cylinder (#2774 review): everything above it goes.
#[test]
fn a_top_only_band_cuts_everything_over_its_ceiling() {
    let base = path(vec![(pt(0.0, 0.5, 0.0), pt(2.0, 2.5, 0.0)), (pt(2.0, 2.5, 0.0), pt(4.0, 2.5, 0.0))]);
    let got = band_trim(&base, (0.0, 2.0), &rho, "Rotary Parallel").expect("the top-only band");
    let got = cut(&got);
    in_band(&got, 0.0, 2.0);
    assert!(got.len() >= 2, "the in-band stretch stands: {got:?}");
    assert!(same(got[0], pt(0.0, 0.5, 0.0)), "the run starts at the axis end: {:?}", got[0]);
    assert!(same(got[got.len() - 1], pt(1.5, 2.0, 0.0)), "the run ends at the R2 ceiling: {got:?}");
}

/// The ring select honors the one-sided bands too: a Bottom radius alone
/// keeps the rings at or over the floor, a Top radius alone those at or
/// under the ceiling, and a band that leaves no ring refuses by name.
#[test]
fn the_ring_select_honors_one_sided_bands() {
    // Three constant-radius rings (the contour's shape): r1, r1.5, r3.
    let ring = |r: f64| (pt(0.0, r, 0.0), pt(8.0, r, 0.0));
    let base = path(vec![ring(1.0), ring(1.5), ring(3.0)]);
    let bottom_only = band_select(&base, (1.2, f64::INFINITY), &rho, "Rotary Contour").expect("bottom-only");
    let kept = cut(&bottom_only);
    assert_eq!(kept.len(), 4, "the r1 ring is under the floor: {kept:?}");
    assert!(same(kept[0], pt(0.0, 1.5, 0.0)) && same(kept[2], pt(0.0, 3.0, 0.0)), "{kept:?}");
    let top_only = band_select(&base, (0.0, 2.0), &rho, "Rotary Contour").expect("top-only");
    let kept = cut(&top_only);
    assert_eq!(kept.len(), 4, "the r3 ring is over the ceiling: {kept:?}");
    assert!(same(kept[0], pt(0.0, 1.0, 0.0)) && same(kept[2], pt(0.0, 1.5, 0.0)), "{kept:?}");
    let e = band_select(&base, (3.5, f64::INFINITY), &rho, "Rotary Contour").expect_err("no ring stands over R3.5");
    assert!(e.to_string().contains("crosses no part skin"), "{e}");
}

/// A stated radius that is no distance above the axis refuses by name -
/// the API backstop of the command gate's own range law.
#[test]
fn a_radius_that_is_not_a_distance_above_the_axis_refuses() {
    let radial = RadialHeights { clearance: None, retract: None, top: None, bottom: Some(-1.0) };
    let e = band_window(&radial, "Rotary Contour").expect_err("a negative bottom radius");
    assert!(e.to_string().contains("bottom radius"), "{e}");
    let radial = RadialHeights { clearance: None, retract: None, top: Some(f64::NAN), bottom: Some(1.0) };
    let e = band_window(&radial, "Rotary Contour").expect_err("a NaN top radius");
    assert!(e.to_string().contains("top radius"), "{e}");
    let radial = RadialHeights { clearance: None, retract: None, top: Some(1.0), bottom: Some(2.0) };
    let e = band_window(&radial, "Rotary Contour").expect_err("an unordered band");
    assert!(e.to_string().contains("does not order"), "{e}");
    // The supported one-sided bands answer their window.
    let radial = RadialHeights { clearance: None, retract: None, top: None, bottom: Some(1.0) };
    assert_eq!(band_window(&radial, "Rotary Contour").expect("bottom-only"), Some((1.0, f64::INFINITY)));
    let radial = RadialHeights { clearance: None, retract: None, top: Some(2.0), bottom: None };
    assert_eq!(band_window(&radial, "Rotary Contour").expect("top-only"), Some((0.0, 2.0)));
}
