//! Waterjet program checks (#1881).
//! Provenance: our own machine-limit checks over cached moves and the issue's XY/optional-Z rules.

use flecs_ecs::prelude::{Entity, World};
use super::{move_extremes, say, slowest_axis, Machine, MachineCheck};
use crate::{model, operation::Family, tools::ToolKind};

pub(super) fn check_waterjet_ops(world: &World, machine: &Machine, ops: &[(String, Entity)]) -> Result<MachineCheck, String> {
    let mut out = MachineCheck::default();
    out.notes.push(camcraft_i18n::tf("{}: no spindle; spindle rpm and power are not judged", &[format!("{}", machine.model)]));
    if machine.axis(2).is_none() {
        out.notes.push(camcraft_i18n::tf("{}: no Z axis; Z depth and travel are not judged", &[format!("{}", machine.model)]));
    }
    for (name, e) in ops {
        let op = model::operation_of(world, e).ok_or_else(|| camcraft_i18n::tf("{name}: lost its operation", &[format!("{}", name)]))?;
        if op.params.kind().info().family() != Family::Waterjet {
            say(&mut out, camcraft_i18n::tf("{name}: a waterjet machine admits only waterjet operations", &[format!("{}", name)]));
        }
        let tool = model::find_by_name(world, &op.tool).and_then(|e| model::tool_of(world, &e))
            .ok_or_else(|| camcraft_i18n::tf("{name}: tool {} is not in the library", &[format!("{}", name), format!("{}", op.tool)]))?;
        if tool.kind != ToolKind::Waterjet {
            say(&mut out, camcraft_i18n::tf("{name}: a waterjet operation requires a waterjet tool", &[format!("{}", name)]));
        }
        if let Some(why) = tool.torch_refusal() {
            say(&mut out, format!("{name}: {why}"));
        }
        let cache = model::toolpath_of(world, e).ok_or_else(|| camcraft_i18n::tf("{name}: lost its toolpath", &[format!("{}", name)]))?;
        let (_, origin) = crate::posting::wcs_context(world, op.setup.as_deref());
        for (i, mv) in cache.path.moves.iter().enumerate() {
            if mv.rotary.is_some() || mv.axis.is_some_and(|a| !crate::cl::is_vertical(&a)) {
                say(&mut out, camcraft_i18n::tf("{name}: move {i} tilts or turns; a waterjet has no rotary axis", &[format!("{}", name), format!("{}", i)]));
            }
            if let Some(feed) = mv.feed {
                // An arc drives both XY axes even when its endpoints coincide.
                let d = if mv.arc.is_some() { [1.0, 1.0, 0.0] }
                    else { [mv.to.x - mv.from.x, mv.to.y - mv.from.y, 0.0] };
                if let Some(ceil) = machine.feed_ceiling(d) && feed > ceil {
                    say(&mut out, camcraft_i18n::tf("{name}: move {i} feeds {feed:.0} mm/min, {} tops out at {ceil:.0}", &[format!("{}", name), format!("{}", i), format!("{:.0}", feed), format!("{}", slowest_axis(&machine.axes, ceil)), format!("{:.0}", ceil)]));
                }
            }
            for p in move_extremes(mv) {
                for (k, w) in [p.x, p.y, p.z].iter().enumerate() {
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
