//! Pattern folders at the post (#3012): which sections the program
//! writes for a folder's copies, and the gate that judges each copy
//! against the machine travel and the fixtures.
//!
//! Provenance: linear and grid part patterns as general CAM practice;
//! the link between copies is the clearance-plane rapid of
//! `cl::transform::copies`. Our own law; no vendor material consulted.
//!
//! By tool: each member is one section that cuts every enabled copy,
//! linked at its clearance plane, so the tool changes once. By part:
//! the members cut copy 1, then copy 2, and so on, one section each.
//! Every copy posts in the setup's own WCS; the moves are shifted, the
//! work offset is not.

use std::borrow::Cow;
use std::collections::HashMap;

use flecs_ecs::prelude::{Entity, World};

use crate::cl::transform::copies;
use crate::cl::Toolpath as ClPath;
use crate::model;
use crate::pattern_folder::{enabled_offsets, folders, member_problem};
use crate::schema::{PatternDef, PatternOrder};

/// One section of the posted program.
pub(super) struct Unit {
    pub name: String,
    pub entity: Entity,
    /// The copies' shifts; empty for an operation outside a folder (its
    /// own path, untouched).
    pub offsets: Vec<[f64; 3]>,
    /// The operation's last section: its after-action and its setup
    /// change ride this one only.
    pub last: bool,
}

/// The path a unit posts: the cache itself, or its joined copies.
pub(super) fn path_for<'a>(unit: &Unit, path: &'a ClPath, link_z: f64) -> Result<Cow<'a, ClPath>, String> {
    if unit.offsets.is_empty() {
        return Ok(Cow::Borrowed(path));
    }
    copies(path, &unit.offsets, link_z).map(Cow::Owned).map_err(|e| e.to_string())
}

fn label(world: &World, key: &str) -> String {
    model::find_by_name(world, key).map_or_else(|| key.to_owned(), |e| model::operation_label(world, key, &e))
}

/// The folders that make copies (not suppressed, with a posted member),
/// each with its posted members in folder order.
fn active(world: &World, ops: &[(String, Entity)]) -> Vec<(String, PatternDef, Vec<String>)> {
    folders(world)
        .into_iter()
        .filter(|(_, def)| !def.suppressed)
        .filter_map(|(name, def)| {
            let posted: Vec<String> =
                def.members.iter().filter(|m| ops.iter().any(|(n, _)| n == *m)).cloned().collect();
            (!posted.is_empty()).then_some((name, def, posted))
        })
        .collect()
}

/// A folder the post cannot write, by name.
fn refusal(world: &World, name: &str, def: &PatternDef, posted: &[String], ops: &[(String, Entity)]) -> Option<String> {
    let folder = label(world, name);
    let setup = def.setup.as_deref();
    let square = crate::setup::wcs_entity(world, setup)
        .and_then(|e| model::transform_of(world, &e))
        .is_none_or(|loc| matches!(super::wcs_rotation::offset_frame(&loc), super::wcs_rotation::OffsetFrame::Square));
    if !square || crate::setup_wcs_flip::of_setup(world, setup).iter().any(|f| *f) {
        return Some(format!(
            "{folder}: the setup's WCS is turned or flipped - a pattern folder copies along an unturned WCS only"
        ));
    }
    let home = crate::setup::effective(world, setup);
    for m in &def.members {
        let Some(e) = model::find_by_name(world, m) else {
            return Some(format!("{folder}: member '{m}' is gone - set the folder's members again"));
        };
        if let Some(why) = member_problem(world, m, &e) {
            return Some(format!("{folder}: {why}"));
        }
        if crate::program_entry::effective_setup(world, &e) != home {
            return Some(format!("{folder}: member '{}' is in another setup - move it back or take it out", label(world, m)));
        }
    }
    // The posted members stand together, in folder order (the row law).
    let at = ops.iter().position(|(n, _)| *n == posted[0]);
    let together = at.is_some_and(|i| ops.get(i..i + posted.len()).is_some_and(|s| s.iter().map(|(n, _)| n).eq(posted.iter())));
    if !together {
        return Some(format!(
            "{folder}: its member operations must stand directly after it, in folder order - reorder them"
        ));
    }
    None
}

/// The program's sections in order, the folders expanded. A folder the
/// post cannot write refuses the whole program, by name.
pub(super) fn plan(world: &World, ops: &[(String, Entity)]) -> Result<Vec<Unit>, String> {
    let active = active(world, ops);
    let mut owner: HashMap<&str, usize> = HashMap::new();
    for (i, (name, def, posted)) in active.iter().enumerate() {
        if let Some(why) = refusal(world, name, def, posted, ops) {
            return Err(why);
        }
        for m in posted {
            owner.insert(m.as_str(), i);
        }
    }
    let mut out = Vec::with_capacity(ops.len());
    let mut i = 0;
    while let Some((name, entity)) = ops.get(i) {
        let Some((_, def, posted)) = owner.get(name.as_str()).and_then(|f| active.get(*f)) else {
            out.push(Unit { name: name.clone(), entity: *entity, offsets: Vec::new(), last: true });
            i += 1;
            continue;
        };
        let members: Vec<(String, Entity)> = ops.iter().skip(i).take(posted.len()).cloned().collect();
        let shifts: Vec<[f64; 3]> = enabled_offsets(def).into_iter().map(|(_, o)| o).collect();
        match def.order_mode {
            PatternOrder::ByTool => out.extend(
                members.into_iter().map(|(name, entity)| Unit { name, entity, offsets: shifts.clone(), last: true }),
            ),
            PatternOrder::ByPart => {
                for (k, shift) in shifts.iter().enumerate() {
                    let last = k + 1 == shifts.len();
                    out.extend(members.iter().map(|(name, entity)| Unit {
                        name: name.clone(),
                        entity: *entity,
                        offsets: vec![*shift],
                        last,
                    }));
                }
            }
        }
        i += posted.len();
    }
    Ok(out)
}

/// The pattern clause of the verification gate: each shifted copy of a
/// member is judged against its setup's machine travel and fixtures,
/// as copy 1 is by the program's own clauses. A plan refusal blocks.
pub(super) fn pattern_problems(world: &World) -> Vec<String> {
    let Ok(ops) = super::program_ops(world) else {
        return Vec::new(); // the program's own refusal speaks first
    };
    let mut out = Vec::new();
    for (name, def, posted) in active(world, &ops) {
        if let Some(why) = refusal(world, &name, &def, &posted, &ops) {
            out.push(why);
            continue;
        }
        let (_, machine) = crate::machine::of_setup(world, def.setup.as_deref());
        let (turning, fixed) = crate::fixture::workholding_for(world, def.setup.as_deref());
        let meshes = crate::fixture::meshes_for(world, def.setup.as_deref());
        for (copy, shift) in enabled_offsets(&def).into_iter().filter(|(_, o)| *o != [0.0; 3]) {
            for m in &posted {
                let Some(e) = model::find_by_name(world, m) else { continue };
                let (Some(op), Some(cache)) = (model::operation_of(world, &e), model::toolpath_of(world, &e)) else {
                    continue; // the post's own walk refuses a lost op by name
                };
                let at = format!("{} copy {copy} ({})", label(world, m), label(world, &name));
                let path = match copies(&cache.path, &[shift], 0.0) {
                    Ok(p) => p,
                    Err(why) => {
                        out.push(format!("{at}: {why}"));
                        continue;
                    }
                };
                if let Some(machine) = &machine {
                    let (_, origin) = super::wcs_context(world, op.setup.as_deref());
                    let mut check = crate::machine::MachineCheck::default();
                    let source = Some(crate::machine::SourceLocation::Operation(&at));
                    check.check_path(machine, source, &path, origin, [false; 3], true);
                    out.extend(check.problems);
                }
                match crate::fixture_struck::op_hits(world, &op, &path, &turning, &fixed, &meshes) {
                    Ok(hits) => out.extend(hits.into_iter().map(|hit| format!("{at}: {hit}"))),
                    Err(why) => out.push(format!("{at}: {why}")),
                }
            }
        }
    }
    out
}

/// The not-judged line for copies: the stock sim walks each member once,
/// at copy 1, so the stock and model verdicts say nothing of the others.
pub(super) fn not_judged(world: &World) -> Option<String> {
    let ops = super::program_ops(world).ok()?;
    let names: Vec<String> = active(world, &ops)
        .into_iter()
        .filter(|(_, def, _)| enabled_offsets(def).iter().any(|(_, o)| *o != [0.0; 3]))
        .map(|(name, _, _)| label(world, &name))
        .collect();
    (!names.is_empty()).then(|| {
        format!(
            "pattern copies ({}): the stock sim judges each member at copy 1 only - the other copies are checked for machine travel and fixtures, not against stock or model",
            names.join(", ")
        )
    })
}
