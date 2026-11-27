//! #4389: holder files - the document's user holders written to, and
//! read from, a `camcraft-components/1` file (`component_library`). The
//! export writes each holder of the component table that is not built
//! in. The import reads the file whole and adds each holder as a user
//! holder in one undo step; an id that the document or the library holds
//! already takes a new `user:holder:<name>` id, as "Copy as user holder"
//! does. A file that is not correct, or a row that is not a holder, is
//! refused by name and the document does not change (AGENTS.md Prime
//! Directive: no partial import).
//! Provenance: our own format and our own snapshot-and-restore command
//! model; no vendor holder format consulted.

use flecs_ecs::prelude::World;

use camcraft_tools::component::{ComponentKind, ToolComponent};
use camcraft_tools::connection_code::ConnectionCode;

use crate::commands::tool_assembly::row_refusal;
use crate::commands::{Command, CommandError};
use crate::component_library::{self, LibraryComponent, is_builtin_id};
use crate::schema::DocumentComponent;
use crate::tool_assembly::{components, table_set};

/// The id space of a user holder that the import names.
pub const USER_HOLDER_PREFIX: &str = "user:holder:";

/// Is `c` a spindle holder: an adaptive item whose machine side is a
/// spindle taper (steep taper, HSK, PSC or KM)? The Holders page lists
/// the same set.
#[must_use]
pub fn is_holder(c: &ToolComponent) -> bool {
    matches!(
        c.kind(),
        ComponentKind::AdaptiveItem {
            mcs: ConnectionCode::SteepTaper { .. } | ConnectionCode::Hsk { .. } | ConnectionCode::Psc { .. } | ConnectionCode::Km { .. },
            ..
        }
    )
}

/// The user holders of the document, in table order: each holder row
/// whose id is not in the built-in id space.
#[must_use]
pub fn user_holders(world: &World) -> Vec<DocumentComponent> {
    components(world).into_iter().filter(|r| is_holder(&r.component) && !is_builtin_id(r.component.id())).collect()
}

/// The holder file text of the document's user holders. Refused by name:
/// a document with no user holder, and a row that the file format can
/// not hold (`LibraryComponent::from_row`).
pub fn export_holders(world: &World) -> Result<String, String> {
    let rows = user_holders(world);
    if rows.is_empty() {
        return Err("the document holds no user holder to export".to_owned());
    }
    let entries = rows.into_iter().map(LibraryComponent::from_row).collect::<Result<Vec<_>, _>>()?;
    component_library::to_text(&entries)
}

/// One imported holder: its id in the file and its id in the document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedHolder {
    pub file_id: String,
    pub id: String,
}

/// Add the holders of a holder file to the component table, in one undo
/// step. Checked whole when made: nothing enters the history on a refusal.
pub struct ImportHolders {
    prev: Vec<DocumentComponent>,
    next: Vec<DocumentComponent>,
    imported: Vec<ImportedHolder>,
}

impl ImportHolders {
    /// Plan the import of holder file `text`. `reserved` are the ids that
    /// a new id must not take besides the document's (the component
    /// library's). Refused by name: every `component_library::parse`
    /// refusal, a file with no component, a component that is not a
    /// holder, and every component table row law.
    pub fn from_text(world: &World, text: &str, reserved: &[String]) -> Result<Self, CommandError> {
        let entries = component_library::parse(text).map_err(|e| CommandError::Invalid(format!("not a holder file: {e}")))?;
        if entries.is_empty() {
            return Err(CommandError::Invalid("the holder file holds no holder".to_owned()));
        }
        let prev = components(world);
        let mut next = prev.clone();
        let mut imported = Vec::with_capacity(entries.len());
        for entry in entries {
            let file_id = entry.component.id().to_owned();
            if !is_holder(&entry.component) {
                return Err(CommandError::Invalid(format!(
                    "tool component '{file_id}' in the holder file is not a holder (a spindle taper on its machine side)"
                )));
            }
            let taken = |id: &str| next.iter().any(|r| r.component.id() == id) || reserved.iter().any(|r| r == id);
            let id = if taken(&file_id) { free_id(entry.component.name(), taken) } else { file_id.clone() };
            let component = ToolComponent::new(&id, entry.component.name(), entry.component.kind().clone())
                .map_err(|e| CommandError::Invalid(format!("tool component '{file_id}': {e}")))?;
            let row = DocumentComponent::new(component, entry.geometry);
            if let Some(why) = row_refusal(&row) {
                return Err(CommandError::Invalid(why));
            }
            next.push(row);
            imported.push(ImportedHolder { file_id, id });
        }
        Ok(Self { prev, next, imported })
    }

    /// The holders that the import adds, in file order.
    #[must_use]
    pub fn imported(&self) -> &[ImportedHolder] {
        &self.imported
    }

    /// "N holders imported" and the ids that changed.
    #[must_use]
    pub fn summary(&self) -> String {
        let n = self.imported.len();
        let mut s = format!("{n} holder{} imported as user holders", if n == 1 { "" } else { "s" });
        let renamed: Vec<String> =
            self.imported.iter().filter(|i| i.file_id != i.id).map(|i| format!("'{}' as '{}'", i.file_id, i.id)).collect();
        if !renamed.is_empty() {
            s.push_str(&format!("; new ids: {}", renamed.join(", ")));
        }
        s
    }
}

/// The first free id `user:holder:<name>`, then ` (2)`, ` (3)` ...
fn free_id(name: &str, taken: impl Fn(&str) -> bool) -> String {
    let base = format!("{USER_HOLDER_PREFIX}{name}");
    let mut id = base.clone();
    let mut n: u32 = 1;
    while taken(&id) {
        n = n.saturating_add(1);
        id = format!("{base} ({n})");
    }
    id
}

impl Command for ImportHolders {
    fn apply(&self, world: &World) {
        table_set(world, self.next.clone());
    }

    fn invert(&self, world: &World) {
        table_set(world, self.prev.clone());
    }
}
