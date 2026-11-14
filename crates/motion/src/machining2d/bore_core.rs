//! The Bore ride's core law (#1556): how wide a hole the centre dive and
//! one ring clear at the floor, for a flat, bull-nose or ball cutter.
//!
//! Provenance: plain torus geometry of a corner-radiused end mill (a
//! cutter of radius `r` whose corner of radius `cr` rounds into a flat
//! bottom of radius `r - cr`), as machine-tool handbooks draw it; the
//! ride's bands are those of `bore.rs`. Our own derivation, written
//! down in `docs/decisions/1556-bore-core-door.md`.

/// How far the ring's inner edge may stand past the dive's disc before
/// the gap counts as material (a float-noise allowance, not a cusp
/// budget: at the limit the two bands meet edge to edge).
pub(super) const CORE_TOL: f64 = 1e-6;

/// The radius a cutter cuts at `height` above its tip: the flat bottom
/// (`r - cr`) plus the corner's reach at that height, `sqrt(cr^2 -
/// (cr - h)^2) = sqrt(h (2 cr - h))`; the full radius `r` from `cr` up.
/// `corner_radius` is clamped to `[0, r]` (0 a square end mill, `r` a
/// ball); a negative height reads as the tip.
#[must_use]
pub fn cutting_radius_at(tool_radius: f64, corner_radius: f64, height: f64) -> f64 {
    let r = tool_radius.max(0.0);
    let cr = corner_radius.clamp(0.0, r);
    if height >= cr {
        return r;
    }
    let h = height.max(0.0);
    (r - cr) + (h * (2.0 * cr - h)).max(0.0).sqrt()
}

/// How far the cutter reaches at the plane the core would stand on.
/// A through hole (`through_allowance = Some(a)`, the tip `a` under the
/// floor) is judged at the floor plane, where the cutter cuts at
/// height `a`: a core left there has no floor under it and comes loose.
/// A blind hole (`None`) keeps the full radius: its floor stays under
/// the corner, so what the corner leaves is a ridge standing on it
/// (the usual bull-nose cusp), never a free piece.
#[must_use]
pub fn bore_floor_reach(tool_diameter: f64, corner_radius: f64, through_allowance: Option<f64>) -> f64 {
    let r = tool_diameter * 0.5;
    match through_allowance {
        Some(a) => cutting_radius_at(r, corner_radius, a),
        None => r.max(0.0),
    }
}

/// The core the ride leaves across a hole of `hole_radius`, as the gap
/// between the dive's disc (radius `reach`) and the ring's inner edge
/// (`rc - reach`, `rc = R - r - leave`): `rc - 2 reach`. Positive is
/// material no pass touches.
#[must_use]
pub fn bore_core_gap(hole_radius: f64, tool_diameter: f64, corner_radius: f64, leave: f64, through_allowance: Option<f64>) -> f64 {
    let rc = hole_radius - (tool_diameter * 0.5 + leave);
    rc - 2.0 * bore_floor_reach(tool_diameter, corner_radius, through_allowance)
}

/// The one verdict the kernel's door and Detect holes both read: does
/// the ride leave a core in this hole? Non-finite input reads as no -
/// the kernel's finite door refuses it by name before this is asked.
#[must_use]
pub fn bore_leaves_core(hole_radius: f64, tool_diameter: f64, corner_radius: f64, leave: f64, through_allowance: Option<f64>) -> bool {
    bore_core_gap(hole_radius, tool_diameter, corner_radius, leave, through_allowance) > CORE_TOL
}

/// The widest hole the ride clears without a core: the dive and the
/// ring meet when `rc = 2 reach`, i.e. `D = d + 4 reach + 2 leave` -
/// `3d + 2 leave` for a square end mill (and any blind hole).
#[must_use]
pub fn max_bore_diameter(tool_diameter: f64, corner_radius: f64, stock_to_leave: f64, through_allowance: Option<f64>) -> f64 {
    tool_diameter + 4.0 * bore_floor_reach(tool_diameter, corner_radius, through_allowance) + 2.0 * stock_to_leave
}
