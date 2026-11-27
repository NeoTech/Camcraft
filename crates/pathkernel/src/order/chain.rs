//! The sequence of the curves in the order stage (#4177): the levels of the
//! ring index in the walk's sense, and the chain order of the split pieces
//! of a band walked from the stock edge (#3934): the runs of consecutive
//! levels in one open region form a chain, and the chains are cut one
//! after the other.
//! Provenance: the chain order is ported from the Adaptive air plan of
//! `camcraft-motion` (`machining2d/adaptive/air_chain.rs`, `chains`), as
//! the design record decided for #3934; it is the return pass of
//! contour-parallel pocketing (Held, "On the Computational Geometry of
//! Pocket Machining", 1991). Our own design record
//! `docs/decisions/4024-path-kernel-design.md` "Stage 4: Order".

use crate::guide::Curve;
use crate::{Point2, seg_dist};

/// The curve indices grouped by ring index, the groups in the walk's
/// order (the deepest first when `deeper_first`), each group in the order
/// of the guide set.
pub(super) fn levels(curves: &[Curve], deeper_first: bool) -> Vec<Vec<usize>> {
    let mut ks: Vec<u32> = curves.iter().map(Curve::k).collect();
    ks.sort_unstable();
    ks.dedup();
    if deeper_first {
        ks.reverse();
    }
    ks.into_iter().map(|k| (0..curves.len()).filter(|&i| curves.get(i).is_some_and(|c| c.k() == k)).collect()).collect()
}

fn dist(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// The chain of each run: each run of a level joins the nearest chain
/// whose last run ends within `reach` of both of its ends (one run for
/// each chain and level), else it starts a new chain. The result gives the
/// chain number of each run (chains numbered in the order they start).
/// `ends[i]` are the first and the last point of run `i` as it will be cut.
pub(super) fn chains(levels: &[Vec<usize>], ends: &[(Point2, Point2)], reach: f64) -> Vec<usize> {
    let mut out: Vec<Vec<usize>> = Vec::new();
    for runs in levels {
        let mut taken = vec![false; out.len()];
        for &run in runs {
            let Some(&(a, b)) = ends.get(run) else {
                continue;
            };
            let near = |c: &Vec<usize>| c.last().and_then(|&l| ends.get(l)).map(|&(la, lb)| dist(la, a).max(dist(lb, b)));
            let best = out
                .iter()
                .enumerate()
                .filter(|(i, _)| !taken.get(*i).copied().unwrap_or(true))
                .filter_map(|(i, c)| near(c).map(|d| (i, d)))
                .filter(|&(_, d)| d <= reach)
                .min_by(|x, y| x.1.total_cmp(&y.1));
            match best.and_then(|(i, _)| out.get_mut(i).map(|c| (i, c))) {
                Some((i, c)) => {
                    c.push(run);
                    if let Some(t) = taken.get_mut(i) {
                        *t = true;
                    }
                }
                None => out.push(vec![run]),
            }
        }
    }
    let mut id = vec![usize::MAX; ends.len()];
    for (c, runs) in out.iter().enumerate() {
        for &r in runs {
            if let Some(slot) = id.get_mut(r) {
                *slot = c;
            }
        }
    }
    id
}

/// True when the curves `a` and `b` come within `reach` of each other.
fn near(a: &Curve, b: &Curve, reach: f64) -> bool {
    let bounds = |c: &Curve| {
        c.pts().iter().fold((f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY), |m, p| {
            let q = p.at();
            (m.0.min(q.x), m.1.min(q.y), m.2.max(q.x), m.3.max(q.y))
        })
    };
    let (x, y) = (bounds(a), bounds(b));
    if x.0 > y.2 + reach || y.0 > x.2 + reach || x.1 > y.3 + reach || y.1 > x.3 + reach {
        return false;
    }
    let segs = |c: &Curve| -> Vec<(Point2, Point2)> {
        let p: Vec<Point2> = c.pts().iter().map(|q| q.at()).collect();
        let close = if c.closed() { p.first().zip(p.last()).map(|(f, l)| (*l, *f)) } else { None };
        p.windows(2).map(|w| (w[0], w[1])).chain(close).collect()
    };
    let one = |from: &Curve, to: &[(Point2, Point2)]| from.pts().iter().any(|q| to.iter().any(|(s, e)| seg_dist(q.at(), *s, *e) <= reach));
    one(a, &segs(b)) || one(b, &segs(a))
}

/// The runs that each run waits for: the runs of the level before it in
/// the walk (`k + 1` when the deepest come first, else `k - 1`) that come
/// within `reach` of it. A ring is cut only when its neighbours on the
/// side the walk comes from are cut (the order of the offset tree, Held
/// 1991).
pub(super) fn needs(runs: &[Curve], deeper_first: bool, reach: f64) -> Vec<Vec<usize>> {
    runs.iter()
        .map(|c| {
            let before = if deeper_first { c.k().checked_add(1) } else { c.k().checked_sub(1) };
            (0..runs.len()).filter(|&j| runs.get(j).is_some_and(|o| Some(o.k()) == before && near(c, o, reach))).collect()
        })
        .collect()
}
