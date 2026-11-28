//! The radial ball drop (#2772, the radial drop slice): the exact
//! ball-centre radius at an (axial station, angle) about a STATED
//! rotation axis, for any model - a shape of revolution or not (a
//! helical rotor, a carving). It is the 4-axis wrap's version of the
//! 3-axis drop cutter: the ball comes in along the radial line from
//! far outside and stops at its first contact with the mesh.
//!
//! ## The law
//!
//! The ball centre rides the ray `c(t) = o + t·u`, where `o` is the
//! axis point at the station and `u` the unit radial at the angle (the
//! axis.rs basis law, `rotary_flow_unwrap::basis`). A triangle grown
//! by the ball is convex, so the stations where the ball touches or
//! cuts it are one interval of `t`, and its top end lies on the grown
//! triangle's boundary: a face patch (the plane offset by `r`, foot
//! inside the triangle), an edge cylinder (radius `r`, foot inside
//! the edge) or a vertex sphere. Each gives a closed form; the
//! largest valid root over all facets is the contact. Every station
//! outward of it is a ball clear of every facet, and the shank of a
//! ball nose on the same radial line stays inside the union of those
//! balls - so the contact is gouge-free for the ball and its shank.
//!
//! No contact (no facet within `r` of the ray) is `None`, never a
//! made-up radius. A contact below `r` puts the ball across the axis;
//! the caller judges it (this kernel only measures).
//!
//! Provenance: the drop-cutter construction of published CAM
//! literature (ball against face, edge and vertex), restated for a
//! radial ray; written from geometry and our own design notes, no
//! vendor code, output or text consulted.

use super::axis::Axis;
use super::rotary_flow_unwrap::{basis, Basis};
use crate::geometry::{Point3, Triangle};
use crate::machining2d::GenError;

/// Below this squared length a direction is no direction.
const EPS: f64 = 1e-12;
/// The axial bin count cap: enough to cull a long part, small enough
/// that an empty bin costs nothing.
const MAX_BINS: usize = 4096;

/// One facet, prepared once: its corners, its unit normal (None for a
/// degenerate facet - its edges and corners still stand), and the
/// bounding sphere the per-ray cull reads.
struct Facet {
    v: [Point3; 3],
    n: Option<Point3>,
    centre: Point3,
    reach: f64,
}

/// The radial drop over one mesh, one axis and one ball radius.
pub struct RadialDrop {
    facets: Vec<Facet>,
    origin: Point3,
    dir: Point3,
    frame: Basis,
    r: f64,
    /// Axial bins: facet indices whose station span, grown by `r`,
    /// meets the bin. `s0` is bin 0's start, `w` the bin width.
    bins: Vec<Vec<usize>>,
    s0: f64,
    w: f64,
}

fn add(a: Point3, b: Point3) -> Point3 {
    Point3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}

fn scale(a: Point3, k: f64) -> Point3 {
    Point3::new(a.x * k, a.y * k, a.z * k)
}

fn unit(a: Point3) -> Option<Point3> {
    let l2 = a.dot(a);
    (l2.is_finite() && l2 > EPS).then(|| scale(a, 1.0 / l2.sqrt()))
}

impl RadialDrop {
    /// Prepares the kernel. Refuses by name a ball radius that is not
    /// a positive finite length, an empty mesh, a facet with a
    /// non-finite corner, and an axis that is no direction.
    pub fn new(facets: &[Triangle], ax: &Axis, r: f64) -> Result<Self, GenError> {
        if !(r.is_finite() && r > 0.0) {
            return Err(GenError::Invalid(format!(
                "the radial drop needs a positive finite ball radius, got {r}"
            )));
        }
        if facets.is_empty() {
            return Err(GenError::Invalid("the radial drop has no facets to rest the ball on".into()));
        }
        let (Some(dir), true) = (unit(ax.dir), ax.origin.is_finite()) else {
            return Err(GenError::Invalid(
                "the radial drop's rotation axis is not a finite direction through a finite point".into(),
            ));
        };
        let frame = basis(&Axis { origin: ax.origin, dir })?;
        let mut prepared = Vec::with_capacity(facets.len());
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for (i, t) in facets.iter().enumerate() {
            if !(t.a.is_finite() && t.b.is_finite() && t.c.is_finite()) {
                return Err(GenError::Invalid(format!(
                    "facet {i} of the radial drop's mesh has a corner that is not finite"
                )));
            }
            let v = [t.a, t.b, t.c];
            let centre = scale(add(add(t.a, t.b), t.c), 1.0 / 3.0);
            let reach = v.iter().map(|&q| (q - centre).dot(q - centre).sqrt()).fold(0.0, f64::max);
            for q in v {
                let s = (q - ax.origin).dot(dir);
                lo = lo.min(s);
                hi = hi.max(s);
            }
            prepared.push(Facet { v, n: unit((t.b - t.a).cross(t.c - t.a)), centre, reach });
        }
        let (s0, span) = (lo - r, (hi - lo) + 2.0 * r);
        let n_bins = ((span / r).ceil().max(1.0) as usize).min(MAX_BINS);
        let w = span / n_bins as f64;
        let mut bins = vec![Vec::new(); n_bins];
        for (i, f) in prepared.iter().enumerate() {
            let ss = f.v.map(|q| (q - ax.origin).dot(dir));
            let (a, b) = (ss[0].min(ss[1]).min(ss[2]) - r, ss[0].max(ss[1]).max(ss[2]) + r);
            let (ia, ib) = (bin_of(a, s0, w, n_bins), bin_of(b, s0, w, n_bins));
            for bin in &mut bins[ia..=ib] {
                bin.push(i);
            }
        }
        Ok(Self { facets: prepared, origin: ax.origin, dir, frame, r, bins, s0, w })
    }

    /// The unit radial at angle `a` (radians) about the axis.
    #[must_use]
    pub fn radial(&self, a: f64) -> Point3 {
        add(scale(self.frame.e1, a.cos()), scale(self.frame.e2, a.sin()))
    }

    /// The ball-centre radius at axial station `s` (measured along the
    /// axis from its origin) and angle `a`: the largest `t` where a
    /// ball of radius `r` centred at `origin + s·dir + t·radial(a)`
    /// touches the mesh. `None` when no facet comes within `r` of the
    /// ray's line, or when the station is not finite.
    #[must_use]
    pub fn centre_radius(&self, s: f64, a: f64) -> Option<f64> {
        if !(s.is_finite() && a.is_finite()) || s < self.s0 || s > self.s0 + self.w * self.bins.len() as f64 {
            return None;
        }
        let o = add(self.origin, scale(self.dir, s));
        let u = self.radial(a);
        let bin = &self.bins[bin_of(s, self.s0, self.w, self.bins.len())];
        let mut best: Option<f64> = None;
        for &i in bin {
            let f = &self.facets[i];
            // Cull: the facet's bounding sphere is farther than r from
            // the ray's whole line.
            let q = f.centre - o;
            let along = q.dot(u);
            let off2 = q.dot(q) - along * along;
            let lim = f.reach + self.r;
            if off2 > lim * lim {
                continue;
            }
            if let Some(t) = facet_top(f, o, u, self.r) {
                best = Some(best.map_or(t, |b| b.max(t)));
            }
        }
        best
    }
}

/// The bin of station `s`, clamped into `0..n`.
fn bin_of(s: f64, s0: f64, w: f64, n: usize) -> usize {
    let k = ((s - s0) / w).floor();
    if k.is_finite() && k > 0.0 { (k as usize).min(n - 1) } else { 0 }
}

/// The top of the interval where the ball on `o + t·u` touches facet
/// `f`: the largest valid root among the face, edge and vertex
/// patches of the facet grown by `r`. None when the line misses it.
fn facet_top(f: &Facet, o: Point3, u: Point3, r: f64) -> Option<f64> {
    let mut best = f64::NEG_INFINITY;
    // Vertices: |o + t·u - v|^2 = r^2.
    for v in f.v {
        let w = o - v;
        let b = u.dot(w);
        let disc = b * b - (w.dot(w) - r * r);
        if disc >= 0.0 {
            best = best.max(-b + disc.sqrt());
        }
    }
    // Edges: the distance to the edge's line is r, foot inside it.
    for k in 0..3 {
        let (a, b) = (f.v[k], f.v[(k + 1) % 3]);
        let e = b - a;
        let l2 = e.dot(e);
        if l2 <= EPS {
            continue; // a collapsed edge is its two corners
        }
        let w = o - a;
        let (de, we) = (u.dot(e), w.dot(e));
        let qa = 1.0 - de * de / l2;
        if qa <= EPS {
            continue; // parallel to the ray: its corners hold the top
        }
        let qb = w.dot(u) - we * de / l2;
        let qc = w.dot(w) - we * we / l2 - r * r;
        let disc = qb * qb - qa * qc;
        if disc < 0.0 {
            continue;
        }
        let t = (-qb + disc.sqrt()) / qa;
        let along = (we + t * de) / l2;
        if (0.0..=1.0).contains(&along) {
            best = best.max(t);
        }
    }
    // Face: the plane offset by r on either side, foot inside.
    if let Some(n) = f.n {
        let nd = n.dot(u);
        if nd.abs() > EPS {
            let h = n.dot(o - f.v[0]);
            for side in [r, -r] {
                let t = (side - h) / nd;
                let foot = add(add(o, scale(u, t)), scale(n, -side));
                if inside(foot, &f.v, n) {
                    best = best.max(t);
                }
            }
        }
    }
    best.is_finite().then_some(best)
}

/// Whether `p` (on the facet's plane) lies inside the triangle: the
/// three edge tests against the unit normal `n`.
fn inside(p: Point3, v: &[Point3; 3], n: Point3) -> bool {
    (0..3).all(|k| {
        let (a, b) = (v[k], v[(k + 1) % 3]);
        (b - a).cross(p - a).dot(n) >= 0.0
    })
}
