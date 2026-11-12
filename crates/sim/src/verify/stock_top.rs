//! Issue #1091: law (d)'s vertical dives re-read against the stock's top under the tool, not the stock's highest point - our own exact upper bound on a closed mesh's column (the topmost surface over a point lies on a facet whose plan covers it), specced from the shop rule "never rapid into uncut stock"; no vendor material consulted.
//!
//! The column law caps every column at `VerifyContext::stock_top_z`, the
//! highest point of the whole stock: on a stepped stock (a casting, a
//! carried body with a boss) a dive to just above the low part is
//! convicted as if the boss stood there too. [`acquit_by_stock_top`]
//! re-reads such a dive the way [`super::acquit_by_prior`] re-reads one
//! against the program ahead: it only removes a conviction, and only
//! where the solids prove the whole tool stack ends above all material.
//!
//! The bound: for a closed mesh, the material over any plan point p
//! stands no higher than the highest vertex of a facet whose plan
//! (triangle, or the segment of a vertical facet) comes within the disc
//! around the dive - the topmost surface over p is on a facet covering p.
//! A facet that only grazes the disc still lends its highest vertex, so
//! a slope reads high (an over-report, never a hide). Each band of the
//! stack is judged at its own disc and its own height over the tip: the
//! flute's radius at the tip, the shank's from the flute's top, the
//! holder's from the shank's top (and on up - the cap is a height, so a
//! band that is taller changes nothing). Fails closed: a solid that is
//! not one closed, outward shell ([`super::stock_top_shell`] - an open
//! shell, a non-manifold edge, a non-finite vertex, no facets), a band
//! whose radius or offset is not a finite non-negative number, or a
//! flute disc that meets no facet at all leaves the conviction standing.

use super::laws::dist2_seg;
use super::stock_top_shell::closed_shell;
use super::types::{Mode, Warning};

use crate::cl::{MoveKind, Toolpath};
use crate::geo2d::Point2;
use crate::geometry::Triangle;
use crate::tools::ToolGeom;

/// `warns` without the convictions of upright vertical rapids DOWN whose
/// whole tool stack ends above every facet of `solids` within reach
/// (see the module doc). `solids` are the closed meshes that bound the
/// material the op can meet - the stock and the part - in the path's
/// own frame. Lateral rapids, posed dives, and every other warning keep
/// their verdict; so does every dive of a mode whose rapids are not
/// judged by the vertical column (radial, revolved, rotary).
pub fn acquit_by_stock_top(
    path: &Toolpath,
    mode: Mode,
    solids: &[&[Triangle]],
    tool: &ToolGeom,
    z_tol: f64,
    warns: &mut Vec<Warning>,
) {
    if matches!(mode, Mode::Radial | Mode::Revolve | Mode::Rotary)
        || solids.is_empty()
        || !z_tol.is_finite()
        || !solids.iter().all(|s| closed_shell(s))
    {
        return;
    }
    let Some(bands) = bands(tool) else { return };
    warns.retain(|w| {
        let Warning::RapidInStock { at } = *w else { return true };
        let Some(m) = path.moves.get(at) else { return true };
        let (dx, dy) = (m.to.x - m.from.x, m.to.y - m.from.y);
        let upright = m.axis.is_none_or(|a| a[0].abs() <= 1e-12 && a[1].abs() <= 1e-12 && a[2] > 0.0);
        let vertical_down =
            m.kind == MoveKind::Rapid && upright && dx * dx + dy * dy <= z_tol * z_tol && m.to.z < m.from.z;
        if !vertical_down || !m.to.z.is_finite() {
            return true;
        }
        let c = Point2::new(m.to.x, m.to.y);
        !clear_of(solids, c, m.to.z, &bands, z_tol)
    });
}

/// The stack's bands as `(radius, height of its bottom over the tip)`;
/// `None` when any number is not a finite non-negative value.
fn bands(t: &ToolGeom) -> Option<[(f64, f64); 3]> {
    let shank_at = t.flute_len;
    let holder_at = t.flute_len + t.shank_len;
    let b = [(t.radius, 0.0), (t.shank_radius, shank_at), (t.holder_radius, holder_at)];
    b.iter().all(|&(r, h)| r.is_finite() && r >= 0.0 && h.is_finite() && h >= 0.0).then_some(b)
}

/// Whether every band of the stack, its tip at (c, `tip`), stands above
/// the highest facet within its disc (less `z_tol`). The flute's disc
/// must meet some facet - an empty reading is no proof.
fn clear_of(solids: &[&[Triangle]], c: Point2, tip: f64, bands: &[(f64, f64); 3], z_tol: f64) -> bool {
    bands.iter().enumerate().all(|(k, &(r, h))| match top_within(solids, c, r) {
        Some(top) => tip + h >= top - z_tol,
        None => k > 0, // nothing under a stack band is air; under the flute it is unknown
    })
}

/// The highest vertex of any facet whose plan comes within `r` of `c`.
fn top_within(solids: &[&[Triangle]], c: Point2, r: f64) -> Option<f64> {
    let r2 = r * r;
    let mut top: Option<f64> = None;
    for t in solids.iter().flat_map(|s| s.iter()) {
        if dist2_tri(c, t) <= r2 {
            let z = t.a.z.max(t.b.z).max(t.c.z);
            top = Some(top.map_or(z, |m| m.max(z)));
        }
    }
    top
}

/// The squared plan distance from `p` to triangle `t` (0 inside it). A
/// triangle whose plan is degenerate is read as its three edges.
fn dist2_tri(p: Point2, t: &Triangle) -> f64 {
    let (a, b, c) = (Point2::new(t.a.x, t.a.y), Point2::new(t.b.x, t.b.y), Point2::new(t.c.x, t.c.y));
    let cross = |u: Point2, v: Point2| (v.x - u.x) * (p.y - u.y) - (v.y - u.y) * (p.x - u.x);
    let (d0, d1, d2) = (cross(a, b), cross(b, c), cross(c, a));
    let neg = d0 < 0.0 || d1 < 0.0 || d2 < 0.0;
    let pos = d0 > 0.0 || d1 > 0.0 || d2 > 0.0;
    let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    if area != 0.0 && !(neg && pos) {
        return 0.0;
    }
    dist2_seg(p, a, b).min(dist2_seg(p, b, c)).min(dist2_seg(p, c, a))
}


#[cfg(test)]
#[path = "stock_top_tests.rs"]
mod tests;
