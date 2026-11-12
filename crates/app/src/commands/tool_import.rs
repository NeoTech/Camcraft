//! #300: import a tool library file as ONE undo unit. The library was
//! already checked whole by `tool_library::parse_tool_library`; here each
//! tool meets the document. The duplicate law: a name the document
//! already uses is never overwritten - an identical tool of that name is
//! skipped (it is already here), a different one comes in renamed with
//! the first free ` (2)`, ` (3)`... suffix, so an operation holding the
//! old tool keeps exactly the tool it had.
//!
//! #889: the import takes the holders choice as a parameter
//! ([`ImportTools::with_holders`]). On, each tool lands with the body
//! its file carries (judged again here against the tool); off, every
//! tool lands bare - its holder cylinder only, today's import. Identity
//! for the skip: a library tool that lands with a body is the document's
//! tool only when record AND body match (the DTO the file writes, so
//! "identical" means the bytes a save would write); one that lands bare
//! matches on the record alone, as before - a bodyless file never
//! duplicates a document tool the user has since fitted a holder to.
//!
//! Provenance: our own command model (`AddTool`'s snapshot-free add and
//! remove-by-name inverse, batched); no vendor material consulted.

use flecs_ecs::prelude::World;

use crate::commands::tool::reserved;
use crate::commands::{Command, CommandError};
use crate::model;
use crate::project::ToolBodyDto;
use crate::schema::{ToolBody, ToolDef};
use crate::tool_body;
use crate::tool_library::LibraryTool;
use crate::tools::Tool;

/// What an import did with each library tool, for the status line.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImportReport {
    /// Tools added under their own name.
    pub added: Vec<String>,
    /// `(library name, name it was added as)` for a name already taken.
    pub renamed: Vec<(String, String)>,
    /// Library names skipped: the document holds that exact tool already.
    pub skipped: Vec<String>,
    /// #889: tools that landed with their file's holder body.
    pub holders: usize,
    /// #889: bodies the file carried that the import left out (holders off).
    pub holders_left_out: usize,
}

impl ImportReport {
    /// One line: "imported 3 tools; 1 renamed (A -> A (2)); 2 already in the document (B, C)".
    #[must_use]
    pub fn summary(&self) -> String {
        let n = self.added.len().saturating_add(self.renamed.len());
        let mut s = format!("imported {n} tool{}", if n == 1 { "" } else { "s" });
        if !self.renamed.is_empty() {
            let list: Vec<String> = self.renamed.iter().map(|(a, b)| format!("{a} -> {b}")).collect();
            s.push_str(&format!("; {} renamed ({})", self.renamed.len(), list.join(", ")));
        }
        if !self.skipped.is_empty() {
            s.push_str(&format!("; {} already in the document ({})", self.skipped.len(), self.skipped.join(", ")));
        }
        if self.holders > 0 {
            s.push_str(&format!("; {} with {} holder", self.holders, if self.holders == 1 { "its" } else { "their" }));
        }
        if self.holders_left_out > 0 {
            s.push_str(&format!(
                "; {} holder{} in the file not imported",
                self.holders_left_out,
                if self.holders_left_out == 1 { "" } else { "s" }
            ));
        }
        s
    }
}

/// Add every new tool of a library in one step; undo removes exactly
/// those tools and nothing else.
pub struct ImportTools {
    tools: Vec<(String, Tool, Option<ToolBody>)>,
    report: ImportReport,
}

/// The body as a save writes it - the identity the skip compares.
fn body_key(b: Option<&ToolBody>) -> Option<ToolBodyDto> {
    b.map(ToolBodyDto::from_body)
}

impl ImportTools {
    /// Plan the import with the file's holders taken (the default, and
    /// a no-op for a file without bodies).
    pub fn new(world: &World, library: Vec<LibraryTool>) -> Result<Self, CommandError> {
        Self::with_holders(world, library, true)
    }

    /// Plan the import against the document; `holders` takes each tool's
    /// body from the file (`true`) or lands every tool bare (`false`).
    /// Refuses (nothing applied) a library naming setup furniture, a tool
    /// `is_sane` refuses, a body that no longer fits its tool, or a
    /// library whose every tool is already in the document.
    pub fn with_holders(world: &World, library: Vec<LibraryTool>, holders: bool) -> Result<Self, CommandError> {
        let mut tools: Vec<(String, Tool, Option<ToolBody>)> = Vec::with_capacity(library.len());
        let mut report = ImportReport::default();
        for LibraryTool { name, tool, body } in library {
            if let Some(err) = reserved(world, &name) {
                return Err(err);
            }
            if !tool.is_sane() {
                return Err(CommandError::Invalid(format!(
                    "tool '{name}': parameters must be finite and positive"
                )));
            }
            if tools.iter().any(|(n, _, _)| *n == name) {
                return Err(CommandError::Invalid(format!("tool '{name}' appears twice in the library")));
            }
            let body = match body {
                Some(b) if holders => {
                    // Judged again at the command, as SetToolBody does.
                    tool_body::fits(&tool, &b).map_err(|e| CommandError::Invalid(format!("tool '{name}': {e}")))?;
                    Some(b)
                }
                Some(_) => {
                    report.holders_left_out = report.holders_left_out.saturating_add(1);
                    None
                }
                None => None,
            };
            let taken = |n: &str| model::find_by_name(world, n).is_some() || tools.iter().any(|(t, _, _)| t == n);
            let same = |e: &flecs_ecs::prelude::Entity| {
                model::tool_of(world, e).as_ref() == Some(&tool)
                    && (body.is_none() || body_key(tool_body::body_of(world, e).as_ref()) == body_key(body.as_ref()))
            };
            let lands = match model::find_by_name(world, &name) {
                None if !taken(&name) => {
                    report.added.push(name.clone());
                    Some(name)
                }
                Some(e) if same(&e) => {
                    report.skipped.push(name);
                    None
                }
                _ => {
                    let Some(free) = (2..=9999u32).map(|i| format!("{name} ({i})")).find(|c| !taken(c)) else {
                        return Err(CommandError::NameConflict(name));
                    };
                    report.renamed.push((name, free.clone()));
                    Some(free)
                }
            };
            if let Some(n) = lands {
                if body.is_some() {
                    report.holders = report.holders.saturating_add(1);
                }
                tools.push((n, tool, body));
            }
        }
        if tools.is_empty() {
            return Err(CommandError::Invalid(
                "nothing to import: every tool in the library is already in the document".into(),
            ));
        }
        Ok(ImportTools { tools, report })
    }

    #[must_use]
    pub fn report(&self) -> &ImportReport {
        &self.report
    }
}

impl Command for ImportTools {
    fn apply(&self, world: &World) {
        for (name, tool, body) in &self.tools {
            let entity = match model::find_by_name(world, name) {
                Some(entity) => entity,
                None => model::spawn(world, name),
            };
            entity.entity_view(world).set(ToolDef { tool: tool.clone(), number: None });
            tool_body::set_body(world, &entity, body.as_ref());
        }
    }

    fn invert(&self, world: &World) {
        for (name, _, _) in &self.tools {
            if let Some(entity) = model::find_by_name(world, name) {
                model::remove(world, &entity);
            }
        }
    }
}
