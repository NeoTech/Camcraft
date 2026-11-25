//! Pins of the Outside blend (#3934) on a rectangular stock box around a
//! round part, as the 2D Offset's Outside band plans it: the first rings
//! follow the stock edge (parallel to the box, one step apart), then the
//! rings follow the selected edge (parallel to the part), and the part
//! wall is last. Before #3934 every ring after the outer edge was an
//! offset of the part, clipped by the box, so the second ring did not run
//! parallel to the stock at all.

use super::super::level_rings::{level_pieces, unswept};
use super::outside_blend;
use crate::geo2d::{Loop, Point2, Region};
use crate::machining2d::level_order::{Piece, WALL};
use crate::machining2d::pass_start::{PassStart, StartSide};
use crate::machining2d::ring_fill::RING;

const R: f64 = 4.0;
const STEP: f64 = 2.0;
const BOX: (f64, f64) = (40.0, 25.0);

fn region(loops: Vec<Vec<Point2>>) -> Region {
    let loops = loops.into_iter().map(|l| Loop::new(l).unwrap_or_else(|e| panic!("loop: {e:?}"))).collect();
    Region::new(loops).unwrap_or_else(|e| panic!("region: {e:?}"))
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Point2> {
    vec![Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)]
}

fn disc(r: f64) -> Vec<Point2> {
    (0..128)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / 128.0;
            Point2::new(r * a.cos(), r * a.sin())
        })
        .collect()
}

fn stock() -> Region {
    region(vec![rect(-BOX.0, -BOX.1, BOX.0, BOX.1)])
}

fn part() -> Region {
    region(vec![disc(12.0)])
}

/// The band the Outside operation reads (the stock grown by the tool
/// diameter, less the part) and its cutter-centre area (the band eroded
/// by the tool radius).
fn band_and_center() -> (Region, Region) {
    let band = stock().offset(2.0 * R).and_then(|s| s.difference(&part())).expect("band");
    let center = band.offset(-R).expect("center");
    (band, center)
}

fn pieces() -> Vec<Piece> {
    let (band, center) = band_and_center();
    let side = StartSide { start: PassStart::Outside, stock: Some(stock()), around_part: true };
    match level_pieces(&band, &center, R, STEP, true, &side, R) {
        Ok(Some(ps)) => ps,
        other => panic!("the blend family: {:?}", other.map(|o| o.map(|p| p.len()))),
    }
}

/// How far `q` stands outside the stock box in XY (0 inside it): the
/// Euclidean distance, so a point on the rounded corner of a box offset
/// reads the offset distance, not its Chebyshev projection.
fn outside_box(q: Point2) -> f64 {
    let dx = (-BOX.0 - q.x).max(q.x - BOX.0).max(0.0);
    let dy = (-BOX.1 - q.y).max(q.y - BOX.1).max(0.0);
    dx.hypot(dy)
}

/// How far `q` stands outside the part disc.
fn off_part(q: Point2) -> f64 {
    (q.x.hypot(q.y) - 12.0).max(0.0)
}

fn assert_level(name: &str, ps: &[Piece], level: u32, dist: impl Fn(Point2) -> f64 + Copy, want: f64) {
    let at: Vec<&Piece> = ps.iter().filter(|p| p.level == level).collect();
    assert!(!at.is_empty(), "{name}: no piece at level {level}");
    for p in at {
        for &q in &p.pts {
            let got = dist(q);
            assert!((got - want).abs() < 0.05, "{name}: ({:.3}, {:.3}) reads {got:.3}, not {want:.3}", q.x, q.y);
        }
    }
}

/// #3934 pin: ring 0 follows the stock edge, ring 1 is one step in from
/// it, and both run parallel to the box. Before the blend, every ring
/// after ring 0 was an offset of the part: ring 1 then varied from the
/// stock and read the part's distance, never the stock's.
#[test]
fn the_first_rings_follow_the_stock_edge() {
    let ps = pieces();
    assert_level("ring 0", &ps, 0, outside_box, R);
    assert_level("ring 1", &ps, RING, outside_box, R - STEP);
}

/// #3934 pin: after the two fronts meet, a ring follows the selected
/// edge, one step off the part keep-out, and the part wall is cut last
/// (level `WALL`). The stock side never runs into the part's keep-out.
#[test]
fn the_last_rings_follow_the_selected_edge_and_the_wall_is_last() {
    let ps = pieces();
    let top = ps.iter().filter(|p| p.level != WALL).map(|p| p.level).max().expect("a ring");
    let deep = top / (2 * RING);
    assert!(deep >= 2, "the fixture has a stock and a part front: deepest {deep}");
    // The part ring next to the wall: hole 1, whose level is the top one.
    assert_level("part ring 1", &ps, top, off_part, R + STEP);
    // The part wall, last.
    assert_level("part wall", &ps, WALL, off_part, R);
    // None of the stock rings stands inside the part's keep-out.
    for p in &ps {
        if p.level < RING * (deep + 1) {
            for &q in &p.pts {
                assert!(off_part(q) >= R - 1e-6, "a stock ring reaches the part: ({:.3}, {:.3})", q.x, q.y);
            }
        }
    }
}

/// #3934: the family leaves no part of the cutter-centre area standing,
/// so the completeness guard keeps it (a plan it rejects would fall back
/// to the Auto order, and the shape pins above would not be the posted
/// plan).
#[test]
fn the_blend_cuts_the_cutter_centre_area_whole() {
    let (_, center) = band_and_center();
    let ps = pieces();
    let passes: Vec<Vec<Point2>> = ps
        .iter()
        .map(|p| {
            let mut w = p.pts.clone();
            if p.closed && let Some(&f) = w.first() {
                w.push(f);
            }
            w
        })
        .collect();
    let left = unswept(&center, &passes, R).expect("unswept");
    assert!(left <= 0.1, "the blend leaves {left:.3} mm2 standing");
}

/// #3934: the early rings are whole loops, and no piece is a core piece.
/// Where a ring reaches the medial line it is an open stretch that ends in
/// the core, never on a wall (an end on the part wall would cut the full
/// width there, the old Outside's defect).
#[test]
fn the_early_rings_are_whole_and_ends_never_stand_on_a_wall() {
    let ps = pieces();
    assert!(ps.len() >= 3 * RING as usize, "the fixture has both fronts: {}", ps.len());
    assert!(ps.iter().all(|p| !p.core), "a core piece in the blend family");
    for p in ps.iter().filter(|p| p.level <= RING) {
        assert!(p.closed, "ring {} of the stock side is not a whole loop", p.level / RING);
    }
    // An open stretch's ends stand at the medial line, at least one step
    // off the part keep-out, never on the part wall.
    for p in ps.iter().filter(|p| !p.closed) {
        for q in [p.pts.first(), p.pts.last()].into_iter().flatten() {
            assert!(off_part(*q) >= R + STEP - 1e-3, "a blend end stands on the part wall: ({:.3}, {:.3})", q.x, q.y);
        }
    }
}

/// The helper refuses a step that never converges instead of looping.
#[test]
fn a_step_that_never_converges_is_refused_by_name() {
    let (_, center) = band_and_center();
    let hc = part().offset(R).expect("keep-out");
    let mut out = Vec::new();
    let Err(e) = outside_blend(&center, &hc, 0.0, R, true, 16, &mut out) else { panic!("zero step must be refused") };
    assert!(e.to_string().contains("positive and finite"), "{e}");
}
