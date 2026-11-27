//! #3645: pull a user library copy of a tool component into the
//! document - the explicit, undoable sync. The library copy replaces the
//! document row (its id stays the document's), the row records the link
//! (`LibraryRef`: the library id and the copy's content hash), and every
//! tool built from the row is derived again, all in one undo step
//! through the edit law of `EditDocumentComponent`. Operations go Stale
//! by their stamps. Our own command model.

use flecs_ecs::prelude::World;

use camcraft_tools::component::ToolComponent;

use crate::commands::tool_assembly::EditDocumentComponent;
use crate::commands::{Command, CommandError};
use crate::component_library::LibraryComponent;
use crate::schema::{DocumentComponent, LibraryRef};
use crate::tool_assembly as ta;

/// Replace the document component `id` with its library copy.
pub struct SyncDocumentComponent(EditDocumentComponent);

impl SyncDocumentComponent {
    /// The library copy is the one the row links to, or (no link yet)
    /// the one with the row's id. Refused by name: an id the document
    /// does not hold, a copy the library does not hold, a row that is
    /// already the linked copy, and each refusal of the edit law (a tool
    /// that the new copy cannot build).
    pub fn new(world: &World, id: &str, library: &[LibraryComponent]) -> Result<Self, CommandError> {
        let id = id.trim();
        let Some(row) = ta::component(world, id) else {
            return Err(CommandError::Invalid(format!("the document holds no tool component '{id}'")));
        };
        let lib_id = row.library_ref.as_ref().map_or(id, |r| r.id.as_str());
        let Some(entry) = library.iter().find(|e| e.component.id() == lib_id) else {
            return Err(CommandError::Invalid(format!(
                "the component library holds no tool component '{lib_id}' to sync '{id}' from"
            )));
        };
        let content_hash = entry.content_hash();
        let component = ToolComponent::new(id, entry.component.name(), entry.component.kind().clone())
            .map_err(|e| CommandError::Invalid(e.to_string()))?;
        let next = DocumentComponent {
            component,
            geometry: entry.geometry.clone(),
            library_ref: Some(LibraryRef { id: lib_id.to_owned(), content_hash }),
            // #3646 / #3643: a library copy holds no cutter facts, no
            // running data and no lathe body; the row keeps its own, as a
            // hand edit does.
            milling: row.milling,
            running: row.running.clone(),
            lathe_body: row.lathe_body.clone(),
        };
        if next == row {
            return Err(CommandError::Invalid(format!("the tool component '{id}' is already up to date with the library")));
        }
        EditDocumentComponent::with_row(world, next).map(Self)
    }

    /// The tools whose records this sync changes (sorted).
    #[must_use]
    pub fn updated_tools(&self) -> Vec<String> {
        self.0.updated_tools()
    }
}

impl Command for SyncDocumentComponent {
    fn apply(&self, world: &World) {
        self.0.apply(world);
    }

    fn invert(&self, world: &World) {
        self.0.invert(world);
    }
}
