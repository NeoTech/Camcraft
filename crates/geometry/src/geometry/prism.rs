//! Recognize a solid as the extrusion of a convex polygon, using its supporting
//! half-planes and the mesh's closed-shell topology (elementary solid geometry).

use super::{Triangle, slice_bodies};
use crate::geo2d::{Loop, Point2, SCALE};

/// True only for a single, consistently wound vertical prism bounded by the
/// stated polygon and caps. Examines every facet, including internal surfaces
/// that an XY projection or a finite number of horizontal slices can hide.
pub fn is_closed_convex_prism(facets: &[Triangle], ring: &Loop, bottom: f64, top: f64) -> bool {
    let corners = ring.points();
    // Bound the half-plane tests before expanding the shell topology.
    if facets.is_empty()
        || corners.len() < 3
        || !bottom.is_finite()
        || !top.is_finite()
        || bottom >= top
        || facets.len().saturating_mul(corners.len()) > 8_000_000
    {
        return false;
    }
    let area = ring.area();
    if !area.is_finite() || area == 0.0 {
        return false;
    }
    let sense = area.signum();
    // Region vertices have been rounded to the XY clipping lattice.
    let tol = 1.0 / SCALE;
    let mut sides = Vec::with_capacity(corners.len());
    for (&a, &b) in corners.iter().zip(corners.iter().cycle().skip(1)) {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let length = dx.hypot(dy);
        if !length.is_finite() || length == 0.0 {
            return false;
        }
        let inward = Point2::new(-sense * dy / length, sense * dx / length);
        sides.push((a, inward));
    }
    for facet in facets {
        let points = [facet.a, facet.b, facet.c];
        if points
            .iter()
            .any(|p| !p.is_finite() || p.z < bottom || p.z > top)
        {
            return false;
        }
        let normal = facet.normal();
        let bottom_cap = points.iter().all(|p| p.z == bottom) && normal.z < 0.0;
        let top_cap = points.iter().all(|p| p.z == top) && normal.z > 0.0;
        let mut boundary_wall = false;
        for (a, inward) in &sides {
            let distances = points.map(|p| (p.x - a.x) * inward.x + (p.y - a.y) * inward.y);
            // Every vertex of the retained solid must be on the inside of
            // every side: a concavity or extra material fails this test.
            if distances.iter().any(|&d| !d.is_finite() || d < -tol) {
                return false;
            }
            if distances.iter().all(|d| d.abs() <= tol)
                && normal.x * inward.x + normal.y * inward.y < 0.0
                && normal.z.abs() < 1e-12
            {
                boundary_wall = true;
            }
        }
        if !(bottom_cap || top_cap || boundary_wall) {
            return false;
        }
    }
    slice_bodies::closed_solids(facets).is_some_and(|solids| solids.len() == 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Point3, cube};

    #[test]
    fn an_outer_projection_does_not_prove_the_solid_is_a_prism() {
        let min = Point3::new(-20.0, -10.0, 0.0);
        let max = Point3::new(20.0, 10.0, 20.0);
        let block = cube(min, max);
        let ring = Loop::new(vec![
            Point2::new(min.x, min.y),
            Point2::new(max.x, min.y),
            Point2::new(max.x, max.y),
            Point2::new(min.x, max.y),
        ])
        .unwrap();
        assert!(is_closed_convex_prism(&block.facets, &ring, min.z, max.z));
        let mut shells = block.facets.to_vec();
        let cavity = cube(Point3::new(-1.0, -1.0, 4.0), Point3::new(1.0, 1.0, 6.0));
        shells.extend(cavity.facets.iter().map(|t| Triangle::new(t.a, t.c, t.b)));
        assert!(!is_closed_convex_prism(&shells, &ring, min.z, max.z));
        assert!(!is_closed_convex_prism(
            &block.facets[1..],
            &ring,
            min.z,
            max.z
        ));
    }
}
