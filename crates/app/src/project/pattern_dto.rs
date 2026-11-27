//! The file mirror of a pattern folder of the operation list (#3012).
//! Provenance: our own project format.
//!
//! A folder is an entity row of kind `pattern` carrying this payload.
//! No version rung: a document without folders writes no such row, so
//! its bytes are unchanged, and a reader from before #3012 meets a kind
//! word it does not know and refuses the file by name (serde's "unknown
//! variant `pattern`") - never a silent load without the copies.

use serde::{Deserialize, Serialize};

use super::*;
use crate::pattern_folder::{checked_kind, copy_count};
use crate::schema::{PatternDef, PatternKind, PatternOrder};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PatternOrderDto {
    #[default]
    ByTool,
    ByPart,
}

/// The copies: `{"type": "linear", ...}` or `{"type": "grid", ...}`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PatternLayoutDto {
    Linear { dir: [f64; 3], count: u32, spacing: f64 },
    Grid { dir1: [f64; 3], count1: u32, spacing1: f64, dir2: [f64; 3], count2: u32, spacing2: f64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PatternDto {
    pub layout: PatternLayoutDto,
    #[serde(default, skip_serializing_if = "is_by_tool")]
    pub order_mode: PatternOrderDto,
    /// Member operation keys, in folder order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<String>,
    /// 1-based copy numbers left out of the program.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disabled: Vec<u32>,
    /// Position in the setup's list, the `OperationDto::order` law.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
    /// The display name (`RenameOperation`); absent - the key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The setup it stands in; absent - the first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub suppressed: bool,
}

fn is_by_tool(o: &PatternOrderDto) -> bool {
    *o == PatternOrderDto::ByTool
}

fn to_kind(layout: &PatternLayoutDto) -> PatternKind {
    match *layout {
        PatternLayoutDto::Linear { dir, count, spacing } => PatternKind::Linear { dir, count, spacing },
        PatternLayoutDto::Grid { dir1, count1, spacing1, dir2, count2, spacing2 } => {
            PatternKind::Grid { dir1, count1, spacing1, dir2, count2, spacing2 }
        }
    }
}

/// The payload for a folder row, `None` on every other entity.
pub(super) fn collect(world: &World, entity: &flecs_ecs::prelude::Entity) -> Option<PatternDto> {
    crate::pattern_folder::pattern_of(world, entity).map(|p| PatternDto {
        layout: match p.kind {
            PatternKind::Linear { dir, count, spacing } => PatternLayoutDto::Linear { dir, count, spacing },
            PatternKind::Grid { dir1, count1, spacing1, dir2, count2, spacing2 } => {
                PatternLayoutDto::Grid { dir1, count1, spacing1, dir2, count2, spacing2 }
            }
        },
        order_mode: match p.order_mode {
            PatternOrder::ByTool => PatternOrderDto::ByTool,
            PatternOrder::ByPart => PatternOrderDto::ByPart,
        },
        members: p.members,
        disabled: p.disabled_instances,
        order: model::operation_meta_of(world, entity).order,
        label: model::operation_meta_of(world, entity).label,
        setup: p.setup,
        suppressed: p.suppressed,
    })
}

/// The folder rows' structural law, run before any world write: the
/// payload rides a `pattern` row and nothing else; the row is named,
/// carries nothing but it, and its pattern passes the command's checks.
pub(super) fn validate(dto: &EntityDto, kind: EntityKind) -> Result<(), ProjectError> {
    let is_pattern = matches!(kind, EntityKind::Pattern);
    if dto.pattern.is_some() != is_pattern {
        return Err(corrupt(format!(
            "'{}' {}",
            dto.name,
            if is_pattern { "is a pattern folder without its pattern" } else { "carries a pattern but is not a pattern folder" }
        )));
    }
    let Some(p) = &dto.pattern else {
        return Ok(());
    };
    if dto.name.is_empty() {
        return Err(corrupt("a pattern folder must be named"));
    }
    if dto.facets.is_some() || dto.stock.is_some() || dto.tool.is_some() || dto.operation.is_some() || dto.setup.is_some() {
        return Err(corrupt(format!("'{}' is a pattern folder, not a solid, tool, operation or setup", dto.name)));
    }
    let kind = checked_kind(&to_kind(&p.layout)).map_err(|why| corrupt(format!("pattern folder '{}': {why}", dto.name)))?;
    let copies = copy_count(&kind);
    if p.disabled.iter().any(|n| *n == 0 || *n > copies) || p.disabled.len() >= usize::try_from(copies).unwrap_or(usize::MAX) {
        return Err(corrupt(format!("pattern folder '{}' turns off copies it does not have, or all of them", dto.name)));
    }
    for (i, m) in p.members.iter().enumerate() {
        if m.is_empty() || p.members[..i].contains(m) {
            return Err(corrupt(format!("pattern folder '{}' names an empty or repeated member", dto.name)));
        }
    }
    if p.label.as_deref().is_some_and(|l| l.trim().is_empty() || l.len() > 120 || l.chars().any(char::is_control)) {
        return Err(corrupt(format!("pattern folder '{}' has a display name that is not 1-120 printable characters", dto.name)));
    }
    if p.setup.as_deref().is_some_and(str::is_empty) {
        return Err(corrupt(format!("pattern folder '{}' names an empty setup", dto.name)));
    }
    Ok(())
}

/// Land a folder row (validated already).
pub(super) fn apply(world: &World, entity: &flecs_ecs::prelude::Entity, dto: Option<PatternDto>) -> Result<(), ProjectError> {
    let p = dto.ok_or_else(|| corrupt("pattern folder entity without its pattern"))?;
    let kind = checked_kind(&to_kind(&p.layout)).map_err(corrupt)?;
    let mut disabled = p.disabled;
    disabled.sort_unstable();
    disabled.dedup();
    entity.entity_view(world).set(PatternDef {
        setup: p.setup,
        suppressed: p.suppressed,
        order_mode: match p.order_mode {
            PatternOrderDto::ByTool => PatternOrder::ByTool,
            PatternOrderDto::ByPart => PatternOrder::ByPart,
        },
        kind,
        members: p.members,
        disabled_instances: disabled,
    });
    if p.order.is_some() || p.label.is_some() {
        entity.entity_view(world).set(OperationMeta { order: p.order, label: p.label, ..Default::default() });
    }
    Ok(())
}
