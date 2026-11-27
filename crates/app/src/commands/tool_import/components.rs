//! #3645: the tool components that an import lands with its tools, in
//! the same undo step. Our own command model.
//!
//! The id law: a library row whose id the document already holds is
//! reused when the whole row is the same (name, item, geometry, the
//! #3646 cutter facts and running data and the #3643 lathe body; only
//! the library link is left out, as for a link-only edit), and refuses
//! the whole import by the id when it differs. The rows of a tool that
//! the import skips as identical are checked by the same rule. An import
//! never changes or renames a document row: other tools may be built
//! from it, and a silent rename would hide which row a stack names.

use flecs_ecs::prelude::World;

use crate::commands::CommandError;
use crate::model;
use crate::schema::{DocumentComponent, ToolAssembly};
use crate::tool_assembly as ta;
use crate::tool_library::LibraryAssembly;

/// The table before and after an import that adds rows.
pub(super) type TableChange = (Vec<DocumentComponent>, Vec<DocumentComponent>);

/// The whole row except the library link: the comparison of a
/// link-only edit (`EditDocumentComponent`).
fn same_content(a: &DocumentComponent, b: &DocumentComponent) -> bool {
    DocumentComponent { library_ref: None, ..a.clone() } == DocumentComponent { library_ref: None, ..b.clone() }
}

fn differs(id: &str) -> CommandError {
    CommandError::Invalid(format!(
        "tool component '{id}' in the tool library is not the same as the document's tool component '{id}' - rename one of them; nothing imported"
    ))
}

/// The rows of a tool that the import skips as identical: each row that
/// the document holds by the same id must be the same, or the whole
/// import is refused by the id.
pub(super) fn check_skipped(world: &World, skipped: &LibraryAssembly) -> Result<(), CommandError> {
    let table = ta::components(world);
    for (_, row) in &skipped.rows {
        let id = row.component.id();
        if table.iter().any(|doc| doc.component.id() == id && !same_content(doc, row)) {
            return Err(differs(id));
        }
    }
    Ok(())
}

/// What [`plan`] gives: the table change (`None`: every row is in the
/// document already), and the ids added and reused (sorted).
pub(super) type TablePlan = (Option<TableChange>, Vec<String>, Vec<String>);

/// The table plan for the stacks that land.
pub(super) fn plan(world: &World, landing: &[&LibraryAssembly]) -> Result<TablePlan, CommandError> {
    let prev = ta::components(world);
    let mut next = prev.clone();
    let mut added: Vec<String> = Vec::new();
    let mut reused: Vec<String> = Vec::new();
    // The file's table order, so an export after the import is the same.
    let mut rows: Vec<&(usize, DocumentComponent)> = landing.iter().flat_map(|a| a.rows.iter()).collect();
    rows.sort_by_key(|(i, _)| *i);
    for (_, row) in rows {
        let id = row.component.id();
        if added.iter().any(|a| a == id) || reused.iter().any(|r| r == id) {
            continue;
        }
        match prev.iter().find(|r| r.component.id() == id) {
            Some(doc) if same_content(doc, row) => reused.push(id.to_owned()),
            Some(_) => return Err(differs(id)),
            None => {
                added.push(id.to_owned());
                next.push(row.clone());
            }
        }
    }
    added.sort();
    reused.sort();
    let change = if added.is_empty() { None } else { Some((prev, next)) };
    Ok((change, added, reused))
}

/// Land (`Some`) or clear (`None`) the stack of tool `name`.
pub(super) fn put_assembly(world: &World, name: &str, stack: Option<&ToolAssembly>) {
    if let Some(entity) = model::find_by_name(world, name) {
        ta::set_assembly(world, &entity, stack);
    }
}
