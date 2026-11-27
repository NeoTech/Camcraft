//! #4339: convert standalone document tools to component stacks in one
//! undo step - the rows of the `tool_convert` plan that the user selects.
//! The components each stack needs join the document's table (built-in
//! copies keep their library link), each tool gets its stack, and its
//! derived record and body land as `SetToolAssembly` lands them. Checked
//! whole at construction: a "no match" row refuses the whole command by
//! the tool's name and its reason, and nothing changes. Our own command
//! model, the snapshot-and-restore inverse every sibling uses.

use flecs_ecs::prelude::World;

use crate::commands::tool_assembly::ToolUpdate;
use crate::commands::{Command, CommandError};
use crate::model;
use crate::schema::{DocumentComponent, ToolAssembly};
use crate::tool_assembly::{self as ta, components, table_set};
use crate::tool_convert::{plan, Verdict};

/// Convert the selected standalone tools to component stacks.
pub struct ConvertToComponents {
    prev: Vec<DocumentComponent>,
    next: Vec<DocumentComponent>,
    tools: Vec<(String, ToolAssembly, ToolUpdate)>,
}

impl ConvertToComponents {
    /// Convert the tools `names` (the user's selection: a "different" row
    /// converts only when it is named). Refused by name: an empty
    /// selection, a name twice, a name that is not a document tool, a
    /// "no match" row (its reason), and every `SetToolAssembly` refusal.
    pub fn new(world: &World, names: &[String]) -> Result<Self, CommandError> {
        if names.is_empty() {
            return Err(CommandError::Invalid("no tool is selected to convert".to_owned()));
        }
        let mut seen: Vec<&str> = Vec::with_capacity(names.len());
        for n in names {
            if seen.contains(&n.as_str()) {
                return Err(CommandError::Invalid(format!("the tool '{n}' is named twice")));
            }
            seen.push(n);
        }
        let rows = plan(world, names).map_err(CommandError::Invalid)?;
        let prev = components(world);
        let mut next = prev.clone();
        for row in &rows {
            if let Verdict::NoMatch(why) = &row.verdict {
                return Err(CommandError::Invalid(format!("the tool '{}' is not converted: {why}", row.tool)));
            }
            for add in &row.adds {
                if !next.iter().any(|r| r.component.id() == add.component.id()) {
                    next.push(add.clone());
                }
            }
        }
        let mut tools = Vec::with_capacity(rows.len());
        for row in rows {
            let update = ToolUpdate::new(world, &row.tool, &row.links, &next)?;
            tools.push((row.tool, ToolAssembly(row.links), update));
        }
        Ok(Self { prev, next, tools })
    }

    /// Convert every standalone tool whose stack derives the same record
    /// and gauge length (the preview's default selection). Refused by
    /// name when there is none.
    pub fn same_rows(world: &World) -> Result<Self, CommandError> {
        let same: Vec<String> = plan(world, &[])
            .map_err(CommandError::Invalid)?
            .into_iter()
            .filter(|r| r.verdict == Verdict::Same)
            .map(|r| r.tool)
            .collect();
        if same.is_empty() {
            return Err(CommandError::Invalid("no standalone tool converts to the same tool record".to_owned()));
        }
        Self::new(world, &same)
    }

    /// The tools this command converts, in selection order.
    #[must_use]
    pub fn tools(&self) -> Vec<String> {
        self.tools.iter().map(|(n, _, _)| n.clone()).collect()
    }
}

impl Command for ConvertToComponents {
    fn apply(&self, world: &World) {
        table_set(world, self.next.clone());
        for (name, stack, update) in &self.tools {
            update.apply(world);
            if let Some(entity) = model::find_by_name(world, name) {
                ta::set_assembly(world, &entity, Some(stack));
            }
        }
    }

    fn invert(&self, world: &World) {
        for (name, _, update) in self.tools.iter().rev() {
            if let Some(entity) = model::find_by_name(world, name) {
                ta::set_assembly(world, &entity, None);
            }
            update.invert(world);
        }
        table_set(world, self.prev.clone());
    }
}
