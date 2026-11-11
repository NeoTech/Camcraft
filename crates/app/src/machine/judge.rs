//! The per-move judgment helpers the machine checks share (#669 split
//! them out of `mod.rs` when the plasma check became their second
//! reader): a move's extreme points, the axis a feed ceiling belongs
//! to, the C-letter gate, the rigid-tap gate (#786), and the capped
//! reporter.
//!
//! Provenance: general machine-tool practice (an arc bulges to its
//! quarter points; a coordinated feed is capped by its slowest driven
//! axis; a rigid tap needs a spindle the control can synchronise the
//! feed to - RS-274 G84), our own code moved unchanged from
//! `machine/mod.rs` plus the tap gate.

use super::{AxisLimits, Machine, MachineCheck, REPORT_CAP};
use crate::geometry::Point3;

/// The arc's four XY quarter points plus both endpoints; a straight
/// move's two endpoints. (A quarter is outside the sweep only when
/// the arc goes the other way around it; checking all four can only
/// over-report a limit the post would then also cross, never hide a
/// violation - and over-reporting a REAL stroke is stage C's honest
/// direction of error.)
pub(super) fn move_extremes(mv: &crate::cl::Move) -> Vec<Point3> {
    let mut v = vec![mv.from, mv.to];
    // XZ / YZ (M11.2 A): the same four quarters, in the arc's plane.
    if let Some(arc) = mv.arc.filter(|a| !a.plane.is_xy()) {
        let pl = arc.plane;
        let (cu, cv) = pl.center_uv(arc.center);
        let (u0, v0, _) = pl.uvw(mv.from);
        let (_, _, w1) = pl.uvw(mv.to);
        let r = ((u0 - cu).powi(2) + (v0 - cv).powi(2)).sqrt();
        v.push(pl.point(cu + r, cv, w1));
        v.push(pl.point(cu - r, cv, w1));
        v.push(pl.point(cu, cv + r, w1));
        v.push(pl.point(cu, cv - r, w1));
        return v;
    }
    if let Some(arc) = &mv.arc {
        let dx = mv.from.x - arc.center.x;
        let dy = mv.from.y - arc.center.y;
        let r = (dx * dx + dy * dy).sqrt();
        v.push(Point3::new(arc.center.x + r, arc.center.y, mv.to.z));
        v.push(Point3::new(arc.center.x - r, arc.center.y, mv.to.z));
        v.push(Point3::new(arc.center.x, arc.center.y + r, mv.to.z));
        v.push(Point3::new(arc.center.x, arc.center.y - r, mv.to.z));
    }
    v
}

pub(super) fn slowest_axis(axes: &[AxisLimits], ceil: f64) -> String {
    axes.iter()
        .find(|a| a.feed == Some(ceil))
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "an axis".into())
}

pub(super) fn say(out: &mut MachineCheck, s: String) {
    if out.problems.len() < REPORT_CAP {
        out.problems.push(s);
    } else {
        out.more += 1;
    }
}

/// The C letter gate (issue #33): a move stating spindle C angle `c`
/// needs a rotary named C on the machine, and a stated C travel must
/// hold the unwrapped angle. None when it is fine.
pub(super) fn c_letter_problem(machine: &Machine, i: usize, c: f64) -> Option<String> {
    let Some(ax) = machine.axes.iter().find(|x| x.rotary && x.name.eq_ignore_ascii_case("C")) else {
        return Some(format!("move {i} turns the spindle as C {c:.3} deg; {} states no C axis", machine.model));
    };
    let (lo, hi) = ax.travel?;
    (c < lo - 1e-9 || c > hi + 1e-9)
        .then(|| format!("move {i} commands C {c:.3} deg, the travel is {lo:.3}..{hi:.3}"))
}

/// The rigid-tap gate (#786): a path that carries a tap cycle, on a
/// machine whose file says its spindle cannot rigid-tap
/// (`spindle.rigid_tapping: false`), is refused by name - whatever post
/// is chosen, and never converted to another cycle. Unstated reads as
/// "not limited" (the unstated-limit law): nothing is said then.
pub(super) fn tap_gate(out: &mut MachineCheck, machine: &Machine, name: &str, path: &crate::cl::Toolpath) {
    if machine.rigid_tapping != Some(false) {
        return;
    }
    let taps = path.cycles.iter().filter(|c| c.kind == crate::cl::CycleKind::Tap).count();
    if taps > 0 {
        let m = &machine.model;
        say(out, format!("{name}: {taps} tap cycle(s) need rigid tapping, which {m} cannot do - use a tapping head or thread milling"));
    }
}
