//! The T/H number laws of an import (#2442, #2566): a library's own
//! numbers checked whole, and each landing tool's number claimed against
//! the document's. Moved out of `tool_import` (#3645) to keep the module
//! under 400 lines. Our own command model.

use flecs_ecs::prelude::World;

use crate::commands::CommandError;
use crate::tool_library::LibraryTool;

pub(super) fn validate_library_numbers(library: &[LibraryTool]) -> Result<(), CommandError> {
    let mut seen: Vec<(&str, u32)> = Vec::new();
    for entry in library {
        let Some(n) = entry.number else { continue };
        if !crate::tool_numbers::valid(n) {
            return Err(CommandError::Invalid(format!(
                "tool '{}': tool number {n} is out of range - tool numbers run {}",
                entry.name,
                crate::tool_numbers::RANGE_TEXT
            )));
        }
        if let Some((other, _)) = seen.iter().find(|(_, num)| *num == n) {
            return Err(CommandError::Invalid(format!(
                "tools '{other}' and '{}' both claim T{n}",
                entry.name
            )));
        }
        seen.push((entry.name.as_str(), n));
    }
    Ok(())
}

pub(super) fn claim_number(
    world: &World,
    kind: crate::tools::ToolKind,
    owners: &mut Vec<(String, u32)>,
    name: &str,
    explicit: Option<u32>,
) -> Result<u32, CommandError> {
    if let Some(n) = explicit {
        crate::tool_reservations::for_tool(world, name, kind, n).map_err(CommandError::Invalid)?;
        if let Some((other, _)) = owners.iter().find(|(o, num)| *num == n && o != name) {
            return Err(CommandError::Invalid(format!(
                "tool '{name}': tool number T{n} is already '{other}''s"
            )));
        }
        if !owners.iter().any(|(o, _)| o == name) {
            owners.push((name.to_owned(), n));
        }
        return Ok(n);
    }
    let taken: Vec<u32> = owners.iter().map(|(_, n)| *n).collect();
    let n = crate::tool_reservations::allocate(world, name, kind, &taken).map_err(CommandError::Invalid)?;
    owners.push((name.to_owned(), n));
    Ok(n)
}
