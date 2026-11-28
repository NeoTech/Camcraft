//! The rotary kinds' radial heights in the viewport (#2774): on a turned
//! part a height is a RADIUS about the fitted rotation axis, so the
//! viewport draws it as a RING standing on that axis - the same line the
//! generators pose about - and never as a rectangle at a world Z.
//!
//! The ring set follows the open operation's DRAFT parameters, like the
//! height planes beside it: spinning a radius walks its ring before Apply
//! commits it. A radial height whose source is not there (a stock radius
//! with no stock) draws nothing: the panel row says why, and the gate
//! refuses by name - the viewport never draws a circle at a guessed radius.
//!
//! Provenance: our own display of the domain model's radial heights; a
//! circle about a line is elementary geometry. No vendor material consulted.

use camcraft_app::geometry::Point3;
use camcraft_app::operation::{self, RadialHeight};

use crate::egui_app::ground_ink::{Overlay, overlay};
use crate::height_memo;
use crate::interact::{self, CLEARANCE_RGB, RETRACT_RGB};
use crate::state::State;

/// The four radial slots' names, in the order `radial_fields` returns them.
const NAMES: [&str; 4] = ["Clearance", "Retract", "Top", "Bottom"];

/// The name each radial slot carries in the viewport and the panel row.
/// A kind that stores no slot for one of them never draws or edits it.
pub(crate) fn name_of(slot: usize) -> &'static str {
    NAMES[slot.min(NAMES.len() - 1)]
}

/// The colour the ring wears: the two rapid levels keep the palette of the
/// Z planes they stand in for, the two cut levels the region's and the
/// cut's.
fn rgb_of(slot: usize) -> [f32; 3] {
    match slot {
        0 => CLEARANCE_RGB,
        1 => RETRACT_RGB,
        2 => overlay(Overlay::Region),
        _ => overlay(Overlay::Cut),
    }
}

/// The open operation's radial slots, or `None` for a kind that has none.
/// A slot that is `None` keeps the Z-plane law, and its plane draws.
pub(crate) fn draft(state: &State) -> Option<[Option<RadialHeight>; 4]> {
    operation::radial_fields(&state.op_ui.params)
}

/// One radial height of the open draft, resolved to a radius and standing
/// on the axis it is measured about.
pub(crate) struct RadialMark {
    pub rgb: [f32; 3],
    /// The radius, in millimetres, about the axis.
    pub radius: f64,
    /// A point on the axis the ring stands on, in world coordinates.
    pub on_axis: [f64; 3],
    /// The axis' unit direction.
    pub dir: [f64; 3],
}

/// A unit vector standing off `d`. The shortest component of `d` picks the
/// helper, so any direction gets a perpendicular, never a degenerate one.
fn off_axis(d: [f64; 3]) -> [f64; 3] {
    let helper = if d[0].abs() <= d[1].abs() && d[0].abs() <= d[2].abs() {
        [1.0, 0.0, 0.0]
    } else if d[1].abs() <= d[2].abs() {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let u = [
        helper[1] * d[2] - helper[2] * d[1],
        helper[2] * d[0] - helper[0] * d[2],
        helper[0] * d[1] - helper[1] * d[0],
    ];
    let len = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    if !len.is_finite() || len < 1e-12 {
        return [0.0; 3];
    }
    [u[0] / len, u[1] / len, u[2] / len]
}

/// The ring's orthonormal basis in the plane standing square to `dir`:
/// `u` a unit perpendicular of `dir`, `v` the unit `dir × u` - already the
/// second perpendicular, NEVER passed back through `off_axis` (which picks
/// another helper and turns `v` into a direction the ring's plane does not
/// contain: an X axis rang its points along a line through ±sqrt(2)·r, a
/// Z axis laid its second vector along the axis itself).
fn ring_basis(dir: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let u = off_axis(dir);
    let v = [
        dir[1] * u[2] - dir[2] * u[1],
        dir[2] * u[0] - dir[0] * u[2],
        dir[0] * u[1] - dir[1] * u[0],
    ];
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let v = if len.is_finite() && len > 1e-12 { [v[0] / len, v[1] / len, v[2] / len] } else { [0.0; 3] };
    (u, v)
}

/// One ring's chords: `SEGMENTS` points on the circle of `radius` about
/// `on_axis`, in the plane standing square to `dir` - the orthonormal
/// basis of [`ring_basis`], so every point stands IN the plane at the
/// stated radius, on any axis.
fn ring_points(on_axis: [f64; 3], dir: [f64; 3], radius: f64) -> Vec<[f64; 3]> {
    let (u, v) = ring_basis(dir);
    let step = std::f64::consts::TAU / SEGMENTS as f64;
    (0..SEGMENTS)
        .map(|i| {
            let (s, c) = (step * i as f64).sin_cos();
            [
                on_axis[0] + (u[0] * c + v[0] * s) * radius,
                on_axis[1] + (u[1] * c + v[1] * s) * radius,
                on_axis[2] + (u[2] * c + v[2] * s) * radius,
            ]
        })
        .collect()
}

/// The open operation's radial heights, resolved against the source's and
/// the setup stock's radii about the judged axis - the SAME axis generation
/// turns about (a stated axis first, the free fit otherwise, #2772).
/// Nothing to resolve, no fittable axis, or a height whose source is
/// missing: that ring is not drawn - the panel names the reason and the
/// gate refuses.
pub(crate) fn marks(state: &State) -> Vec<RadialMark> {
    let Some(fields) = draft(state) else {
        return Vec::new();
    };
    if !fields.iter().any(Option::is_some) {
        return Vec::new();
    }
    let setup = interact::shown_setup(state);
    let src = state.op_ui.src();
    let Some(view) =
        height_memo::radial_axis(&state.world, setup.as_deref(), &src, camcraft_app::rotary_stated::op_stated(&state.op_ui.params))
    else {
        return Vec::new();
    };
    let d = view.dir;
    let mut out = Vec::new();
    for (slot, h) in fields.into_iter().enumerate() {
        let Some(h) = h else { continue };
        let Ok(radius) = operation::resolve_one(name_of(slot), h, view.part_rho, view.stock_rho) else {
            continue;
        };
        if !radius.is_finite() || radius <= 0.0 {
            continue;
        }
        out.push(RadialMark {
            rgb: rgb_of(slot),
            radius,
            on_axis: view.origin,
            dir: d,
        });
    }
    out
}

/// Chords per ring: enough for the circle to read round at any zoom this
/// overlay reaches, few enough to stay far under the overlay cap.
const SEGMENTS: usize = 32;

/// The rings as overlay geometry: one circle each, standing on the judged
/// axis at its radius, in world coordinates.
pub(crate) fn ring_lines(state: &State) -> Vec<[f32; 6]> {
    let marks = marks(state);
    if marks.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for m in &marks {
        let points = ring_points(m.on_axis, m.dir, m.radius);
        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            interact::push_line(
                &mut out,
                Point3::new(a[0], a[1], a[2]),
                Point3::new(b[0], b[1], b[2]),
                m.rgb,
            );
        }
    }
    out
}

#[cfg(test)]
#[path = "radial_view_tests.rs"]
mod tests;
