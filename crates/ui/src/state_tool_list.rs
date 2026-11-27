//! #4335 (U9 of #3663): the tool list's rows and the search's hits, kept
//! by State; `tool_library_dialog::list_rows` builds them. Frame cost (`crates/ui/AGENTS.md`): the rows of the shown
//! source (Document or Catalog) are read once for each document version;
//! `rev` moves only when that read differs. The hits are built again only
//! when `rev`, the search text or the document's unit changes, and the
//! hits of "My library" only when the library read, the text or the unit
//! changes. A still frame reads and builds nothing (`ListBuilds`).
//! Provenance: our own caches over our own tool records; no HSMWorks
//! material involved.

use std::rc::Rc;

use camcraft_app::tools::Tool;
use camcraft_app::units::Unit;

use super::{ToolCategory, ToolSource};

/// One row of the tool list.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolRow {
    pub name: String,
    pub tool: Tool,
    /// The tool number (a document row only).
    pub number: Option<u32>,
    /// The library holder and the gauge length in it (a document row only).
    pub holder: Option<String>,
    pub gauge: Option<f64>,
    /// The document has a tool of this name.
    pub in_document: bool,
    /// The row's text for the search, lowered.
    pub(crate) text: String,
}

/// The rows of a source with the rows that match the search.
pub struct ToolHits {
    pub rows: Rc<Vec<ToolRow>>,
    /// The indexes of the rows that match the search, in the rows' order.
    pub hits: Vec<usize>,
    /// For each category, the count of the matching rows.
    pub counts: Vec<(ToolCategory, usize)>,
}

/// The reads and builds of the caches (the counter tests read them).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListBuilds {
    pub reads: u64,
    pub hits: u64,
    pub library_hits: u64,
}

pub(crate) type HitsKey = (u64, String, Unit);
pub(crate) type LibraryKey = (u64, String, Unit);

/// The caches of the tool list.
#[derive(Default)]
pub struct ToolListCache {
    pub(crate) read_at: Option<(u64, ToolSource)>,
    pub(crate) rev: u64,
    pub(crate) rows: Rc<Vec<ToolRow>>,
    pub(crate) hits: Option<(HitsKey, Rc<ToolHits>)>,
    pub(crate) library: Option<(LibraryKey, Rc<Vec<usize>>)>,
    pub(crate) builds: ListBuilds,
}

impl ToolListCache {
    /// The reads and builds so far.
    #[must_use]
    pub fn builds(&self) -> ListBuilds {
        self.builds
    }
}

