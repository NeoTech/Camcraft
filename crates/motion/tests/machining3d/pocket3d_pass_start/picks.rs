//! #3329, the 3D Pocket slice: the start and exit points seat the
//! levels of the parent suite's pocket (40 mm square, the 10 mm island,
//! a tool of radius 3), now on three levels (Z 8, 3, 0). The start
//! re-seats the first ring of each level's first pass and runs on to its
//! old start; the exit runs the last ring on to its point nearest the
//! exit; a zigzag row chain starts at its nearer end. Each picked path
//! is gouge-checked at every cut move, sampled each 0.25 mm, and cuts
//! all that the automatic path cuts. A module of `pocket3d_pass_start`.
//!
//! Gate map: covers camcraft_motion::machining3d (level_seam,
//! level_columns, picked, ladder).
//! Provenance: our own regression for #3329; no vendor material
//! consulted.

use super::*;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::machining2d::GenError;
use camcraft_motion::machining3d::{LevelSeam, PICK_Z_UNREAD, pocket3d_offset, pocket3d_picked};

/// The three cut levels of the fixture at a 5 mm stepdown.
const LEVELS: [f64; 3] = [8.0, 3.0, 0.0];

fn seam(start: Option<(f64, f64)>, exit: Option<(f64, f64)>) -> LevelSeam {
    let pt = |(x, y)| Point2::new(x, y);
    LevelSeam { start: start.map(pt), exit: exit.map(pt), z_unread: false }
}

fn p3(strategy: PocketStrategy) -> Pocket3DParams {
    Pocket3DParams { stepdown: 5.0, ..params(strategy, PassStart::Auto) }
}

/// The floor plate alone: the zigzag fixture has no island.
fn plate() -> Vec<Triangle> {
    cuboid([-30.0, -30.0, -5.0], [70.0, 70.0, -1.0])
}

fn picked(strategy: PocketStrategy, sm: &LevelSeam) -> Result<Toolpath, GenError> {
    pocket3d_picked(&model(), &square(0.0, 0.0, 40.0), None, &p3(strategy), false, None, None, sm)
}

fn auto(strategy: PocketStrategy) -> Toolpath {
    pocket3d(&model(), &square(0.0, 0.0, 40.0), &p3(strategy)).expect("the automatic ladder")
}

fn near(a: Point3, x: f64, y: f64, z: f64, tol: f64) -> bool {
    (a.x - x).hypot(a.y - y) <= tol && (a.z - z).abs() < 1e-6
}

/// The first plunge onto each cut level, in level order.
fn level_plunges(t: &Toolpath) -> Vec<Point3> {
    LEVELS
        .iter()
        .filter_map(|&z| t.moves.iter().find(|m| m.kind == MoveKind::Plunge && (m.to.z - z).abs() < 1e-6).map(|m| m.to))
        .collect()
}

/// The XY distance from `q` to the closed square `lo..hi`.
fn to_square(q: Point3, lo: f64, hi: f64) -> f64 {
    let dx = (lo - q.x).max(q.x - hi).max(0.0);
    let dy = (lo - q.y).max(q.y - hi).max(0.0);
    dx.hypot(dy)
}

/// The gouge pin: every cut move (feed and plunge), sampled each
/// 0.25 mm along its length, keeps the tool radius off the island
/// (when there is one), stays inside the footprint less the radius,
/// and stays on or above the floor. The fixtures post no arcs.
fn no_gouge(t: &Toolpath, island: bool, side: f64) {
    for m in t.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        assert!(m.arc.is_none(), "the fixture posts no arc: {m:?}");
        let len = ((m.to.x - m.from.x).powi(2) + (m.to.y - m.from.y).powi(2) + (m.to.z - m.from.z).powi(2)).sqrt();
        let n = ((len / 0.25).ceil() as usize).max(1);
        for k in 0..=n {
            let f = k as f64 / n as f64;
            let c = Point3::new(m.from.x + (m.to.x - m.from.x) * f, m.from.y + (m.to.y - m.from.y) * f, m.from.z + (m.to.z - m.from.z) * f);
            if island {
                assert!(to_square(c, 15.0, 25.0) >= R - 0.01, "a cut gouges the island at {c:?}");
            }
            let inside = c.x >= R - 0.01 && c.x <= side - R + 0.01 && c.y >= R - 0.01 && c.y <= side - R + 0.01;
            assert!(inside, "a cut leaves the footprint at {c:?}");
            assert!(c.z >= -1e-6, "a cut goes under the floor at {c:?}");
        }
    }
}

/// The cells of a 0.5 mm grid over the footprint that the tool disc
/// sweeps at level `z` (feeds and plunges that end on it), within
/// `r` of the cutter centre.
fn swept(t: &Toolpath, z: f64, r: f64, side: f64) -> Vec<bool> {
    let segs: Vec<(Point3, Point3)> = t
        .moves
        .iter()
        .filter(|m| m.kind != MoveKind::Rapid && (m.to.z - z).abs() < 1e-6)
        .map(|m| if m.kind == MoveKind::Plunge { (m.to, m.to) } else { (m.from, m.to) })
        .filter(|(a, _)| (a.z - z).abs() < 1e-6)
        .collect();
    let n = (side / 0.5) as usize;
    let mut out = vec![false; n * n];
    for (i, cell) in out.iter_mut().enumerate() {
        let (x, y) = ((i % n) as f64 * 0.5 + 0.25, (i / n) as f64 * 0.5 + 0.25);
        *cell = segs.iter().any(|(a, b)| {
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let dd = dx * dx + dy * dy;
            let s = if dd > 0.0 { (((x - a.x) * dx + (y - a.y) * dy) / dd).clamp(0.0, 1.0) } else { 0.0 };
            (a.x + dx * s - x).hypot(a.y + dy * s - y) <= r
        });
    }
    out
}

/// The picked path cuts all that the automatic path cuts, level by
/// level (the automatic sweep read 0.05 mm narrow, so a cell on the
/// sweep's very rim is not a lattice coin toss).
fn covers(auto: &Toolpath, got: &Toolpath, side: f64) {
    for z in LEVELS {
        let (a, g) = (swept(auto, z, R - 0.05, side), swept(got, z, R, side));
        assert!(a.iter().any(|&c| c), "the automatic path cuts at Z {z}");
        let left = a.iter().zip(&g).filter(|(a, g)| **a && !**g).count();
        assert_eq!(left, 0, "the picked path leaves {left} cells of the automatic cut at Z {z}");
    }
}

/// No pick is the automatic ladder, move for move, for each strategy
/// that takes a pick, plain and on the offset shell.
#[test]
fn no_pick_is_the_automatic_ladder_byte_for_byte() {
    let none = LevelSeam::default();
    for strategy in [PocketStrategy::Waterline, PocketStrategy::OffsetRings, PocketStrategy::Zigzag, PocketStrategy::Auto] {
        assert_eq!(picked(strategy, &none).expect("picked"), auto(strategy), "{strategy:?}");
        let fp = square(0.0, 0.0, 40.0);
        let shell = pocket3d_offset(&model(), &fp, &p3(strategy)).expect("the offset shell");
        let got = pocket3d_picked(&model(), &fp, None, &p3(strategy), true, None, None, &none).expect("picked shell");
        assert_eq!(got, shell, "{strategy:?} on the offset shell");
    }
}

/// The start point (30, 33) is beside the core ring (Waterline, core
/// first): every level descends on it at (30, 32.2), where the
/// automatic ladder descends at the ring's own start (32.2, 32.2).
/// A pick with a Z says once that the Z is not read.
#[test]
fn a_start_pick_seats_the_first_ring_of_every_level() {
    let a = auto(PocketStrategy::Waterline);
    let first = level_plunges(&a);
    assert!(first.iter().all(|q| near(*q, 32.2, 32.2, q.z, 1e-6)), "the automatic descent: {first:?}");
    let sm = LevelSeam { z_unread: true, ..seam(Some((30.0, 33.0)), None) };
    let t = picked(PocketStrategy::Waterline, &sm).expect("the picked ladder");
    let got = level_plunges(&t);
    assert_eq!(got.len(), 3, "one descent per level: {got:?}");
    for (q, z) in got.iter().zip(LEVELS) {
        assert!(near(*q, 30.0, 32.2, z, 1e-6), "Z {z}: the level descends at the start point: {q:?}");
    }
    assert_eq!(t.notes.iter().filter(|n| *n == PICK_Z_UNREAD).count(), 1, "the unread Z is said once: {:?}", t.notes);
    no_gouge(&t, true, 40.0);
    covers(&a, &t, 40.0);
}

/// The exit point (20, 11) is beside the island ring, the last ring:
/// the operation ends at (20, 12) and rises straight to the clearance
/// plane there. With the start point too, both hold.
#[test]
fn an_exit_pick_ends_the_operation_on_the_last_ring() {
    let a = auto(PocketStrategy::Waterline);
    for sm in [seam(None, Some((20.0, 11.0))), seam(Some((30.0, 33.0)), Some((20.0, 11.0)))] {
        let t = picked(PocketStrategy::Waterline, &sm).expect("the picked ladder");
        let n = t.moves.len();
        let (cut, up) = (&t.moves[n - 2], &t.moves[n - 1]);
        assert!(cut.kind == MoveKind::Feed && near(cut.to, 20.0, 12.0, 0.0, 1e-6), "{sm:?}: the last cut ends at the exit: {cut:?}");
        assert!(up.kind == MoveKind::Rapid && near(up.to, 20.0, 12.0, 20.0, 1e-6), "{sm:?}: the rise is straight up: {up:?}");
        no_gouge(&t, true, 40.0);
        covers(&a, &t, 40.0);
    }
}

/// Zigzag rows on a 39.62 mm pocket with no island: the row chain runs
/// from (3.01, 3.01) to (36.61, 36.61). The start point (38, 38) is
/// nearer its far end, so each level's chain runs the other way and
/// descends there (the entry rule's 0.5 mm step along the row off the
/// wall).
#[test]
fn zigzag_rows_start_at_their_end_nearest_the_start_point() {
    let (fp, p) = (square(0.0, 0.0, 39.62), p3(PocketStrategy::Zigzag));
    let a = pocket3d(&plate(), &fp, &p).expect("the automatic rows");
    let t = pocket3d_picked(&plate(), &fp, None, &p, false, None, None, &seam(Some((38.0, 38.0)), None)).expect("picked rows");
    for (q, z) in level_plunges(&a).iter().zip(LEVELS) {
        assert!(near(*q, 3.01, 3.01, z, 1.0), "Z {z}: the automatic rows descend at their first end: {q:?}");
    }
    let got = level_plunges(&t);
    assert_eq!(got.len(), 3, "one descent per level: {got:?}");
    for (q, z) in got.iter().zip(LEVELS) {
        assert!(near(*q, 36.61, 36.61, z, 1.0), "Z {z}: the reversed rows descend at their far end: {q:?}");
        let at = t.moves.iter().position(|m| m.kind == MoveKind::Plunge && m.to == *q).map(|i| t.moves[i + 1].to);
        assert!(at.is_some_and(|c| near(c, 36.61, 36.61, z, 1e-6)), "Z {z}: the rows start at their far end: {at:?}");
    }
    no_gouge(&t, false, 39.62);
    covers(&a, &t, 39.62);
}

fn refusal(r: Result<Toolpath, GenError>) -> String {
    r.err().map(|e| e.to_string()).unwrap_or_default()
}

/// What a pick cannot keep is refused by name: a start beyond the
/// first ring of a level, an exit beyond the end of the operation, a
/// start where the entry rule finds no room near it, and any pick on
/// an Adaptive (HSM) level.
#[test]
fn an_impossible_pick_is_refused_by_name() {
    let e = refusal(picked(PocketStrategy::Waterline, &seam(Some((20.0, 7.0)), None)));
    assert!(e.contains("start point (20.000, 7.000)") && e.contains("first pass") && e.contains("Z 8.000"), "{e}");
    let e = refusal(picked(PocketStrategy::Waterline, &seam(None, Some((38.0, 38.0)))));
    assert!(e.contains("exit point (38.000, 38.000)") && e.contains("last pass") && e.contains("Z 0.000"), "{e}");
    // The rows' far end at (3.01, 36.99) sits 0.38 mm from the row
    // under it: no plunge spot along the row links back to it.
    let e = refusal(pocket3d_picked(&plate(), &square(0.0, 0.0, 40.0), None, &p3(PocketStrategy::Zigzag), false, None, None, &seam(Some((2.0, 38.0)), None)));
    assert!(e.contains("start point (2.000, 38.000)") && e.contains("entry rule at Z 8.000"), "{e}");
    // With no island the Adaptive plan spirals each level (with it,
    // this fixture falls back to rings, which take the pick).
    let adaptive = pocket3d_picked(&plate(), &square(0.0, 0.0, 40.0), None, &p3(PocketStrategy::Adaptive), false, None, None, &seam(Some((20.0, 20.0)), None));
    let e = refusal(adaptive);
    assert!(e.contains("Adaptive (HSM)") && e.contains("start point"), "{e}");
}
