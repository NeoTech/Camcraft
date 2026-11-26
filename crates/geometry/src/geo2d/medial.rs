//! The medial axis of a region (#4176): the centre line of a slot or a
//! channel, the skeleton of an area, with the clearance radius at each of
//! its points.
//! Provenance: the medial axis of Blum, "A transformation for extracting
//! new descriptors of shape" (1967); computed as the Voronoi diagram of
//! points sampled on the boundary (Brandt and Algazi, "Continuous skeleton
//! computation by Voronoi diagram", CVGIP: Image Understanding 55, 1992),
//! the Voronoi edges as the duals of the Delaunay edges (`delaunay.rs`),
//! and pruned by the object angle of the two boundary points of each edge
//! (the theta-medial axis of Attali and Montanvert, 1996; Foskey, Lin and
//! Manocha, "Efficient computation of a simplified medial axis", 2003);
//! each branch simplified by Ramer (1972) / Douglas and Peucker (1973).
//! Written new; no vendor material consulted.

use std::collections::HashMap;

use super::delaunay::triangulate;
use super::{GeoError, Point2, Region};

/// The most boundary samples of one medial axis.
pub const MEDIAL_MAX_SAMPLES: usize = 200_000;
/// The clearance (mm) down to which the sampled axis keeps the tolerance:
/// the sample spacing is `sqrt(8 * tol * MEDIAL_REF)` (the error of a
/// Voronoi vertex between two sampled lines at clearance `c` is at most
/// `spacing^2 / (16 c)`).
pub const MEDIAL_REF: f64 = 0.25;

/// One branch of the medial axis: its points, the clearance radius at each
/// point (the distance to its nearest boundary samples: within the sample
/// spacing of the distance to the boundary), and whether it is a closed
/// loop (round a hole of the area).
#[derive(Clone, Debug, PartialEq)]
pub struct MedialCurve {
    pub points: Vec<Point2>,
    pub radii: Vec<f64>,
    pub closed: bool,
}

/// The branches of a medial axis and the count of short side branches
/// (spurs of sampling noise) that the pruning left out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MedialAxis {
    pub curves: Vec<MedialCurve>,
    pub spurs: usize,
}

impl Region {
    /// The boundary sample spacing (mm) of [`Region::medial_axis`] for `tol`.
    #[must_use]
    pub fn medial_spacing(tol: f64) -> f64 {
        (8.0 * tol * MEDIAL_REF).sqrt()
    }

    /// The medial axis of the region within about `tol` (where the
    /// clearance is [`MEDIAL_REF`] or more), keeping the Voronoi edges whose
    /// two boundary points subtend `min_angle` radians or more at the axis.
    /// The output depends on the point set of the boundary only: not on the
    /// order of the loops or the start vertex of a loop. Refused for a tolerance or angle that is
    /// not a positive finite number, an empty region, or more than
    /// [`MEDIAL_MAX_SAMPLES`] samples.
    pub fn medial_axis(&self, tol: f64, min_angle: f64) -> Result<MedialAxis, GeoError> {
        if !(tol.is_finite() && tol > 0.0 && min_angle.is_finite() && min_angle > 0.0) {
            return Err(GeoError::Invalid("medial axis: the tolerance and the angle must be positive and finite".into()));
        }
        if self.is_empty() {
            return Err(GeoError::Invalid("medial axis: the region is empty".into()));
        }
        let h = Region::medial_spacing(tol);
        let pts = samples(self, h)?;
        let tris = triangulate(&pts)?;
        let graph = voronoi(self, &pts, &tris, min_angle, h)?;
        let (edges, spurs) = prune(graph.edges, &graph.nodes, 4.0 * h);
        let curves = branches(&graph.nodes, &edges).into_iter().map(|(ids, closed)| curve(&graph.nodes, &ids, closed, 0.5 * tol)).collect();
        Ok(MedialAxis { curves, spurs })
    }
}

/// The boundary samples: each edge cut into equal parts of `h` or less,
/// sorted (y, then x) and without repeats.
fn samples(r: &Region, h: f64) -> Result<Vec<Point2>, GeoError> {
    let mut out = Vec::new();
    for l in r.loops() {
        let v = l.points();
        for (i, a) in v.iter().enumerate() {
            let b = v[(i + 1) % v.len()];
            let len = (b.x - a.x).hypot(b.y - a.y);
            let n = (len / h).ceil();
            if !n.is_finite() || out.len() as f64 + n > MEDIAL_MAX_SAMPLES as f64 {
                return Err(GeoError::Invalid(format!("medial axis: more than {MEDIAL_MAX_SAMPLES} boundary samples")));
            }
            let n = (n as usize).max(1);
            out.extend((0..n).map(|k| {
                let t = k as f64 / n as f64;
                Point2::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y))
            }));
        }
    }
    out.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
    out.dedup_by(|a, b| (a.x - b.x).abs() <= 1e-9 && (a.y - b.y).abs() <= 1e-9);
    Ok(out)
}

/// A node of the axis: its place and its clearance.
#[derive(Clone, Copy, Debug)]
struct Node {
    at: Point2,
    radius: f64,
}

struct Graph {
    nodes: Vec<Node>,
    edges: Vec<(usize, usize)>,
}

fn circumcenter(a: Point2, b: Point2, c: Point2) -> Option<Point2> {
    let (bx, by, cx, cy) = (b.x - a.x, b.y - a.y, c.x - a.x, c.y - a.y);
    let d = 2.0 * (bx * cy - by * cx);
    let (b2, c2) = (bx * bx + by * by, cx * cx + cy * cy);
    let p = Point2::new(a.x + (cy * b2 - by * c2) / d, a.y + (bx * c2 - cx * b2) / d);
    (d.abs() > 0.0 && p.x.is_finite() && p.y.is_finite()).then_some(p)
}

fn find(uf: &mut [usize], mut i: usize) -> usize {
    while let Some(&j) = uf.get(i) {
        if j == i {
            break;
        }
        let up = uf.get(j).copied().unwrap_or(j);
        if let Some(s) = uf.get_mut(i) {
            *s = up;
        }
        i = up;
    }
    i
}

/// The kept Voronoi edges: duals of the Delaunay edges whose two
/// triangles have their centres inside the region, kept when the two
/// boundary points subtend `min_angle` or more at an end. Centres closer
/// than a thousandth of `h` are one node (four points on one circle).
fn voronoi(r: &Region, pts: &[Point2], tris: &[[usize; 3]], min_angle: f64, h: f64) -> Result<Graph, GeoError> {
    let centres: Vec<Option<Node>> = tris
        .iter()
        .map(|t| {
            let (a, b, c) = (pts[t[0]], pts[t[1]], pts[t[2]]);
            circumcenter(a, b, c).filter(|p| r.contains(*p)).map(|p| Node { at: p, radius: (p.x - a.x).hypot(p.y - a.y) })
        })
        .collect();
    let mut shared: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (i, t) in tris.iter().enumerate() {
        for j in 0..3 {
            let (a, b) = (t[(j + 1) % 3], t[(j + 2) % 3]);
            shared.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let mut pairs: Vec<((usize, usize), (usize, usize))> =
        shared.into_iter().filter_map(|(e, ts)| if let [x, y] = ts[..] { Some((e, (x.min(y), x.max(y)))) } else { None }).collect();
    pairs.sort_unstable();
    let mut uf: Vec<usize> = (0..tris.len()).collect();
    let mut kept = Vec::new();
    for ((p, q), (t1, t2)) in pairs {
        let (Some(Some(n1)), Some(Some(n2))) = (centres.get(t1), centres.get(t2)) else { continue };
        let chord = (pts[p].x - pts[q].x).hypot(pts[p].y - pts[q].y);
        let angle = |n: &Node| 2.0 * (0.5 * chord / n.radius).clamp(-1.0, 1.0).asin();
        if (n1.at.x - n2.at.x).hypot(n1.at.y - n2.at.y) <= 1e-3 * h {
            let (a, b) = (find(&mut uf, t1), find(&mut uf, t2));
            if let Some(s) = uf.get_mut(a.max(b)) {
                *s = a.min(b);
            }
        } else if angle(n1).max(angle(n2)) >= min_angle {
            kept.push((t1, t2));
        }
    }
    // One node for each root that a kept edge reaches.
    let mut id: HashMap<usize, usize> = HashMap::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for (t1, t2) in kept {
        let (a, b) = (find(&mut uf, t1), find(&mut uf, t2));
        if a == b {
            continue;
        }
        let mut node = |t: usize| -> Result<usize, GeoError> {
            if let Some(&i) = id.get(&t) {
                return Ok(i);
            }
            let n = centres.get(t).copied().flatten().ok_or_else(|| GeoError::Invalid("medial axis: a node is outside".into()))?;
            nodes.push(n);
            id.insert(t, nodes.len() - 1);
            Ok(nodes.len() - 1)
        };
        let (x, y) = (node(a)?, node(b)?);
        edges.push((x.min(y), x.max(y)));
    }
    edges.sort_unstable();
    edges.dedup();
    Ok(Graph { nodes, edges })
}

fn dist(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn degrees(n: usize, edges: &[(usize, usize)]) -> Vec<usize> {
    let mut d = vec![0usize; n];
    for &(a, b) in edges {
        for v in [a, b] {
            if let Some(x) = d.get_mut(v) {
                *x += 1;
            }
        }
    }
    d
}

/// Leaves out each side branch shorter than `spur` that ends in a leaf
/// and starts at a fork (two rounds); a branch with two leaf ends is never
/// left out. Returns the kept edges and the count of left-out branches.
fn prune(mut edges: Vec<(usize, usize)>, nodes: &[Node], spur: f64) -> (Vec<(usize, usize)>, usize) {
    let mut count = 0;
    for _ in 0..2 {
        let deg = degrees(nodes.len(), &edges);
        let mut drop: Vec<(usize, usize)> = Vec::new();
        for (ids, closed) in branches(nodes, &edges) {
            let (Some(&s), Some(&e)) = (ids.first(), ids.last()) else { continue };
            let (ds, de) = (deg.get(s).copied().unwrap_or(0), deg.get(e).copied().unwrap_or(0));
            let side = !closed && ((ds == 1 && de >= 3) || (de == 1 && ds >= 3));
            let len: f64 = ids.windows(2).map(|w| dist(nodes[w[0]].at, nodes[w[1]].at)).sum();
            if side && len < spur {
                count += 1;
                drop.extend(ids.windows(2).map(|w| (w[0].min(w[1]), w[0].max(w[1]))));
            }
        }
        if drop.is_empty() {
            break;
        }
        edges.retain(|e| !drop.contains(e));
    }
    (edges, count)
}

/// The branches of the graph: maximal paths between nodes whose degree is
/// not two, then the loops of degree-two nodes. Deterministic: the nodes
/// and the edges are visited in sorted order.
fn branches(nodes: &[Node], edges: &[(usize, usize)]) -> Vec<(Vec<usize>, bool)> {
    let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); nodes.len()];
    for (k, &(a, b)) in edges.iter().enumerate() {
        if let (Some(_), Some(_)) = (nodes.get(a), nodes.get(b)) {
            adj[a].push((b, k));
            adj[b].push((a, k));
        }
    }
    let key = |i: usize| (nodes[i].at.y, nodes[i].at.x);
    let mut order: Vec<usize> = (0..nodes.len()).collect();
    order.sort_by(|&a, &b| key(a).0.total_cmp(&key(b).0).then(key(a).1.total_cmp(&key(b).1)));
    let mut used = vec![false; edges.len()];
    let mut out = Vec::new();
    let walk = |start: usize, first: (usize, usize), used: &mut Vec<bool>| -> Vec<usize> {
        let mut ids = vec![start];
        let (mut at, mut via) = first;
        loop {
            used[via] = true;
            ids.push(at);
            if adj[at].len() != 2 || at == start {
                break;
            }
            match adj[at].iter().find(|(_, e)| !used[*e]) {
                Some(&next) => (at, via) = next,
                None => break,
            }
        }
        ids
    };
    for pass in [false, true] {
        for &s in &order {
            if (adj[s].len() == 2) != pass {
                continue;
            }
            let starts = adj[s].clone();
            for first in starts {
                if used[first.1] {
                    continue;
                }
                let mut ids = walk(s, first, &mut used);
                let closed = ids.len() > 2 && ids.first() == ids.last();
                if closed {
                    ids.pop();
                }
                out.push((ids, closed));
            }
        }
    }
    out
}

/// The points and radii of a branch, simplified within `eps`.
fn curve(nodes: &[Node], ids: &[usize], closed: bool, eps: f64) -> MedialCurve {
    let all: Vec<Node> = ids.iter().filter_map(|&i| nodes.get(i).copied()).collect();
    let keep = if closed && all.len() > 3 {
        // A loop: split at its first point and the point farthest from it.
        let far = (1..all.len()).max_by(|&a, &b| dist(all[0].at, all[a].at).total_cmp(&dist(all[0].at, all[b].at))).unwrap_or(1);
        let mut ring: Vec<Node> = all.clone();
        ring.push(all[0]);
        let mut a = simplify(&ring[..=far], eps);
        let b = simplify(&ring[far..], eps);
        a.pop();
        a.extend(b);
        a.pop();
        a
    } else {
        simplify(&all, eps)
    };
    MedialCurve { points: keep.iter().map(|n| n.at).collect(), radii: keep.iter().map(|n| n.radius).collect(), closed }
}

/// Ramer-Douglas-Peucker: the points of `v` that keep every left-out point
/// within `eps` of the kept polyline. The two ends are kept.
fn simplify(v: &[Node], eps: f64) -> Vec<Node> {
    if v.len() <= 2 {
        return v.to_vec();
    }
    let mut keep = vec![false; v.len()];
    keep[0] = true;
    keep[v.len() - 1] = true;
    let mut stack = vec![(0, v.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let (pa, pb) = (v[a].at, v[b].at);
        let far = (a + 1..b).map(|i| (i, seg(v[i].at, pa, pb))).max_by(|x, y| x.1.total_cmp(&y.1));
        if let Some((i, d)) = far
            && d > eps
        {
            keep[i] = true;
            stack.push((a, i));
            stack.push((i, b));
        }
    }
    v.iter().zip(keep).filter(|(_, k)| *k).map(|(n, _)| *n).collect()
}

fn seg(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    dist(p, Point2::new(a.x + t * dx, a.y + t * dy))
}

#[cfg(test)]
#[path = "medial_tests.rs"]
mod tests;
