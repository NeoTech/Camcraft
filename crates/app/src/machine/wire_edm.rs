//! The machine check on a wire EDM (#532, part of #290): what a program
//! of wire cuts asks the machine, judged against what its file states.
//!
//! Provenance: our own rules over the umbrella's machine model (#290:
//! X/Y carry the lower guide and table, U/V the upper guide; the wire
//! is the machine's condition) and the unstated-limits law of
//! `assets/machines/AGENTS.md`. No vendor manual consulted.
//!
//! What is judged: the X/Y travel of every move against the stated
//! strokes, and the refusals by name - an op that is not wire EDM work,
//! and for a tapered cut (#536) every pose: its lean against the stated
//! maximum, and the upper guide's offset from the lower one (U/V,
//! from the stated guide planes) against the U and V strokes - no
//! guide planes, no U/V axis, or a lean or offset past what is stated
//! is refused by name. What is NOT: the wire sweep against the part,
//! the slug, the start hole's clearance (wire verification, #534), and
//! any tool number, rpm or feed - a wire EDM has no changer and no
//! spindle, and the control's generator sets the rate. Those are
//! SKIPPED with a named note on the check, never failed and never
//! silent.

use flecs_ecs::prelude::{Entity, World};

use super::{move_extremes, say, Machine, MachineCheck};
use crate::geometry::Point3;
use crate::model;
use crate::operation::Family;

/// [`super::check_ops`] for a machine of kind wire EDM.
pub(super) fn check_wire_ops(world: &World, machine: &Machine, ops: &[(String, Entity)]) -> Result<MachineCheck, String> {
    let mut out = MachineCheck::default();
    out.notes.push(camcraft_i18n::tf("{}: a wire EDM - only the X/Y travel is judged here; the wire's sweep against the part, the slug and the start hole are judged at the post by the wire laws (#534)", &[format!("{}", machine.model)]));
    out.notes.push(camcraft_i18n::tf("{}: a wire EDM has no spindle and no tool changer - rpm, feed and tool numbers are not judged (the control's generator sets the rate from the condition)", &[format!("{}", machine.model)]));
    for (name, e) in ops {
        let op = model::operation_of(world, e).ok_or_else(|| camcraft_i18n::tf("{name}: lost its operation", &[format!("{}", name)]))?;
        let kind = op.params.kind();
        let family = kind.info().family();
        if family != Family::WireEdm {
            say(
                &mut out,
                camcraft_i18n::tf("{name}: a {kind} operation is {} work - a wire EDM cuts only wire EDM operations", &[format!("{}", name), format!("{}", kind), format!("{}", family.work())]),
            );
            continue;
        }
        let cache = model::toolpath_of(world, e).ok_or_else(|| camcraft_i18n::tf("{name}: lost its toolpath", &[format!("{}", name)]))?;
        let (_, origin) = crate::posting::wcs_context(world, op.setup.as_deref());
        for (i, mv) in cache.path.moves.iter().enumerate() {
            if mv.rotary.is_some() {
                say(&mut out, camcraft_i18n::tf("{name}: move {i} turns a rotary axis - a wire EDM cut has none", &[format!("{}", name), format!("{}", i)]));
            }
            let mut reach = move_extremes(mv);
            if let Some(a) = mv.axis.filter(|a| !crate::cl::is_vertical(a)) {
                match judge_pose(machine, mv.to, a, origin[2]) {
                    // The X/Y the post writes for a lean: the lower guide's.
                    Ok(lower) => reach.push(Point3::new(lower[0], lower[1], mv.to.z)),
                    Err(why) => say(&mut out, format!("{name}: move {i} {why}")),
                }
            }
            for p in reach {
                for (k, w) in [p.x, p.y].iter().enumerate() {
                    let Some(ax) = machine.axis(k) else { continue };
                    let Some((lo, hi)) = ax.travel else { continue };
                    let v = (w - origin[k]) * ax.direction.sign();
                    if v < lo || v > hi {
                        say(&mut out, camcraft_i18n::tf("{name}: move {i} reaches {v:.3} in {}, the stroke is {lo:.3}..{hi:.3}", &[format!("{}", name), format!("{}", i), format!("{:.3}", v), format!("{}", ax.name), format!("{:.3}", lo), format!("{:.3}", hi)]));
                    }
                }
            }
        }
    }
    Ok(out)
}

/// The machine's guide planes in the work frame: stated in machine Z,
/// moved by the work offset's Z (`origin_z`).
///
/// # Errors
/// Either plane unstated, named.
pub fn guide_planes(machine: &Machine, origin_z: f64) -> Result<(f64, f64), String> {
    match machine.edm.as_ref().map(|e| (e.lower_guide_z, e.upper_guide_z)) {
        Some((Some(lo), Some(hi))) if lo.is_finite() && hi.is_finite() && hi > lo => Ok((lo + origin_z, hi + origin_z)),
        _ => Err(camcraft_i18n::tf("machine {} states no edm.lower_guide_z and edm.upper_guide_z - a tapered wire's guide positions are read from them", &[format!("{}", machine.model)])),
    }
}

/// The taper door (#1800): a non-zero `taper_deg` is admitted only on a
/// machine that can post it - one that states `edm.max_taper_deg` at or
/// above the taper and, when it tapers by U/V moves (no
/// `edm.taper_angle`), both guide planes. Read at the operation's
/// create and edit commands, so the GUI and MCP refuse the parameter by
/// name instead of a later generate or post. A zero taper is the
/// vertical cut and is always admitted. Nothing is guessed: an unstated
/// limit refuses.
///
/// # Errors
/// An unstated or exceeded maximum taper, or unstated guide planes on a
/// U/V machine, named.
pub fn taper_door(machine: &Machine, taper_deg: f64) -> Result<(), String> {
    if taper_deg == 0.0 {
        return Ok(());
    }
    let a = taper_deg.abs();
    let edm = machine.edm.as_ref();
    let Some(max) = edm.and_then(|e| e.max_taper_deg) else {
        return Err(camcraft_i18n::tf("a {a:.3} degree taper is asked, and machine {} states no edm.max_taper_deg - a taper is cut only on a machine that states how far it leans; set the taper to 0", &[format!("{:.3}", a), format!("{}", machine.model)]));
    };
    if a > max + 1e-9 {
        return Err(camcraft_i18n::tf("a {a:.3} degree taper is asked, and machine {} leans the wire at most {max:.3} degrees (edm.max_taper_deg)", &[format!("{:.3}", a), format!("{}", machine.model), format!("{:.3}", max)]));
    }
    if edm.and_then(|e| e.taper_angle).is_none() {
        guide_planes(machine, 0.0).map_err(|e| camcraft_i18n::tf("a {a:.3} degree taper is asked, and {e}; set the taper to 0", &[format!("{:.3}", a), format!("{}", e)]))?;
    }
    Ok(())
}

/// The wire of a move ending at `to` with pose `axis` (#290 D2: the
/// unit direction toward the upper guide) on the two guide planes
/// `(lower, upper)`: the lower guide's X Y and the upper guide's offset
/// from it (U V).
///
/// # Errors
/// A pose that is not a leaning-up direction.
pub fn guide_points(to: Point3, axis: [f64; 3], (lo, hi): (f64, f64)) -> Result<([f64; 2], [f64; 2]), String> {
    if !(axis.iter().all(|c| c.is_finite()) && axis[2] > 0.0175) {
        return Err("states a pose the wire cannot hold (leaning past 89 degrees, or not a direction)".into());
    }
    let (tx, ty) = (axis[0] / axis[2], axis[1] / axis[2]);
    let lower = [to.x + tx * (lo - to.z), to.y + ty * (lo - to.z)];
    let span = hi - lo;
    Ok((lower, [tx * span, ty * span]))
}

/// One pose against the machine: its lean against the stated maximum,
/// the workpiece's plane between the guides, and the U/V offset against
/// the U and V strokes.
fn judge_pose(machine: &Machine, to: Point3, axis: [f64; 3], origin_z: f64) -> Result<[f64; 2], String> {
    let lean = axis[0].hypot(axis[1]).atan2(axis[2]).to_degrees();
    let edm = machine.edm.as_ref();
    match edm.and_then(|e| e.max_taper_deg) {
        None => return Err(camcraft_i18n::tf("leans the wire {lean:.3} degrees, and machine {} states no edm.max_taper_deg", &[format!("{:.3}", lean), format!("{}", machine.model)])),
        Some(max) if lean > max + 1e-9 => {
            return Err(camcraft_i18n::tf("leans the wire {lean:.3} degrees, past machine {}'s {max:.3} degree maximum taper", &[format!("{:.3}", lean), format!("{}", machine.model), format!("{:.3}", max)]));
        }
        Some(_) => {}
    }
    let (lo, hi) = guide_planes(machine, origin_z)?;
    if !(lo..=hi).contains(&to.z) {
        return Err(camcraft_i18n::tf("rides Z{:.3}, outside the guides' reach (Z{:.3} .. Z{:.3}) - the wire's programmed plane must stand between its guides", &[format!("{:.3}", to.z - origin_z), format!("{:.3}", lo - origin_z), format!("{:.3}", hi - origin_z)]));
    }
    let (lower, uv) = guide_points(to, axis, (lo, hi))?;
    for (letter, w) in [("U", uv[0]), ("V", uv[1])] {
        let Some(ax) = machine.axes.iter().find(|a| a.name.eq_ignore_ascii_case(letter)) else {
            return Err(camcraft_i18n::tf("needs the upper guide {w:.3} mm off in {letter}, and machine {} states no {letter} axis", &[format!("{:.3}", w), format!("{}", letter), format!("{}", machine.model), format!("{}", letter)]));
        };
        if let Some((min, max)) = ax.travel
            && !(min..=max).contains(&w)
        {
            return Err(camcraft_i18n::tf("needs {letter}{w:.3}, outside the {letter} stroke {min:.3}..{max:.3} (the upper guide's travel band)", &[format!("{}", letter), format!("{:.3}", w), format!("{}", letter), format!("{:.3}", min), format!("{:.3}", max)]));
        }
    }
    Ok(lower)
}
