//! The edge route of the link stage (#4177): where no straight link is
//! proved, the tool walks along a loop of the cutter-centre area that both
//! ends of the link stand near (within a tool radius), the shorter way
//! first, and each chord of the walk is proved as a link. The walk joins
//! the loop at the places of the loop nearest to the two ends, or at its
//! vertices only, so that it can run exactly on a ring that lies on the
//! loop (a ring at offset 0).
//! Provenance: ported from the edge route of the ring links of
//! `camcraft-motion` (`machining2d/ring_links.rs`, `edge_route`, #3824);
//! our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 5: Link".

use super::proof::Proof;
use crate::{Point2, near, seg_dist};

fn dist(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// The nearest place of the closed loop `pts` to `q`: the index of its
/// segment (from `pts[i]` to the next point), the place along it (0 to 1)
/// and the distance.
fn project(pts: &[Point2], q: Point2) -> Option<(usize, f64, Point2, f64)> {
    let n = pts.len();
    let mut best: Option<(usize, f64, Point2, f64)> = None;
    for i in 0..n {
        let (Some(&a), Some(&b)) = (pts.get(i), pts.get((i + 1) % n)) else {
            continue;
        };
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len2 = dx * dx + dy * dy;
        let t = if len2 > 0.0 { (((q.x - a.x) * dx + (q.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
        let d = seg_dist(q, a, b);
        if d.is_finite() && best.is_none_or(|(_, _, _, bd)| d < bd) {
            best = Some((i, t, Point2::new(a.x + t * dx, a.y + t * dy), d));
        }
    }
    best
}

/// The loop vertices between two places of a loop of `n` points, forward
/// (in the loop's direction) or back.
fn between(n: usize, (ia, ta): (usize, f64), (ib, tb): (usize, f64), forward: bool) -> Vec<usize> {
    if n == 0 {
        return Vec::new();
    }
    if forward {
        let count = match (ib + n - ia) % n {
            0 if tb >= ta => 0,
            0 => n,
            c => c,
        };
        (1..=count).map(|k| (ia + k) % n).collect()
    } else {
        let count = match (ia + n - ib) % n {
            0 if tb <= ta => 0,
            0 => n,
            c => c,
        };
        (0..count).map(|k| (ia + n - k) % n).collect()
    }
}

fn length(w: &[Point2]) -> f64 {
    w.windows(2).map(|s| dist(s[0], s[1])).sum()
}

/// The proved walk from `a` to `b` along a loop of the centre area, the
/// shortest first; `None` when no walk is proved.
pub(super) fn edge_route(proof: &Proof<'_>, a: Point2, b: Point2) -> Option<Vec<Point2>> {
    let gap = proof.radius();
    let mut ways: Vec<Vec<Point2>> = Vec::new();
    for l in proof.region().center().loops() {
        let pts = l.points();
        if pts.len() < 3 {
            continue;
        }
        let (Some((ia, ta, pa, da)), Some((ib, tb, pb, db))) = (project(pts, a), project(pts, b)) else {
            continue;
        };
        if da > gap || db > gap {
            continue;
        }
        for forward in [true, false] {
            let mid: Vec<Point2> = between(pts.len(), (ia, ta), (ib, tb), forward).into_iter().filter_map(|i| pts.get(i).copied()).collect();
            // Joined at the nearest places of the loop (the walk runs on
            // the edge), or at its vertices only (the walk runs exactly on
            // a ring that lies on the loop).
            for (on, off) in [(Some(pa), Some(pb)), (None, None)] {
                if mid.is_empty() && on.is_none() {
                    continue;
                }
                let mut w = vec![a];
                w.extend(on);
                w.extend(mid.iter().copied());
                w.extend(off);
                w.push(b);
                w.dedup_by(|x, y| near(*x, *y));
                ways.push(w);
            }
        }
    }
    ways.sort_by(|x, y| length(x).total_cmp(&length(y)));
    ways.into_iter().find(|w| proof.path(w).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_walk_between_two_places_goes_each_way_round() {
        assert_eq!(between(4, (0, 0.5), (2, 0.5), true), vec![1, 2]);
        assert_eq!(between(4, (0, 0.5), (2, 0.5), false), vec![0, 3]);
        assert_eq!(between(4, (1, 0.2), (1, 0.8), true), Vec::<usize>::new());
        assert_eq!(between(4, (1, 0.8), (1, 0.2), true).len(), 4);
        assert_eq!(between(4, (1, 0.8), (1, 0.2), false), Vec::<usize>::new());
    }
}
