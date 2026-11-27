//! The SPLINE and ELLIPSE definitions of a block, placed exactly by an INSERT and fitted after
//! the placement, in world units (#1865).
//! Provenance: the affine image of an ellipse `c + M cos t + m sin t` is `A c + b + (A M) cos t +
//! (A m) sin t`; a rational B-spline is invariant under affine maps (L. Piegl and W. Tiller,
//! "The NURBS Book", section 4.4); our own implementation.
//!
//! A block body is read once and placed many times. A fit made before the placement would be
//! scaled by it, and its error with it. Thus the body keeps the curve itself, and each placed
//! copy is fitted within the curve tolerance of the drawing.
use super::curves::spline_arcs;
use super::insert_place::{Affine, conic};
use super::nurbs::Nurbs;
use super::types::{DxfError, Primitive, State, error};
use crate::geo2d::Point2;

/// A curve of a block body, in block coordinates.
pub(super) enum CurveDef {
    /// `c + major cos t + minor sin t` for t in `[start, start + sweep]`, sweep > 0.
    Ellipse {
        c: Point2,
        major: (f64, f64),
        minor: (f64, f64),
        start: f64,
        sweep: f64,
        full: bool,
    },
    Spline(Nurbs),
}

/// A block curve on its layer. `at` is the count of block primitives read before it, which
/// keeps the drawing order when the body is placed.
pub(super) struct LayeredCurve {
    pub layer: String,
    pub curve: CurveDef,
    pub at: usize,
}

/// Keep `curve` of a block body for its placement. One vertex is counted, as for a circle.
pub(super) fn defer(state: &mut State, layer: &str, curve: CurveDef, kind: &str) -> Result<(), DxfError> {
    state.vertex(kind)?;
    let at = state.doc.primitives.len();
    state.curves.push(LayeredCurve { layer: layer.into(), curve, at });
    Ok(())
}

/// `curve` placed by `m` and laid as tangent arcs in world units.
pub(super) fn place_curve(state: &mut State, m: &Affine, curve: &CurveDef, context: &str) -> Result<Vec<Primitive>, DxfError> {
    match curve {
        CurveDef::Ellipse { c, major, minor, start, sweep, full } => {
            let center = m.point(*c, context)?;
            conic(state, center, (m.linear(major.0, major.1), m.linear(minor.0, minor.1)), (*start, *sweep, *full), context)
        }
        CurveDef::Spline(curve) => {
            let placed = curve.mapped(|p| m.point(p, context))?;
            let out = spline_arcs(state, &placed).map_err(|e| DxfError(format!("{context}: {}", e.0)))?;
            if out.is_empty() {
                return Err(error(context, "placed curve has zero length"));
            }
            Ok(out)
        }
    }
}
