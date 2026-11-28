//! #4335 (U9 of #3663): builds the tool list's rows and the search's hits
//! into State's cache (`state_tool_list`). The rows of the shown source are
//! read once for each document version; the hits are built again only when
//! the rows, the search text or the unit change.
//! Provenance: our own caches over our own tool records; no HSMWorks
//! material involved.

use std::rc::Rc;

use camcraft_app::component::{ComponentKind, CuttingItem};
use camcraft_app::tools::{self, Tool};
use camcraft_app::{model, tool_body, tool_numbers};

use super::super::browser_tree::kind_name;
use super::categories;
use super::components::{kind_text, sides};
use super::search::{Facts, Query, haystack};
use crate::state::{LoadedLibrary, State, ToolHits, ToolRow, ToolSource};

/// A row with its search text.
fn row(name: String, tool: Tool, number: Option<u32>, holder: Option<(String, f64)>, in_document: bool) -> ToolRow {
    let (holder, gauge) = match holder {
        Some((h, g)) => (Some(h), g.is_finite().then_some(g)),
        None => (None, None),
    };
    let c = &tool.catalog;
    let text = haystack([
        name.as_str(),
        kind_name(tool.kind),
        categories::label(categories::category_of(tool.kind)),
        holder.as_deref().unwrap_or_default(),
        c.part_number.as_deref().unwrap_or_default(),
        c.vendor.as_deref().unwrap_or_default(),
    ]);
    ToolRow { name, tool, number, holder, gauge, in_document, text }
}

impl State {
    /// The rows of `source` as the document is now.
    fn read_tool_rows(&self, source: ToolSource) -> Vec<ToolRow> {
        match source {
            ToolSource::Document => {
                let numbers = tool_numbers::numbers(&self.world).unwrap_or_default();
                self.tool_names()
                    .into_iter()
                    .filter_map(|name| {
                        let e = model::find_by_name(&self.world, &name)?;
                        let tool = model::tool_of(&self.world, &e)?;
                        let number = numbers.iter().find(|(n, _)| *n == name).map(|(_, t)| *t);
                        let holder = tool_body::body_of(&self.world, &e).and_then(|b| b.library).map(|r| (r.entry, r.gauge_len + r.projection));
                        Some(row(name, tool, number, holder, true))
                    })
                    .collect()
            }
            ToolSource::Catalog => tools::catalog()
                .into_iter()
                .filter(|(_, t)| crate::machine_modes::tool_kind_offered(&self.world, t.kind))
                .map(|(n, t)| {
                    let in_doc = model::find_by_name(&self.world, n).is_some();
                    row(n.to_owned(), t, None, None, in_doc)
                })
                .collect(),
            ToolSource::Library => Vec::new(),
        }
    }

    /// The rows of `source` and the rows that match `query`.
    pub(crate) fn tool_hits(&mut self, source: ToolSource, query: &str) -> Rc<ToolHits> {
        let at = (self.doc_version, source);
        if self.tool_list.read_at != Some(at) {
            let rows = self.read_tool_rows(source);
            let c = &mut self.tool_list;
            c.builds.reads = c.builds.reads.saturating_add(1);
            c.read_at = Some(at);
            if *c.rows != rows {
                c.rows = Rc::new(rows);
                c.rev = c.rev.saturating_add(1);
            }
        }
        let unit = self.units();
        let c = &mut self.tool_list;
        let key = (c.rev, query.to_owned(), unit);
        if let Some((k, h)) = &c.hits
            && *k == key
        {
            return Rc::clone(h);
        }
        c.builds.hits = c.builds.hits.saturating_add(1);
        let q = Query::parse(query);
        let hits: Vec<usize> = c
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                if q.is_empty() {
                    return true;
                }
                let d = [r.tool.diameter, r.tool.shank_d];
                q.matches(&Facts { text: &r.text, diameters_mm: &d, number: r.number }, unit)
            })
            .map(|(i, _)| i)
            .collect();
        let counts = categories::GROUPS
            .iter()
            .flat_map(|(_, cats)| cats.iter())
            .map(|cat| (*cat, hits.iter().filter(|i| c.rows.get(**i).is_some_and(|r| categories::category_of(r.tool.kind) == *cat)).count()))
            .collect();
        let h = Rc::new(ToolHits { rows: Rc::clone(&c.rows), hits, counts });
        c.hits = Some((key, Rc::clone(&h)));
        h
    }

    /// The indexes of the library components that match `query`.
    pub(crate) fn component_hits(&mut self, lib: &LoadedLibrary, query: &str) -> Rc<Vec<usize>> {
        let unit = self.units();
        let c = &mut self.tool_list;
        let key = (lib.generation, query.to_owned(), unit);
        if let Some((k, h)) = &c.library
            && *k == key
        {
            return Rc::clone(h);
        }
        c.builds.library_hits = c.builds.library_hits.saturating_add(1);
        let q = Query::parse(query);
        let hits: Vec<usize> = lib
            .all
            .iter()
            .enumerate()
            .filter(|(_, l)| {
                if q.is_empty() {
                    return true;
                }
                let comp = &l.component;
                let (machine, tool) = sides(comp);
                let text = haystack([comp.name(), comp.id(), kind_text(comp), machine.as_str(), tool.as_str()]);
                let d = match comp.kind() {
                    ComponentKind::CuttingItem(CuttingItem::Solid { tool, .. }) => vec![tool.diameter, tool.shank_d],
                    _ => Vec::new(),
                };
                q.matches(&Facts { text: &text, diameters_mm: &d, number: None }, unit)
            })
            .map(|(i, _)| i)
            .collect();
        let h = Rc::new(hits);
        c.library = Some((key, Rc::clone(&h)));
        h
    }
}
