//! #3898 (U7 of #3663): the tool library window's component state - the
//! "My library" view, the component draft, the stack draft of the open
//! tool, the cached component library - and the handlers of the
//! component actions. Each document change is one undoable app command
//! (#4201 `UpsertDocumentComponent`, `EditDocumentComponent`,
//! `DeleteDocumentComponent`, `SetToolAssembly`; #4233
//! `SyncDocumentComponent`). The user library file is not the document:
//! "Save to My library" and "Delete from My library" write the file
//! (`component_library::store`) and have no undo, as a machine file.
//! Frame cost (`crates/ui/AGENTS.md`): the library file is read when the
//! window opens or after a write of it, never on a still frame
//! (`library_loads`).
//! Provenance: our own view state and handlers over our own commands; no
//! HSMWorks material involved.

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use camcraft_app::commands::{
    DeleteDocumentComponent, EditDocumentComponent, SetToolAssembly, SyncDocumentComponent, UpsertDocumentComponent,
};
use camcraft_app::component::{ADAPTIVE_PORT, ComponentKind, CuttingItem, ToolComponent};
use camcraft_app::connection_code::ConnectionCode;
use camcraft_app::component_geometry::ComponentGeometry;
use camcraft_app::component_library::{self, BUILTIN_PREFIX, LibraryComponent, store};
use camcraft_app::holder_library::{self, HolderEntry};
use camcraft_app::schema::{AssemblyLink, DocumentComponent};
use camcraft_app::tool_assembly as ta;

use super::{ErrorScope, State};

/// The id space of the components a user makes in the window.
pub const USER_PREFIX: &str = "user:";

/// The component groups of the "My library" list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ComponentGroup {
    /// Spindle holders (an adaptive item with a taper on its machine side).
    Holders,
    /// Adapters, extensions, collets (the other adaptive items).
    Adapters,
    /// Indexable cutter bodies, turning holders, boring bars.
    Bodies,
    Inserts,
    /// Solid tools on a shank.
    SolidTools,
}

/// "My library": the group, the name filter, and the component whose
/// compatible parts the list shows (`compatible`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComponentsView {
    pub group: Option<ComponentGroup>,
    pub filter: String,
    /// The id of the component the "Compatible" filter is set on.
    pub compatible: Option<String>,
    /// The library row the action buttons act on.
    pub selected: Option<String>,
    /// The document row the action buttons act on.
    pub doc_selected: Option<String>,
}

/// The editor draft of one document component: what the user typed,
/// before Apply. `gauge` is `Some` for a holder whose built-in shape is
/// known; `code` is the ISO 1832 text of an insert.
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentDraft {
    pub id: String,
    pub name: String,
    pub code: String,
    pub gauge: Option<f64>,
}

/// The component stack draft of the tool open in the editor. `synced` is
/// the `doc_version` it was read at; `links` is what Apply stack lands.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StackDraft {
    pub tool: String,
    pub synced: u64,
    pub links: Vec<AssemblyLink>,
}

/// One read of the component library: the built-ins, then the user's
/// file (`all`), the file's path, and the reason it could not be read.
#[derive(Clone, Debug, Default)]
pub struct LoadedLibrary {
    /// The read's number on this thread: a cache keyed on it sees a new read.
    pub generation: u64,
    pub all: Vec<LibraryComponent>,
    pub path: Option<PathBuf>,
    pub error: Option<String>,
}

/// The component library as the window reads it: a folder override (the
/// tests' temp folder) and the cached read.
#[derive(Clone, Debug, Default)]
pub struct ComponentLib {
    pub dir: Option<PathBuf>,
    loaded: Option<Rc<LoadedLibrary>>,
}

thread_local! {
    static LOADS: Cell<u64> = const { Cell::new(0) };
}

/// The library file reads on this thread - a still frame adds none.
#[cfg(test)]
#[must_use]
pub fn library_loads() -> u64 {
    LOADS.with(Cell::get)
}

/// What the window asks of the document or the library file.
#[derive(Debug, Clone)]
pub enum ComponentAction {
    /// Put library component `id` into the document, linked to its
    /// library copy (one undo step).
    AddFromLibrary { id: String },
    /// A new user component in the document, made from the library or
    /// document component `id` under a new `user:` id, not linked.
    CopyAsUser { id: String },
    /// Replace a document component (the live update of its tools).
    Edit(Box<(ToolComponent, ComponentGeometry)>),
    /// Pull the library copy of document component `id`.
    Sync { id: String },
    DeleteFromDocument { id: String },
    /// Write document component `id` to the user library file, then link
    /// the document row to that copy.
    SaveToLibrary { id: String },
    DeleteFromLibrary { id: String },
    /// Land (`Some`) or clear (`None`, the tool keeps its records) the
    /// component stack of `tool`.
    SetStack { tool: String, links: Option<Vec<AssemblyLink>> },
}

impl State {
    /// The component library, read once and kept until a write of the
    /// file or a new window drops it ([`Self::reload_component_library`]).
    pub(crate) fn component_library(&mut self) -> Rc<LoadedLibrary> {
        if let Some(l) = &self.component_lib.loaded {
            return Rc::clone(l);
        }
        let generation = LOADS.with(|c| {
            c.set(c.get().saturating_add(1));
            c.get()
        });
        let path = self.component_lib.dir.clone().or_else(store::user_dir).map(|d| store::library_path(&d));
        let (user, error) = match &path {
            Some(p) => match store::load(p) {
                Ok(user) => (user, None),
                Err(e) => (Vec::new(), Some(format!("the user component library is not read: {e}"))),
            },
            None => (Vec::new(), Some("no folder for the user component library on this system".to_owned())),
        };
        let l = Rc::new(LoadedLibrary { generation, all: component_library::with_builtins(user), path, error });
        self.component_lib.loaded = Some(Rc::clone(&l));
        l
    }

    /// Read the library file again at the next use.
    pub(crate) fn reload_component_library(&mut self) {
        self.component_lib.loaded = None;
    }

    pub(crate) fn queue_component(&mut self, action: ComponentAction) {
        self.pending.push_back(super::CommandKind::Component(action));
    }

    /// The handler: a refusal lands on the Tool pane by name.
    pub(crate) fn run_component(&mut self, action: ComponentAction) {
        let done = self.history.done_len();
        let out = match action {
            ComponentAction::AddFromLibrary { id } => self.add_from_library(&id),
            ComponentAction::CopyAsUser { id } => self.copy_as_user(&id),
            ComponentAction::Edit(edit) => {
                let (component, geometry) = *edit;
                self.edit_component(component, geometry)
            }
            ComponentAction::Sync { id } => self.sync_component(&id),
            ComponentAction::DeleteFromDocument { id } => DeleteDocumentComponent::new(&self.world, &id)
                .map(|c| {
                    self.history.execute(&self.world, c);
                    format!("tool component {id} deleted from the document")
                })
                .map_err(|e| e.to_string()),
            ComponentAction::SaveToLibrary { id } => self.save_to_library(&id),
            ComponentAction::DeleteFromLibrary { id } => self.delete_from_library(&id),
            ComponentAction::SetStack { tool, links } => self.set_stack(&tool, links),
        };
        if self.history.done_len() != done {
            self.doc_version += 1;
        }
        match out {
            Ok(msg) => self.status = msg,
            Err(e) => self.refuse(ErrorScope::Tool, e),
        }
    }

    fn library_copy(&mut self, id: &str) -> Result<LibraryComponent, String> {
        let lib = self.component_library();
        lib.all
            .iter()
            .find(|c| c.component.id() == id)
            .cloned()
            .ok_or_else(|| format!("the component library holds no tool component '{id}'"))
    }

    /// Upsert, then the sync that records the link: one undo step.
    fn add_from_library(&mut self, id: &str) -> Result<String, String> {
        let entry = self.library_copy(id)?;
        if ta::component(&self.world, id).is_some() {
            return Err(format!("the document holds the tool component '{id}' already - use Sync from library"));
        }
        let add = UpsertDocumentComponent::new(&self.world, entry.component.clone(), entry.geometry.clone())
            .map_err(|e| e.to_string())?;
        self.history.begin_group(format!("Add tool component {id}"));
        self.history.execute(&self.world, add);
        let lib = self.component_library();
        let linked = SyncDocumentComponent::new(&self.world, id, &lib.all);
        let out = match linked {
            Ok(sync) => {
                self.history.execute(&self.world, sync);
                Ok(format!("tool component {id} added to the document"))
            }
            Err(e) => Ok(format!("tool component {id} added to the document, without its library link: {e}")),
        };
        let _ = self.history.commit_group();
        out
    }

    /// The first free id `user:<kind>:<name>` (then ` (2)`, ` (3)` ...).
    fn free_user_id(&mut self, name: &str, kind: &str) -> String {
        let lib = self.component_library();
        let taken = |id: &str| ta::component(&self.world, id).is_some() || lib.all.iter().any(|c| c.component.id() == id);
        let base = format!("{USER_PREFIX}{kind}:{name}");
        let mut id = base.clone();
        let mut n: u32 = 1;
        while taken(&id) {
            n = n.saturating_add(1);
            id = format!("{base} ({n})");
        }
        id
    }

    /// A document row copies whole - its cutter facts and running data
    /// too - under the new id, with no library link; a library entry has
    /// no such facts.
    fn copy_as_user(&mut self, id: &str) -> Result<String, String> {
        let row = match ta::component(&self.world, id) {
            Some(row) => row,
            None => {
                let e = self.library_copy(id)?;
                DocumentComponent::new(e.component, e.geometry)
            }
        };
        let kind = kind_word(&row.component);
        let new_id = self.free_user_id(row.component.name(), kind);
        let copy = ToolComponent::new(&new_id, row.component.name(), row.component.kind().clone()).map_err(|e| e.to_string())?;
        let add = UpsertDocumentComponent::with_row(&self.world, DocumentComponent { component: copy, library_ref: None, ..row })
            .map_err(|e| e.to_string())?;
        self.history.execute(&self.world, add);
        // The new copy opens in the window's component editor.
        if let Some(row) = ta::component(&self.world, &new_id)
            && let Some(v) = self.tool_library.as_mut()
        {
            v.component_edit = Some(draft_of(&row));
        }
        Ok(format!("user tool component {new_id} added to the document"))
    }

    fn edit_component(&mut self, component: ToolComponent, geometry: ComponentGeometry) -> Result<String, String> {
        let id = component.id().to_owned();
        let edit = EditDocumentComponent::new(&self.world, component, geometry).map_err(|e| e.to_string())?;
        let tools = edit.updated_tools();
        self.history.execute(&self.world, edit);
        Ok(updated(&format!("tool component {id} changed"), &tools))
    }

    fn sync_component(&mut self, id: &str) -> Result<String, String> {
        let lib = self.component_library();
        let sync = SyncDocumentComponent::new(&self.world, id, &lib.all).map_err(|e| e.to_string())?;
        let tools = sync.updated_tools();
        self.history.execute(&self.world, sync);
        Ok(updated(&format!("tool component {id} synced from the library"), &tools))
    }

    fn save_to_library(&mut self, id: &str) -> Result<String, String> {
        let Some(row) = ta::component(&self.world, id) else {
            return Err(format!("the document holds no tool component '{id}'"));
        };
        if component_library::is_builtin_id(id) {
            return Err(format!(
                "the tool component '{id}' is a built-in copy - make it a user component first (Copy as user component)"
            ));
        }
        let Some(path) = self.component_library().path.clone() else {
            return Err("no folder for the user component library on this system".to_owned());
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("the user component library folder is not made: {e}"))?;
        }
        let entry = LibraryComponent::from_row(row)?;
        store::put(&path, entry).map_err(|e| format!("not saved to My library: {e}"))?;
        self.reload_component_library();
        // The document row now links to the copy it was saved as.
        let lib = self.component_library();
        if let Ok(sync) = SyncDocumentComponent::new(&self.world, id, &lib.all) {
            self.history.execute(&self.world, sync);
        }
        Ok(format!("tool component {id} saved to My library"))
    }

    fn delete_from_library(&mut self, id: &str) -> Result<String, String> {
        let Some(path) = self.component_library().path.clone() else {
            return Err("no folder for the user component library on this system".to_owned());
        };
        store::delete(&path, id).map_err(|e| format!("not deleted from My library: {e}"))?;
        self.reload_component_library();
        Ok(format!("tool component {id} deleted from My library"))
    }

    fn set_stack(&mut self, tool: &str, links: Option<Vec<AssemblyLink>>) -> Result<String, String> {
        let clear = links.is_none();
        let cmd = match links {
            Some(links) => SetToolAssembly::new(&self.world, tool, links),
            None => SetToolAssembly::clear(&self.world, tool),
        }
        .map_err(|e| e.to_string())?;
        self.history.execute(&self.world, cmd);
        Ok(format!("tool {tool}: component stack {}", if clear { "cleared" } else { "set" }))
    }
}

/// "<what>; tools updated: A, B" (or "<what>" when no tool changed).
fn updated(what: &str, tools: &[String]) -> String {
    if tools.is_empty() { what.to_owned() } else { format!("{what}; tools updated: {}", tools.join(", ")) }
}

/// The group of a component in the "My library" list.
#[must_use]
pub fn group_of(c: &ToolComponent) -> ComponentGroup {
    match c.kind() {
        ComponentKind::CuttingItem(CuttingItem::Insert(_)) => ComponentGroup::Inserts,
        ComponentKind::CuttingItem(CuttingItem::Solid { .. }) => ComponentGroup::SolidTools,
        ComponentKind::ToolItem { .. } => ComponentGroup::Bodies,
        ComponentKind::AdaptiveItem { mcs, .. } => {
            if matches!(
                mcs,
                ConnectionCode::SteepTaper { .. } | ConnectionCode::Hsk { .. } | ConnectionCode::Psc { .. } | ConnectionCode::Km { .. }
            ) {
                ComponentGroup::Holders
            } else {
                ComponentGroup::Adapters
            }
        }
    }
}

/// The word of a component's kind in a new user id ("holder", "insert").
#[must_use]
pub fn kind_word(c: &ToolComponent) -> &'static str {
    match group_of(c) {
        ComponentGroup::Holders => "holder",
        ComponentGroup::Adapters => "adapter",
        ComponentGroup::Bodies => "body",
        ComponentGroup::Inserts => "insert",
        ComponentGroup::SolidTools => "tool",
    }
}

/// The built-in holder whose shape a document holder has: by its library
/// link, by its own id (`builtin:holder:<name>`, `user:holder:<name>`),
/// or by its name.
#[must_use]
pub fn holder_entry(row: &DocumentComponent) -> Option<HolderEntry> {
    if !matches!(row.component.kind(), ComponentKind::AdaptiveItem { .. }) {
        return None;
    }
    let strip = |id: &str| {
        let rest = id.strip_prefix(BUILTIN_PREFIX).or_else(|| id.strip_prefix(USER_PREFIX))?.strip_prefix("holder:")?;
        // A copy's " (2)" is not part of the holder's name.
        Some(rest.rsplit_once(" (").map_or(rest, |(n, _)| n).to_owned())
    };
    let names = [row.library_ref.as_ref().and_then(|r| strip(&r.id)), strip(row.component.id()), Some(row.component.name().to_owned())];
    names.into_iter().flatten().find_map(|n| holder_library::find(&n).ok())
}

/// The gauge length a holder row states: its workpiece port below its MCS.
pub fn gauge_of(row: &DocumentComponent) -> Option<f64> {
    row.geometry.port(ADAPTIVE_PORT).map(|f| -f.origin()[2]).filter(|g| g.is_finite() && *g > 0.0)
}

/// The draft of `row` as it stands in the document.
#[must_use]
pub fn draft_of(row: &DocumentComponent) -> ComponentDraft {
    let code = match row.component.kind() {
        ComponentKind::CuttingItem(CuttingItem::Insert(spec)) => spec.to_string(),
        _ => String::new(),
    };
    let gauge = holder_entry(row).and_then(|_| gauge_of(row));
    ComponentDraft { id: row.component.id().to_owned(), name: row.component.name().to_owned(), code, gauge }
}

