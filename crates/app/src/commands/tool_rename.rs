//! RenameTool (#4370): give a document tool a new name, and move every
//! operation that uses the tool to that name, in one undo step.
//! Provenance: our own command model; identity by name, like every
//! sibling tool command. The entity keeps its records (definition,
//! number, holder body, cutter, component stack); only the key moves.

use flecs_ecs::prelude::{Entity, World};

use crate::commands::{Command, CommandError};
use crate::model;
use crate::operation::{OpStatus, op_stamp, status_of};
use crate::schema::Name;

/// Rename the document tool `from` to `to`, with its operations.
pub struct RenameTool {
    from: String,
    to: String,
    /// The operations that name the tool, in document order.
    operations: Vec<String>,
}

impl RenameTool {
    /// Refused by name, before anything enters the history: an empty
    /// name, an unknown name, an entity that is not a tool, the same
    /// name, a name that another entity has or a setup reserves, a name
    /// that an operation still names (a deleted tool's operation would
    /// silently take this tool), and a name the machine reserves.
    pub fn new(world: &World, from: &str, to: &str) -> Result<Self, CommandError> {
        let to = to.trim();
        if let Some(err) = super::reserved(world, to) {
            return Err(err);
        }
        if to.chars().any(char::is_control) {
            return Err(CommandError::Invalid(format!("tool name '{}' has a control character", to.escape_debug())));
        }
        let Some(entity) = model::find_by_name(world, from) else {
            return Err(CommandError::NotFound(from.to_string()));
        };
        let tool = model::tool_of(world, &entity)
            .ok_or_else(|| CommandError::Invalid(format!("'{from}' is not a tool")))?;
        if to == from {
            return Err(CommandError::Invalid(format!("tool '{from}' already has that name")));
        }
        if model::find_by_name(world, to).is_some() {
            return Err(CommandError::NameConflict(to.to_string()));
        }
        let mut operations = Vec::new();
        for (name, e) in model::named_entities(world) {
            let Some(op) = model::operation_of(world, &e) else { continue };
            if op.tool == to {
                return Err(CommandError::Invalid(format!(
                    "operation '{name}' still names a tool '{to}': give it a tool first, or choose another name"
                )));
            }
            if op.tool == from {
                operations.push(name);
            }
        }
        if let Some(number) = crate::tool_numbers::number_of(world, from) {
            crate::tool_reservations::for_tool_as(world, from, to, tool.kind, number).map_err(CommandError::Invalid)?;
        }
        Ok(RenameTool { from: from.to_string(), to: to.to_string(), operations })
    }

    /// The tool's name before the rename.
    #[must_use]
    pub fn from(&self) -> &str {
        &self.from
    }

    /// The tool's new name.
    #[must_use]
    pub fn to(&self) -> &str {
        &self.to
    }

    /// The operations that follow the tool to its new name.
    #[must_use]
    pub fn operations(&self) -> &[String] {
        &self.operations
    }
}

/// Move the tool and its operations from `old` to `new`. A name is not
/// an input of the toolpath, so an operation that read Ok before reads
/// Ok after: its stamp (which hashes the tool's name) is taken again. A
/// Stale or never-generated operation keeps its stamp as it is.
fn rename(world: &World, old: &str, new: &str, operations: &[String]) {
    let fresh: Vec<Entity> = model::named_entities(world)
        .into_iter()
        .filter(|(_, e)| model::operation_of(world, e).is_some() && status_of(world, e) == OpStatus::Ok)
        .map(|(_, e)| e)
        .collect();
    if let Some(tool) = model::find_by_name(world, old) {
        tool.entity_view(world).set(Name(new.to_string()));
    }
    for name in operations {
        let Some(e) = model::find_by_name(world, name) else { continue };
        if let Some(mut op) = model::operation_of(world, &e).filter(|op| op.tool == old) {
            op.tool = new.to_string();
            e.entity_view(world).set(op);
        }
    }
    for e in fresh {
        let Some(mut op) = model::operation_of(world, &e) else { continue };
        let name = model::name_of(world, &e).unwrap_or_default();
        if let Some(stamp) = op_stamp(world, &op, &name)
            && op.stamp != Some(stamp)
        {
            op.stamp = Some(stamp);
            e.entity_view(world).set(op);
        }
    }
}

impl Command for RenameTool {
    fn apply(&self, world: &World) {
        rename(world, &self.from, &self.to, &self.operations);
    }

    fn invert(&self, world: &World) {
        rename(world, &self.to, &self.from, &self.operations);
    }
}
