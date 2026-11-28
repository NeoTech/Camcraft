//! DuplicateTool (#4340): copy a document tool as a new tool, with every
//! record it carries (geometry, cutting data, holder body, drawn cutter,
//! component stack), under a free name and the next free tool number.
//! Provenance: our own command model; it lands the same `ToolRecords`
//! list the delete path snapshots, so a copy never drops a record.

use flecs_ecs::prelude::World;

use crate::commands::{Command, CommandError};
use crate::model;

use super::ToolRecords;

/// How many name candidates the copy tries before it refuses.
const NAME_TRIES: u32 = 10_000;

/// Make a new document tool that is a copy of `source`, in one undo step.
pub struct DuplicateTool {
    source: String,
    name: String,
    number: u32,
    records: ToolRecords,
}

impl DuplicateTool {
    /// Refused by name, before anything enters the history: an unknown
    /// name, an entity that is not a tool, no free name, no free number.
    pub fn new(world: &World, source: &str) -> Result<Self, CommandError> {
        let Some(entity) = model::find_by_name(world, source) else {
            return Err(CommandError::NotFound(source.to_string()));
        };
        let records = ToolRecords::of(world, &entity)
            .ok_or_else(|| CommandError::Invalid(format!("'{source}' is not a tool")))?;
        let kind = model::tool_of(world, &entity)
            .ok_or_else(|| CommandError::Invalid(format!("'{source}' is not a tool")))?
            .kind;
        let name = free_name(world, source)?;
        let taken: Vec<u32> = crate::tool_numbers::numbers(world)
            .map_err(CommandError::Invalid)?
            .into_iter()
            .map(|(_, n)| n)
            .collect();
        let number = crate::tool_reservations::allocate(world, &name, kind, &taken)
            .map_err(|why| CommandError::Invalid(format!("cannot duplicate tool '{source}': {why}")))?;
        Ok(DuplicateTool { source: source.to_string(), name, number, records: records.renumbered(number) })
    }

    /// The tool this copies.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The copy's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The copy's tool number.
    #[must_use]
    pub fn number(&self) -> u32 {
        self.number
    }
}

/// `<stem> (2)`, `<stem> (3)`, ...: the first name no entity has and no
/// setup reserves. The stem drops a trailing ` (n)`, so the copy of
/// `EM (2)` is `EM (3)`, not `EM (2) (2)`.
fn free_name(world: &World, source: &str) -> Result<String, CommandError> {
    let stem = strip_counter(source);
    (2..NAME_TRIES)
        .map(|n| format!("{stem} ({n})"))
        .find(|c| model::find_by_name(world, c).is_none() && super::reserved(world, c).is_none())
        .ok_or_else(|| CommandError::Invalid(format!("cannot duplicate tool '{source}': no free name for the copy")))
}

fn strip_counter(name: &str) -> &str {
    let Some(body) = name.strip_suffix(')') else { return name };
    match body.rsplit_once(" (") {
        Some((stem, n)) if !stem.is_empty() && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => stem,
        _ => name,
    }
}

impl Command for DuplicateTool {
    fn apply(&self, world: &World) {
        let entity = match model::find_by_name(world, &self.name) {
            Some(entity) => entity,
            None => model::spawn(world, &self.name),
        };
        self.records.land(world, &entity);
    }

    fn invert(&self, world: &World) {
        if let Some(entity) = model::find_by_name(world, &self.name) {
            model::remove(world, &entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::strip_counter;

    #[test]
    fn the_stem_drops_only_a_numeric_counter() {
        assert_eq!(strip_counter("EM (2)"), "EM");
        assert_eq!(strip_counter("EM (12)"), "EM");
        assert_eq!(strip_counter("EM (a)"), "EM (a)");
        assert_eq!(strip_counter("EM ()"), "EM ()");
        assert_eq!(strip_counter("(3)"), "(3)");
        assert_eq!(strip_counter("EM"), "EM");
    }
}
