//! The program stream of a Wire Contour (#533, part of #290): the
//! op's section as the wire EDM post reads it. A wire cut mounts no
//! tool (#532), so the section carries no TOOL, no spindle and no
//! coolant ask; it carries the machine's condition by its id (`PARAM
//! EdmCondition`, which only a script declaring `wireEdm` may post -
//! the engine refuses it by name for any other) and the product side
//! (`PARAM WireSide`, commentary). The moves are the op's cached
//! wire-centre path as generated - the kerf is already in them, so the
//! post states compensation off and never asks the control for a
//! second offset (#290 D5, #532's decision record).
//!
//! With skims (#538) each later pass is announced mid-section by its
//! own `PARAM EdmCondition` (`announce_skims`).
//!
//! Provenance: our own design over the stream contract
//! (docs/CNC-INTERMEDIATE.md) and #290's decisions D2, D5 and D10.

use crate::model;
use crate::operation::{self, WireContourOp};
use crate::post::{self, cnc, Dump};
use crate::post::AfterOperation;
use crate::schema::Operation;
use flecs_ecs::prelude::*;
use serde_json::json;

/// The section of one Wire Contour op `name` (entity `e`), from its
/// cached path. Refused by name when its heights do not resolve.
pub(super) fn stream(
    world: &World,
    name: &str,
    e: &Entity,
    op: &Operation,
    cache: &crate::schema::Toolpath,
    p: &WireContourOp,
    after: AfterOperation,
) -> Result<Vec<cnc::Record>, String> {
    let setup = op.setup.as_deref();
    let hs = operation::resolve_params(world, setup, Some(&op.src), &op.params).map_err(|err| format!("{name}: {err}"))?;
    let (wcs, origin) = super::wcs_context(world, setup);
    let (stock, stock_top_z) = match model::stock_world_aabb_for(world, setup) {
        Some(a) => ([a.min.x, a.max.x, a.min.y, a.max.y], a.max.z),
        None => ([0.0; 4], 0.0),
    };
    let display_name = model::operation_label(world, name, e);
    let op_kind = op.params.kind().to_string();
    let setup_name = crate::setup::effective(world, setup).unwrap_or_default();
    let d = Dump {
        op_name: &display_name,
        op_kind: &op_kind,
        setup_name: &setup_name,
        clearance_z: hs.clearance,
        retract_z: hs.clearance,
        top_z: stock_top_z,
        wcs,
        origin,
        stock,
        stock_top_z,
        after,
        warnings: cache
            .warnings
            .iter()
            .filter(|w| w.blocks_post())
            .map(|w| format!("{name}: {w}"))
            .collect(),
        ..Dump::default()
    };
    let mut records = post::dump(&cache.path, &d);
    // The wire words ride before the section opens, like every PARAM.
    let at = records
        .iter()
        .position(|r| matches!(r, cnc::Record::Section { begin: true }))
        .ok_or_else(|| format!("{name}: the wire section has no start"))?;
    let wire = [
        cnc::Record::Param { name: "EdmCondition".into(), value: json!(p.condition) },
        cnc::Record::Param { name: "WireSide".into(), value: json!(p.side.word()) },
    ];
    records.splice(at..at, wire);
    if !p.skims.is_empty() {
        announce_skims(&mut records, cache, p, name)?;
    }
    Ok(records)
}

/// #538: every pass starts and ends at the threading point (#532), so
/// each return to it closes a pass; the skim that follows is announced
/// by its own `PARAM EdmCondition` before its first move - the post
/// selects that condition there, with the wire still threaded. A path
/// that does not return once per pass is refused, never posted with
/// the wrong condition on a pass.
fn announce_skims(records: &mut Vec<cnc::Record>, cache: &crate::schema::Toolpath, p: &WireContourOp, name: &str) -> Result<(), String> {
    use crate::cl::MoveKind;
    let Some(start) = cache.path.moves.first().map(|m| m.from) else {
        return Err(format!("{name}: the wire path is empty - regenerate it"));
    };
    let at_start = |m: &cnc::MoveRec| (m.x - start.x).abs() < 1e-6 && (m.y - start.y).abs() < 1e-6;
    let mut out = Vec::with_capacity(records.len() + p.skims.len());
    let mut back = 0usize;
    let mut pending = false;
    for r in records.drain(..) {
        if let cnc::Record::Move(m) = &r
            && m.kind != MoveKind::Rapid
        {
            if pending {
                if let Some(s) = back.checked_sub(1).and_then(|i| p.skims.get(i)) {
                    out.push(cnc::Record::Param { name: "EdmCondition".into(), value: json!(s.condition) });
                }
                pending = false;
            }
            if at_start(m) {
                back = back.saturating_add(1);
                pending = true;
            }
        }
        out.push(r);
    }
    *records = out;
    if back != p.skims.len() + 1 {
        return Err(format!(
            "{name}: the wire path returns to its threading point {back} time(s), not once per pass ({} passes) - regenerate it",
            p.skims.len() + 1
        ));
    }
    Ok(())
}
