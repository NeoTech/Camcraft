//! Pattern folders of the operation list (#3012): which operations a
//! folder holds, where its copies stand, and the row law that keeps a
//! folder's members directly after it.
//!
//! Provenance: linear and rectangular (grid) part patterns as general
//! CAM and machining-handbook practice; our own document model, no
//! vendor material consulted.
//!
//! The law: a folder is a row of the operation list, like a Pause entry
//! (`program_entry`), and not an `Operation`. Its members are ordinary
//! operations of the same setup. They stand directly after the folder,
//! in folder order, so the list, the stock sim and the post walk them in
//! one order. Copy 1 is the members' own place; copy `k` is shifted in
//! the XY plane of the setup's WCS. Only the post makes the copies.

use std::collections::{HashMap, HashSet};

use flecs_ecs::core::EntityViewGet; // not re-exported in prelude (0.2.2)
use flecs_ecs::prelude::{Entity, World};

use crate::model;
use crate::operation::{Family, OpParams};
use crate::schema::{PatternDef, PatternKind};

/// The most copies along one direction.
pub const MAX_COUNT: u32 = 1000;
/// The most copies of one folder (the cl join's own limit).
pub const MAX_COPIES: u32 = 10_000;
/// The largest spacing, in millimetres (100 m).
pub const MAX_SPACING: f64 = 100_000.0;

/// The folder on `entity`, if it is one.
#[must_use]
pub fn pattern_of(world: &World, entity: &Entity) -> Option<PatternDef> {
    entity.entity_view(world).try_get::<&PatternDef>(Clone::clone)
}

/// Is this named entity a pattern folder?
#[must_use]
pub fn is_pattern(world: &World, entity: &Entity) -> bool {
    entity.entity_view(world).try_get::<&PatternDef>(|_| ()).is_some()
}

/// Every pattern folder, by key order.
#[must_use]
pub fn folders(world: &World) -> Vec<(String, PatternDef)> {
    model::named_entities(world)
        .into_iter()
        .filter_map(|(n, e)| pattern_of(world, &e).map(|p| (n, p)))
        .collect()
}

/// Every pattern folder's `(key, entity)`, by key order.
#[must_use]
pub fn folder_rows(world: &World) -> Vec<(String, Entity)> {
    model::named_entities(world).into_iter().filter(|(_, e)| is_pattern(world, e)).collect()
}

/// The folder that holds operation `op`, if one does.
#[must_use]
pub fn folder_of_member(world: &World, op: &str) -> Option<(String, PatternDef)> {
    folders(world).into_iter().find(|(_, p)| p.members.iter().any(|m| m == op))
}

/// How many copies the pattern makes (enabled or not).
#[must_use]
pub fn copy_count(kind: &PatternKind) -> u32 {
    match kind {
        PatternKind::Linear { count, .. } => *count,
        PatternKind::Grid { count1, count2, .. } => count1.saturating_mul(*count2),
    }
}

/// Every copy's shift, by copy number (index 0 is copy 1, no shift).
/// A grid numbers along `dir1` first, then the next row along `dir2`.
#[must_use]
pub fn offsets(kind: &PatternKind) -> Vec<[f64; 3]> {
    let shift = |d: &[f64; 3], s: f64, i: u32| [d[0] * s * f64::from(i), d[1] * s * f64::from(i), 0.0];
    match kind {
        PatternKind::Linear { dir, count, spacing } => (0..*count).map(|i| shift(dir, *spacing, i)).collect(),
        PatternKind::Grid { dir1, count1, spacing1, dir2, count2, spacing2 } => (0..*count2)
            .flat_map(|j| {
                (0..*count1).map(move |i| {
                    let a = shift(dir1, *spacing1, i);
                    let b = shift(dir2, *spacing2, j);
                    [a[0] + b[0], a[1] + b[1], 0.0]
                })
            })
            .collect(),
    }
}

/// The enabled copies, `(copy number, shift)`, in copy order.
#[must_use]
pub fn enabled_offsets(def: &PatternDef) -> Vec<(u32, [f64; 3])> {
    (1u32..)
        .zip(offsets(&def.kind))
        .filter(|(n, _)| !def.disabled_instances.contains(n))
        .collect()
}

/// A unit direction in the XY plane, or a refusal that names `what`.
fn unit_xy(dir: [f64; 3], what: &str) -> Result<[f64; 3], String> {
    if !dir.iter().all(|v| v.is_finite()) {
        return Err(format!("{what} must be finite"));
    }
    if dir[2].abs() > 1e-9 {
        return Err(format!("{what} must lie in the XY plane (Z 0) - a pattern copies parts side by side"));
    }
    let len = dir[0].hypot(dir[1]);
    if len < 1e-9 {
        return Err(format!("{what} must not be zero"));
    }
    Ok([dir[0] / len, dir[1] / len, 0.0])
}

fn checked_count(count: u32, spacing: f64, what: &str) -> Result<(), String> {
    if count == 0 || count > MAX_COUNT {
        return Err(format!("{what} count must be 1 to {MAX_COUNT}"));
    }
    if !spacing.is_finite() || spacing <= 0.0 || spacing > MAX_SPACING {
        return Err(format!("{what} spacing must be a finite length above 0 and at most {MAX_SPACING} mm"));
    }
    Ok(())
}

/// The pattern as stored: directions made unit, every count and spacing
/// checked. Refusals name the field.
pub fn checked_kind(kind: &PatternKind) -> Result<PatternKind, String> {
    let out = match *kind {
        PatternKind::Linear { dir, count, spacing } => {
            checked_count(count, spacing, "the")?;
            PatternKind::Linear { dir: unit_xy(dir, "the direction")?, count, spacing }
        }
        PatternKind::Grid { dir1, count1, spacing1, dir2, count2, spacing2 } => {
            checked_count(count1, spacing1, "the first")?;
            checked_count(count2, spacing2, "the second")?;
            let (d1, d2) = (unit_xy(dir1, "the first direction")?, unit_xy(dir2, "the second direction")?);
            if (d1[0] * d2[1] - d1[1] * d2[0]).abs() < 1e-6 {
                return Err("the two grid directions must not be parallel - the copies would overlap".into());
            }
            PatternKind::Grid { dir1: d1, count1, spacing1, dir2: d2, count2, spacing2 }
        }
    };
    if copy_count(&out) > MAX_COPIES {
        return Err(format!("a pattern makes at most {MAX_COPIES} copies"));
    }
    Ok(out)
}

/// Why operation `op` cannot be patterned, if it cannot: only milling
/// work that a shift carries (no probe: its touch sets a work offset).
#[must_use]
pub fn member_problem(world: &World, op: &str, entity: &Entity) -> Option<String> {
    let Some(operation) = model::operation_of(world, entity) else {
        return Some(format!("'{op}' is not an operation"));
    };
    if operation.params.kind().info().family() != Family::Mill {
        return Some(format!("'{op}' is not milling work - only milling operations can be patterned"));
    }
    if matches!(operation.params, OpParams::Probe { .. }) {
        return Some(format!("'{op}' is a probe - its touch sets the work offset, so it cannot be copied"));
    }
    None
}

/// `rows` (one setup's list, in order) with each folder's members moved
/// to directly after it, in folder order. `members` overrides one
/// folder's member list (a command's new list). Members that are not in
/// `rows` are left out.
#[must_use]
pub fn gather(world: &World, rows: &[String], members: Option<(&str, &[String])>) -> Vec<String> {
    let present: HashSet<&str> = rows.iter().map(String::as_str).collect();
    let mut held: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let list = match members {
            Some((key, list)) if key == row => Some(list.to_vec()),
            _ => model::find_by_name(world, row).and_then(|e| pattern_of(world, &e)).map(|p| p.members),
        };
        if let Some(list) = list {
            held.insert(row.clone(), list.into_iter().filter(|m| present.contains(m.as_str())).collect());
        }
    }
    let inside: HashSet<&str> = held.values().flatten().map(String::as_str).collect();
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if inside.contains(row.as_str()) {
            continue;
        }
        out.push(row.clone());
        if let Some(list) = held.get(row) {
            out.extend(list.iter().cloned());
        }
    }
    out
}
