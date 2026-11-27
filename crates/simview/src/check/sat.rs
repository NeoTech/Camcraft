//! Intersection of two closed convex bodies given as triangles (#2586):
//! the bodies of the generated machine model are blocks and cylinders,
//! each convex. Two convex polyhedra are apart if and only if a face
//! normal of one or a cross product of two edge directions separates
//! them (the separating axis theorem).
//! Provenance: Gottschalk, Lin, Manocha, "OBBTree", SIGGRAPH 1996, and
//! Ericson, "Real-Time Collision Detection", 2005, ch. 4-5. Our own code.

use camcraft_sim::geometry::Triangle;

type V = [f64; 3];

fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: V, b: V) -> V {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn unit(a: V) -> Option<V> {
    let n = dot(a, a).sqrt();
    (n > 1e-12 && n.is_finite()).then(|| [a[0] / n, a[1] / n, a[2] / n])
}

/// A convex body: its corners, its face normals and its edge directions.
pub struct Hull {
    pub lo: V,
    pub hi: V,
    verts: Vec<V>,
    normals: Vec<V>,
    edges: Vec<V>,
}

/// Add `d` to `set` unless a parallel direction is in it.
fn push_dir(set: &mut Vec<V>, d: V) {
    if !set.iter().any(|s| dot(cross(*s, d), cross(*s, d)) < 1e-12) {
        set.push(d);
    }
}

impl Hull {
    /// The hull of `tris`. `None` for no triangle or a non-finite one.
    #[must_use]
    pub fn new(tris: &[Triangle]) -> Option<Self> {
        let mut h = Hull { lo: [f64::INFINITY; 3], hi: [f64::NEG_INFINITY; 3], verts: Vec::new(), normals: Vec::new(), edges: Vec::new() };
        for t in tris {
            let p = [[t.a.x, t.a.y, t.a.z], [t.b.x, t.b.y, t.b.z], [t.c.x, t.c.y, t.c.z]];
            if p.iter().flatten().any(|v| !v.is_finite()) {
                return None;
            }
            for v in p {
                for (k, c) in v.iter().enumerate() {
                    h.lo[k] = h.lo[k].min(*c);
                    h.hi[k] = h.hi[k].max(*c);
                }
                h.verts.push(v);
            }
            if let Some(n) = unit(cross(sub(p[1], p[0]), sub(p[2], p[0]))) {
                push_dir(&mut h.normals, n);
            }
            for (a, b) in [(0, 1), (1, 2), (2, 0)] {
                if let Some(e) = unit(sub(p[b], p[a])) {
                    push_dir(&mut h.edges, e);
                }
            }
        }
        (!h.verts.is_empty()).then_some(h)
    }

    fn range(&self, axis: V) -> (f64, f64) {
        self.verts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
            let d = dot(*v, axis);
            (lo.min(d), hi.max(d))
        })
    }
}

/// Whether the bounding boxes of `a` and `b` overlap by more than `tol`.
#[must_use]
pub fn boxes_overlap(a: &Hull, b: &Hull, tol: f64) -> bool {
    (0..3).all(|k| a.lo[k] < b.hi[k] - tol && b.lo[k] < a.hi[k] - tol)
}

/// Whether `a` and `b` overlap by more than `tol` along every axis: a
/// touch (faces in contact) is not an overlap.
#[must_use]
pub fn overlap(a: &Hull, b: &Hull, tol: f64) -> bool {
    if !boxes_overlap(a, b, tol) {
        return false;
    }
    let apart = |axis: V| {
        let (alo, ahi) = a.range(axis);
        let (blo, bhi) = b.range(axis);
        ahi <= blo + tol || bhi <= alo + tol
    };
    if a.normals.iter().chain(&b.normals).any(|n| apart(*n)) {
        return false;
    }
    for ea in &a.edges {
        for eb in &b.edges {
            if let Some(axis) = unit(cross(*ea, *eb))
                && apart(axis)
            {
                return false;
            }
        }
    }
    true
}
