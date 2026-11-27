//! The machine-frame view of the sim playback (#4266): what a
//! work-turning rotary's indexed pose draws. A rotary that turns the
//! WORK (`turns_part`, the machine editor's flag) swings the part, the
//! stock and the steel about the stated rotary axis line while the
//! tool stays upright - the kinematics the gate judges the posed steel
//! with, so the picture trains the same eye the verdict answers to.
//!
//! The turn that stands a pose upright about the line is the gate's
//! own reading (`verify::posed_steel::upright_turn`, `turn_about`):
//! mirrored here, not imported, because those helpers are private to
//! the verify module and the display must not reach into it. Same
//! Rodrigues turn, same convention: the signed turn about the line
//! carrying the pose onto machine +Z is unique for a pose in the
//! plane normal to a level line, whatever the controller's sign.
//!
//! A tool-turning rotary (a head) has no gauge-line pivot stated in
//! the machine file, so the playback keeps today's tip-relative draw
//! for it - the fallback, noted in #4266's PR rather than guessed.
//!
//! Display only: nothing here writes the world or feeds a verdict.
//! A degenerate line, a non-finite number or a pose that cannot stand
//! upright (a leaning line, a pose out of the turn's plane) reads as
//! `None`: the fallback draw, never a wrong picture.
//!
//! Provenance: our own reading of machine-tool kinematics (a table
//! rotary turns the work about its centreline; the programmed pose
//! rides the rotating frame) and our own gate math; no vendor
//! material consulted.

use crate::sim::WorkTurn;

/// A pose out of the turn's plane, or a direction this level off the
/// horizontal: the rotary alone cannot stand it upright - the same
/// tolerance the gate reads the plane with.
const PLANE_TOL: f64 = 1e-6;

/// The unit direction of `dir`, or `None` when it is no direction.
fn unit(dir: [f64; 3]) -> Option<[f64; 3]> {
    let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    (l.is_finite() && l > 1e-12).then_some([dir[0] / l, dir[1] / l, dir[2] / l])
}

/// The signed turn about the line's direction carrying the unit pose
/// `axis` onto machine +Z (`None` when the line leans, is degenerate,
/// or the pose will not stand upright under it: a pose with a part
/// along the line, or a non-horizontal line). The turn any controller
/// makes - unique for a pose lying in the plane normal to a level
/// line, the gate's own reading.
#[must_use]
pub fn upright_turn(dir: [f64; 3], axis: [f64; 3]) -> Option<f64> {
    let d = unit(dir)?;
    let a = unit(axis)?;
    let dot_ad = a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
    if d[2].abs() > PLANE_TOL || dot_ad.abs() > PLANE_TOL {
        return None;
    }
    // dot(d, cross(a, z)) and dot(a, z) for z the machine vertical:
    // the sine and the cosine of the turn about d standing a onto z.
    let t = (d[0] * a[1] - d[1] * a[0]).atan2(a[2]);
    t.is_finite().then_some(t)
}

/// `p` turned by `t` about the line `origin + s*dir` (Rodrigues'
/// formula), world frame.
#[must_use]
pub fn turn_point(origin: [f64; 3], dir: [f64; 3], p: [f64; 3], t: f64) -> [f64; 3] {
    let Some(d) = unit(dir) else { return p };
    let w = [p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]];
    let (s, c) = t.sin_cos();
    let k = [d[1] * w[2] - d[2] * w[1], d[2] * w[0] - d[0] * w[2], d[0] * w[1] - d[1] * w[0]];
    let e = (d[0] * w[0] + d[1] * w[1] + d[2] * w[2]) * (1.0 - c);
    [
        origin[0] + w[0] * c + k[0] * s + d[0] * e,
        origin[1] + w[1] * c + k[1] * s + d[1] * e,
        origin[2] + w[2] * c + k[2] * s + d[2] * e,
    ]
}

/// The playhead's work turn: the angle about `work`'s line that stands
/// the frame's tool axis upright, or `None` (the fallback draw) when
/// there is no line or the axis will not stand upright under it.
#[must_use]
pub fn frame_turn(work: &WorkTurn, axis: [f64; 3]) -> Option<f64> {
    upright_turn(work.dir, axis)
}

/// The tool as the playhead draws it under a work turn: the tip
/// carried rigidly about the line and the axis stood upright - the
/// machine turns the work, the spindle stays vertical. `None` (draw
/// `tip`/`axis` as given, today's picture) when the pose will not
/// stand upright: the fallback, never a guess.
#[must_use]
pub fn posed_tool(tip: [f64; 3], axis: [f64; 3], work: &WorkTurn) -> Option<([f64; 3], [f64; 3])> {
    let t = frame_turn(work, axis)?;
    Some((turn_point(work.origin, work.dir, tip, t), [0.0, 0.0, 1.0]))
}

/// The work-side vertices (`[x, y, z]`, display space) turned by `t`
/// about the line, in place - the part, the stock and the steel ride
/// the rotary table. A degenerate direction leaves them as they are.
pub fn turn_verts(verts: &mut [[f32; 3]], origin: [f64; 3], dir: [f64; 3], t: f64) {
    let Some(d) = unit(dir) else { return };
    let (s, c) = t.sin_cos();
    let k = |w: [f32; 3]| -> [f32; 3] {
        let w = [w[0] as f64, w[1] as f64, w[2] as f64];
        [
            (d[1] * w[2] - d[2] * w[1]) as f32,
            (d[2] * w[0] - d[0] * w[2]) as f32,
            (d[0] * w[1] - d[1] * w[0]) as f32,
        ]
    };
    let e = |w: [f32; 3]| -> f32 {
        let w = [w[0] as f64, w[1] as f64, w[2] as f64];
        ((d[0] * w[0] + d[1] * w[1] + d[2] * w[2]) * (1.0 - c)) as f32
    };
    let o = [origin[0] as f32, origin[1] as f32, origin[2] as f32];
    for v in verts.iter_mut() {
        let w = [v[0] - o[0], v[1] - o[1], v[2] - o[2]];
        let kv = k(w);
        *v = [o[0] + w[0] * c as f32 + kv[0] * s as f32 + d[0] as f32 * e(w), o[1] + w[1] * c as f32 + kv[1] * s as f32 + d[1] as f32 * e(w), o[2] + w[2] * c as f32 + kv[2] * s as f32 + d[2] as f32 * e(w)];
    }
}

#[cfg(test)]
#[path = "machine_view_tests.rs"]
mod tests;
