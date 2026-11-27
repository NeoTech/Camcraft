//! #3645: the user tool component library - the `camcraft-components/1`
//! file format, the built-in read-only components (the holder library
//! and the starter tool catalog), and the sync status of a document
//! component against the library. The file and its folder are in
//! [`store`]; the undoable pull into a document is
//! `crate::commands::SyncDocumentComponent`.
//!
//! The laws here:
//! - **All or nothing.** A library file is read whole: a bad envelope,
//!   a refused row (named by its id), an id twice, a library link on a
//!   row, or an id in the built-in space refuses the whole file.
//! - **Built-ins are read-only.** Their ids start with [`BUILTIN_PREFIX`];
//!   a user file can not hold such an id, so a save never shadows one.
//! - **A link is never applied silently.** [`sync_status`] compares a
//!   document row with its library copy by content hash and says so; only
//!   the explicit sync command changes the document.
//!
//! Provenance: our own format over our own project DTOs
//! (`project::component_wire`); no vendor library format consulted.

pub mod store;

use serde_json::{Value, json};

use camcraft_tools::component::{ComponentKind, CuttingItem, ToolComponent, ADAPTIVE_PORT};
use camcraft_tools::component_geometry::ComponentGeometry;
use camcraft_tools::connection_code::ConnectionCode;
use camcraft_tools::holder_library::{self, Clamp};

use crate::project::{component_from_dto, component_to_dto, content_hash, ComponentDto};
use crate::schema::DocumentComponent;

/// The envelope's `format` tag.
pub const FORMAT: &str = "camcraft-components";
/// The envelope version this build reads and writes.
pub const VERSION: u64 = 1;
/// The id space of the built-in components.
pub const BUILTIN_PREFIX: &str = "builtin:";

/// One component of a library.
#[derive(Clone, Debug, PartialEq)]
pub struct LibraryComponent {
    pub component: ToolComponent,
    pub geometry: ComponentGeometry,
}

impl LibraryComponent {
    /// The content hash a document link records (`project::content_hash`).
    #[must_use]
    pub fn content_hash(&self) -> String {
        content_hash(&self.component, &self.geometry)
    }

    /// The library copy of a document row (its library link dropped).
    /// Refused by name: a row with #3646 cutter facts or running data,
    /// or a #3643 lathe body, which a library component does not hold -
    /// a save never loses them without a word.
    pub fn from_row(row: DocumentComponent) -> Result<Self, String> {
        if row.milling.is_some() || row.running.is_some() || row.lathe_body.is_some() {
            return Err(format!(
                "tool component '{}' holds cutter facts, running data or a lathe body, which the component library does not hold",
                row.component.id()
            ));
        }
        Ok(Self { component: row.component, geometry: row.geometry })
    }

    /// Is this a built-in, read-only component?
    #[must_use]
    pub fn is_builtin(&self) -> bool {
        is_builtin_id(self.component.id())
    }
}

/// Is `id` in the built-in id space?
#[must_use]
pub fn is_builtin_id(id: &str) -> bool {
    id.trim().starts_with(BUILTIN_PREFIX)
}

/// Read a library file's text, whole (see the module laws).
pub fn parse(text: &str) -> Result<Vec<LibraryComponent>, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| format!("not a component library (bad JSON: {e})"))?;
    let Value::Object(o) = &root else {
        return Err("not a component library: expected an object".into());
    };
    match o.get("format").and_then(Value::as_str) {
        Some(FORMAT) => {}
        Some(other) => return Err(format!("not a component library: format is '{other}', not '{FORMAT}'")),
        None => return Err(format!("not a component library: no \"format\": \"{FORMAT}\"")),
    }
    match o.get("version").and_then(Value::as_u64) {
        Some(VERSION) => {}
        Some(v) => return Err(format!("component library version {v} is not supported (this build reads {VERSION})")),
        None => return Err("not a component library: no \"version\"".into()),
    }
    match o.get("units").and_then(Value::as_str) {
        Some("mm") => {}
        Some(u) => return Err(format!("component library units '{u}' are not supported (lengths are mm)")),
        None => return Err("not a component library: no \"units\": \"mm\"".into()),
    }
    let rows = o.get("components").and_then(Value::as_array).ok_or("not a component library: no \"components\" array")?;
    let mut out: Vec<LibraryComponent> = Vec::with_capacity(rows.len());
    for (i, raw) in rows.iter().enumerate() {
        let n = i.saturating_add(1);
        let label = raw.get("id").and_then(Value::as_str).map_or_else(|| format!("#{n}"), |s| format!("'{}'", s.trim()));
        let dto: ComponentDto =
            serde_json::from_value(raw.clone()).map_err(|e| format!("tool component {label} is malformed: {e}"))?;
        if dto.library_ref.is_some() {
            return Err(format!("tool component {label}: a library component has no library link"));
        }
        if dto.milling.is_some() || dto.running.is_some() || dto.lathe_body.is_some() {
            return Err(format!(
                "tool component {label}: a library component holds no cutter facts, no running data and no lathe body"
            ));
        }
        if is_builtin_id(&dto.id) {
            return Err(format!("tool component {label}: the id space '{BUILTIN_PREFIX}' is for the built-in components"));
        }
        let row = component_from_dto(&dto)?;
        if out.iter().any(|c| c.component.id() == row.component.id()) {
            return Err(format!("tool component '{}' is in the library twice", row.component.id()));
        }
        out.push(LibraryComponent { component: row.component, geometry: row.geometry });
    }
    Ok(out)
}

/// The library file's text (pretty, so a person can read it).
pub fn to_text(entries: &[LibraryComponent]) -> Result<String, String> {
    let rows: Vec<ComponentDto> = entries
        .iter()
        .map(|e| component_to_dto(&DocumentComponent::new(e.component.clone(), e.geometry.clone())))
        .collect();
    let doc = json!({ "format": FORMAT, "version": VERSION, "units": "mm", "components": rows });
    serde_json::to_string_pretty(&doc).map_err(|e| format!("component library not written: {e}"))
}

/// The holder library's entries that have a taper, as adaptive items
/// (machine side the taper, workpiece side the collet or the bore). A
/// standalone straight-shank chuck states no shank size, so it is not
/// a built-in component.
fn builtin_holders() -> Vec<LibraryComponent> {
    let mut out = Vec::new();
    for entry in holder_library::library() {
        let Some(taper) = entry.taper else { continue };
        let csw = match entry.clamp {
            Clamp::Er(er) => ConnectionCode::parse(er.name),
            Clamp::Weldon { bore, .. } => ConnectionCode::parse(&format!("WELDON {bore}")),
        };
        let (Ok(mcs), Ok(csw)) = (ConnectionCode::parse(taper.name), csw) else { continue };
        let name = entry.name();
        let id = format!("{BUILTIN_PREFIX}holder:{name}");
        let Ok(component) = ToolComponent::new(&id, &name, ComponentKind::AdaptiveItem { mcs, csw }) else { continue };
        let Ok(geometry) = ComponentGeometry::holder(&entry, ADAPTIVE_PORT, entry.gauge_len) else { continue };
        out.push(LibraryComponent { component, geometry });
    }
    out
}

/// The starter catalog's tools, as solid tools on a plain shank.
fn builtin_tools() -> Vec<LibraryComponent> {
    let mut out = Vec::new();
    for (name, tool) in camcraft_tools::tools::catalog() {
        let mcs = ConnectionCode::Cylindrical { d: tool.shank_d };
        let id = format!("{BUILTIN_PREFIX}tool:{name}");
        let kind = ComponentKind::CuttingItem(CuttingItem::Solid { tool: Box::new(tool), mcs });
        let Ok(component) = ToolComponent::new(&id, name, kind) else { continue };
        out.push(LibraryComponent { component, geometry: ComponentGeometry::default() });
    }
    out
}

/// Every built-in, read-only component: the holders, then the tools.
#[must_use]
pub fn builtins() -> Vec<LibraryComponent> {
    let mut out = builtin_holders();
    out.extend(builtin_tools());
    out
}

/// The library a document syncs from: the built-ins, then the user's
/// own components.
#[must_use]
pub fn with_builtins(user: Vec<LibraryComponent>) -> Vec<LibraryComponent> {
    let mut out = builtins();
    out.extend(user);
    out
}

/// How a document component stands against the library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncStatus {
    /// The row has no library link.
    NotLinked,
    /// The library copy has the content of the document copy.
    UpToDate,
    /// The library copy differs from the document copy (a newer library
    /// copy, or a document edit). `library_hash` is the library copy's.
    UpdateAvailable { library_hash: String },
    /// The library holds no component with the linked id.
    MissingFromLibrary,
}

/// The sync status of `row` against `library`.
#[must_use]
pub fn sync_status(row: &DocumentComponent, library: &[LibraryComponent]) -> SyncStatus {
    let Some(link) = &row.library_ref else {
        return SyncStatus::NotLinked;
    };
    let Some(entry) = library.iter().find(|e| e.component.id() == link.id) else {
        return SyncStatus::MissingFromLibrary;
    };
    let library_hash = entry.content_hash();
    if library_hash == content_hash(&row.component, &row.geometry) {
        SyncStatus::UpToDate
    } else {
        SyncStatus::UpdateAvailable { library_hash }
    }
}
