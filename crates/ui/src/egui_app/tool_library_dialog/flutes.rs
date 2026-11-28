//! #3908 (part of #3663): the helical flutes of the tool library
//! window's 3D preview. The steel meridian is revolved again with more
//! facets, and each point of the fluted length moves in toward the axis
//! where a groove passes: `flutes` grooves, twisted by the catalog helix
//! angle (`catalog.helix_deg`; straight when it is 0 or not stated).
//!
//! DISPLAY ONLY. The fluted mesh lives in this private module of
//! `crates/ui`; no collision, gouge, verify, simulation or toolpath code
//! can reach it, and they keep the conservative revolved envelope
//! (`Tool::profile`, `Tool::geom`, the holder bands). Every fluted point
//! lies on or inside that envelope: a groove only takes radius away.
//!
//! The rule (`docs/decisions/3908-tool-preview-flutes.md`):
//! - kinds: [`fluted_kind`] (#4452: every milling cutter with a cutting
//!   length, the twist drill and the tap); every other kind draws its
//!   plain revolved model;
//! - a stated count of 1..=[`MAX_FLUTES`]; no stated count, no flutes
//!   (#4452: a drill draws 2 and a tap 3 when none is stated);
//! - the groove floor is [`CORE`] of the local radius, so a groove fades
//!   to nothing at a ball tip or a drill point;
//! - each groove takes [`GROOVE`] of one flute pitch, a sine section;
//! - the grooves run out over the top [`RUN_OUT`] of the fluted length;
//! - a right-hand helix: the twist is `z * tan(helix) / R` radians
//!   (the helix lead is `pi * D / tan(helix)`).
//!
//! Bound: at most [`MAX_POINTS`] meridian points, so at most
//! [`MAX_TRIS`] facets - inside the model view's painted budget.
//! Provenance: our own display rule; the helix lead from the handbook
//! definition of a helix angle. No vendor material consulted.

use std::f64::consts::{PI, TAU};

use camcraft_app::geometry::{Point3, Triangle};
use camcraft_app::tools::{Tool, ToolKind};

/// Facets around the axis of the fluted steel.
pub(crate) const SEGS: usize = 96;
/// The groove floor as a fraction of the local radius.
pub(crate) const CORE: f64 = 0.6;
/// The groove's share of one flute pitch (the land takes the rest).
pub(crate) const GROOVE: f64 = 0.45;
/// The run-out at the flute top, a fraction of the fluted length.
pub(crate) const RUN_OUT: f64 = 0.08;
/// The most flutes the preview draws.
pub(crate) const MAX_FLUTES: u32 = 16;
/// The most meridian points; a longer meridian draws no flutes.
pub(crate) const MAX_POINTS: usize = 120;
/// The most facets of the fluted steel (two per segment and step).
pub(crate) const MAX_TRIS: usize = 2 * SEGS * (MAX_POINTS - 1);
// The bound stays inside the model view's painted budget (checked when
// the crate compiles).
const _: () = assert!(MAX_TRIS <= crate::egui_app::tool_model_view::MAX_PAINTED);
/// The rings the fluted length is cut into: at least, at most.
const RINGS: (usize, usize) = (16, 64);
/// The largest twist between two rings, radians.
const TWIST_STEP: f64 = 0.12;

/// The grooves of one tool: the count, the twist per unit of height and
/// the fluted length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Grooves {
    pub n: u32,
    pub twist: f64,
    pub len: f64,
}

impl Grooves {
    /// The grooves `t` shows, or `None` for a tool the preview draws
    /// plain (a kind without flutes, no stated count, bad numbers).
    pub(crate) fn of(t: &Tool) -> Option<Self> {
        let kind = fluted_kind(t.kind);
        let n = t.flutes.or_else(|| default_count(t.kind)).filter(|n| (1..=MAX_FLUTES).contains(n))?;
        let (len, r) = (t.flute_len, t.diameter * 0.5);
        if !(kind && len.is_finite() && len > 1e-9 && r.is_finite() && r > 1e-9) {
            return None;
        }
        let helix = t.catalog.helix_deg.or_else(|| default_helix(t.kind));
        let helix = helix.filter(|h| h.is_finite() && (0.0..=80.0).contains(h)).unwrap_or(0.0);
        Some(Self { n, twist: helix.to_radians().tan() / r, len })
    }

    /// The radius factor at angle `th` and height `z`: 1 on a land and
    /// above the fluted length, down to [`CORE`] at a groove's floor.
    pub(crate) fn scale(&self, th: f64, z: f64) -> f64 {
        if z.is_nan() || z >= self.len || !th.is_finite() {
            return 1.0;
        }
        let fade = ((self.len - z) / (RUN_OUT * self.len)).clamp(0.0, 1.0);
        let pitch = TAU / f64::from(self.n);
        let u = (th - z * self.twist).rem_euclid(pitch) / pitch;
        let s = if u < GROOVE { (PI * u / GROOVE).sin() } else { 0.0 };
        1.0 - (1.0 - CORE) * s * fade
    }

    /// The rings the fluted length is cut into.
    fn rings(&self) -> usize {
        let turn = (self.len * self.twist).abs() / TWIST_STEP;
        (turn.ceil().max(0.0) as usize).clamp(RINGS.0, RINGS.1)
    }
}

/// #4452: the kinds whose cutting part has flutes - the milling cutters
/// (end, ball and bull nose, chamfer, keyseat, T-slot, dovetail,
/// lollipop, form mill, thread mills) and the twist drill and the tap.
/// The lathe tools, the probe, the electrode and the beam and wire tools
/// draw their plain revolved model.
pub(crate) fn fluted_kind(k: ToolKind) -> bool {
    matches!(
        k,
        ToolKind::EndMill
            | ToolKind::BallNose
            | ToolKind::BullNose
            | ToolKind::Drill
            | ToolKind::Taper
            | ToolKind::Tap
            | ToolKind::ThreadMill
            | ToolKind::ThreadMillSingle
            | ToolKind::TSlot
            | ToolKind::Keyseat
            | ToolKind::Dovetail
            | ToolKind::Lollipop
            | ToolKind::FormMill
    )
}

/// #4452: the flute count a kind draws when the tool states none - a
/// twist drill has two flutes (DIN 338), a machine tap three. Display
/// only; the tool keeps no count. Other kinds draw no flutes without one.
fn default_count(k: ToolKind) -> Option<u32> {
    match k {
        ToolKind::Drill => Some(2),
        ToolKind::Tap => Some(3),
        _ => None,
    }
}

/// #4452: the helix a twist drill draws when the tool states none - the
/// 30 degrees of a DIN 338 type N drill. Other kinds draw straight.
fn default_helix(k: ToolKind) -> Option<f64> {
    (k == ToolKind::Drill).then_some(30.0)
}

/// The steel meridian `prof` (`(z, r)`, tip up) revolved with the
/// grooves of `t`, or `None` when `t` shows no flutes.
pub(crate) fn fluted(t: &Tool, prof: &[(f64, f64)]) -> Option<Vec<Triangle>> {
    let g = Grooves::of(t)?;
    let pts = resample(prof, g.len, g.rings());
    if pts.len() > MAX_POINTS {
        return None;
    }
    let tris = revolve(&pts, |th, z, r| r * g.scale(th, z));
    (tris.len() <= MAX_TRIS).then_some(tris)
}

/// `prof` with the segments of the fluted length (`0..len`) cut into
/// steps of at most `len / rings` in height. Non-finite points are left
/// out.
fn resample(prof: &[(f64, f64)], len: f64, rings: usize) -> Vec<(f64, f64)> {
    let step = len / rings.max(1) as f64;
    let pts: Vec<(f64, f64)> = prof.iter().copied().filter(|(z, r)| z.is_finite() && r.is_finite()).collect();
    let mut out = Vec::with_capacity(pts.len() + rings);
    for w in pts.windows(2) {
        let ((z0, r0), (z1, r1)) = (w[0], w[1]);
        out.push((z0, r0));
        let (lo, hi) = (z0.min(z1).max(0.0), z0.max(z1).min(len));
        if hi > lo {
            let k = ((hi - lo) / step).ceil().clamp(1.0, rings as f64) as usize;
            for i in 1..k {
                let s = i as f64 / k as f64;
                out.push((z0 + (z1 - z0) * s, r0 + (r1 - r0) * s));
            }
        }
    }
    out.extend(pts.last().copied());
    out
}

/// A `(z, r)` meridian spun about +Z in [`SEGS`] steps, each point's
/// radius taken through `at(angle, z, r)`. Rings at r = 0 close to a
/// point; zero-length segments are skipped.
fn revolve(prof: &[(f64, f64)], at: impl Fn(f64, f64, f64) -> f64) -> Vec<Triangle> {
    let p = |z: f64, r: f64, k: usize| {
        let a = TAU * k as f64 / SEGS as f64;
        let r = at(a, z, r);
        Point3::new(r * a.cos(), r * a.sin(), z)
    };
    let mut out = Vec::new();
    for w in prof.windows(2) {
        let ((z0, r0), (z1, r1)) = (w[0], w[1]);
        if (z0 - z1).abs() < 1e-12 && (r0 - r1).abs() < 1e-12 {
            continue;
        }
        for k in 0..SEGS {
            let (a, b, c, d) = (p(z0, r0, k), p(z0, r0, k + 1), p(z1, r1, k + 1), p(z1, r1, k));
            if r0 > 1e-12 {
                out.push(Triangle::new(a, b, c));
            }
            if r1 > 1e-12 {
                out.push(Triangle::new(a, c, d));
            }
        }
    }
    out
}

/// #4336 (U10): the most samples along one flute of the side view.
pub(crate) const SIDE_STEPS: usize = 96;

/// #4336 (U10): the flutes of the side view, display only - the centre
/// line of each groove where it faces the viewer, as polylines of
/// `(z, x)` in model units (x across the axis). A groove at angle `th`
/// shows at `x = r(z) * sin(th)` while `cos(th) > 0`; the helix turns `th`
/// by `twist` per unit of height, so a helix draws as the slanted curves
/// of a technical drawing and a straight flute as a vertical line. The
/// lines stop where the run-out starts. `None` when `t` shows no flutes.
/// Built with the model (when the tool changes), never on a still frame.
pub(crate) fn side_lines(t: &Tool, prof: &[(f64, f64)]) -> Option<Vec<Vec<(f64, f64)>>> {
    let g = Grooves::of(t)?;
    let top = g.len * (1.0 - RUN_OUT);
    let pitch = TAU / f64::from(g.n);
    let mut out = Vec::new();
    for k in 0..g.n {
        let phase = pitch * (f64::from(k) + GROOVE * 0.5);
        let mut line: Vec<(f64, f64)> = Vec::new();
        for i in 0..=SIDE_STEPS {
            let z = top * i as f64 / SIDE_STEPS as f64;
            let th = z * g.twist + phase;
            if th.cos() > 0.0 {
                line.push((z, radius_at(prof, z).unwrap_or(t.diameter * 0.5) * th.sin()));
            } else if line.len() > 1 {
                out.push(std::mem::take(&mut line));
            } else {
                line.clear();
            }
        }
        if line.len() > 1 {
            out.push(line);
        }
    }
    Some(out)
}

/// The radius of the meridian `prof` (`(z, r)`, tip up) at height `z`.
fn radius_at(prof: &[(f64, f64)], z: f64) -> Option<f64> {
    prof.windows(2).find_map(|w| {
        let ((z0, r0), (z1, r1)) = (w[0], w[1]);
        let (lo, hi) = (z0.min(z1), z0.max(z1));
        if !(lo <= z && z <= hi) || !(r0.is_finite() && r1.is_finite()) {
            return None;
        }
        let s = if hi - lo > 1e-12 { (z - z0) / (z1 - z0) } else { 1.0 };
        Some(r0 + (r1 - r0) * s)
    })
}

#[cfg(test)]
#[path = "flutes_tests.rs"]
mod tests;
