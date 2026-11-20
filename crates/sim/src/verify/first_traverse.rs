//! #2901: the first traverse of a milling operation, from an X/Y the machine does not know after a tool change, must ride above the whole start stock and every fixture - our own reading of the shop rule "never traverse through uncut stock or steel" (machine-tool handbooks), no vendor material consulted.
//!
//! After a tool change the control does not know where the tool stands
//! in X/Y. The post sends Z to the plane the path starts on, then X/Y
//! at that plane: that one traverse can cross any point of the setup.
//! No line of it is known, so no swept law can judge it. The law here
//! judges the plane instead: the path's start height must stand at or
//! over the top of the start stock (the full block, never the
//! in-process stock, because the line is not known) and the top of each
//! fixture, all read in the path's own frame (a 3+2 plane reads heights
//! along its tool axis). A start under either top is one blocking
//! verdict at move 0 that names both heights.

use super::types::Warning;
use crate::cl::Toolpath;
use crate::fixture::FixtureBox;
use crate::geometry::Point3;

/// The highest point of `b` along the frame's height `z_of` (the
/// largest of its eight corners: an affine map takes its max there).
#[must_use]
pub fn box_top(b: &FixtureBox, z_of: impl Fn(Point3) -> f64) -> f64 {
    (0..8)
        .map(|i| {
            z_of(Point3::new(
                [b.min.x, b.max.x][(i >> 2) & 1],
                [b.min.y, b.max.y][(i >> 1) & 1],
                [b.min.z, b.max.z][i & 1],
            ))
        })
        .fold(f64::NEG_INFINITY, f64::max)
}

/// The first-traverse law: `stock_top` is the top of the start stock in
/// the path's frame (`-inf` when there is none), `steel` the fixtures'
/// boxes in the world, `z_of` the world-to-frame height (`|p| p.z` for
/// an upright path). `None` when the path's start stands at or over
/// both tops within `z_tol`, or when the path is empty or its start is
/// not finite (the non-finite law names that).
#[must_use]
pub fn first_traverse_low(
    path: &Toolpath,
    stock_top: f64,
    steel: &[FixtureBox],
    z_of: impl Fn(Point3) -> f64,
    z_tol: f64,
) -> Option<Warning> {
    let start = path.start()?;
    if !start.z.is_finite() {
        return None;
    }
    let fixture_top = steel.iter().map(|b| box_top(b, &z_of)).filter(|z| z.is_finite()).fold(f64::NEG_INFINITY, f64::max);
    let stock_top = if stock_top.is_finite() { stock_top } else { f64::NEG_INFINITY };
    let (top, fixture) = if fixture_top > stock_top { (fixture_top, true) } else { (stock_top, false) };
    (top.is_finite() && start.z < top - z_tol).then_some(Warning::FirstTraverseLow { at: 0, height_mm: start.z, top_mm: top, fixture })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cl::{MoveKind, PathBuilder};

    fn opening(z: f64) -> Toolpath {
        let mut b = PathBuilder::new(Point3::new(5.0, 5.0, z));
        b.push(MoveKind::Rapid, Point3::new(5.0, 5.0, 1.0), None);
        b.push(MoveKind::Plunge, Point3::new(5.0, 5.0, -1.0), Some(100.0));
        b.finish()
    }

    fn clamp(top: f64) -> FixtureBox {
        FixtureBox { min: Point3::new(200.0, 200.0, 0.0), max: Point3::new(210.0, 210.0, top) }
    }

    #[test]
    fn a_start_under_the_stock_top_is_named_with_both_heights() {
        let w = first_traverse_low(&opening(8.0), 10.0, &[], |p| p.z, 1e-6);
        assert_eq!(w, Some(Warning::FirstTraverseLow { at: 0, height_mm: 8.0, top_mm: 10.0, fixture: false }));
        assert!(w.is_some_and(|w| w.blocks_post()));
    }

    #[test]
    fn a_far_fixture_taller_than_the_start_is_named() {
        let w = first_traverse_low(&opening(15.0), 10.0, &[clamp(12.0), clamp(20.0)], |p| p.z, 1e-6);
        assert_eq!(w, Some(Warning::FirstTraverseLow { at: 0, height_mm: 15.0, top_mm: 20.0, fixture: true }));
    }

    #[test]
    fn a_start_at_or_over_every_top_passes() {
        assert_eq!(first_traverse_low(&opening(25.0), 10.0, &[clamp(20.0)], |p| p.z, 1e-6), None);
        assert_eq!(first_traverse_low(&opening(10.0), 10.0, &[clamp(10.0)], |p| p.z, 1e-6), None);
        assert_eq!(first_traverse_low(&opening(5.0), f64::NEG_INFINITY, &[], |p| p.z, 1e-6), None);
        assert_eq!(first_traverse_low(&Toolpath::new(Vec::new()), 10.0, &[], |p| p.z, 1e-6), None);
        assert_eq!(first_traverse_low(&opening(f64::NAN), 10.0, &[], |p| p.z, 1e-6), None);
    }

    #[test]
    fn a_tilted_frame_reads_the_highest_corner_along_its_axis() {
        // A frame whose height runs along world +X: the clamp's far
        // face (x 210) is its top there, whatever its Z.
        let along_x = |p: Point3| p.x;
        assert_eq!(box_top(&clamp(1.0), along_x), 210.0);
        let w = first_traverse_low(&opening(150.0), 100.0, &[clamp(1.0)], along_x, 1e-6);
        assert_eq!(w, Some(Warning::FirstTraverseLow { at: 0, height_mm: 150.0, top_mm: 210.0, fixture: true }));
    }
}
