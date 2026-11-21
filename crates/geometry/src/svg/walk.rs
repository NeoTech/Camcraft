//! The SVG element tree walked into sketch primitives: groups and their transforms, `use`
//! references inside the file, layers from top-level groups, and the content that is skipped
//! with a note.
//! Provenance: the W3C SVG 1.1 structure chapter (g, defs, symbol, use, the `display`
//! property) and Inkscape's public `inkscape:label` layer convention; original implementation.

use super::emit::Sink;
use super::shapes::{self, Drawn};
use super::transform::{self, Affine};
use super::xml::{Node, Tree};
use super::{SvgError, invalid};
use crate::dxf::ImportSummary;

/// The deepest chain of `use` references followed.
const MAX_USE_DEPTH: usize = 16;
/// Bound the active call stack across both groups and reference expansion.
const MAX_ACTIVE_DEPTH: usize = 256;

/// Never drawn and never noted: definitions and document furniture.
const SILENT: &[&str] = &[
    "defs", "symbol", "metadata", "title", "desc", "namedview", "linearGradient", "radialGradient",
    "filter", "RDF", "perspective", "font", "font-face", "cursor",
];

/// The layer of elements outside any top-level group.
pub(super) const DEFAULT_LAYER: &str = "0";

pub(super) struct Walk<'a> {
    pub tree: &'a Tree,
    pub sink: Sink,
    pub summary: ImportSummary,
    pub max_entities: usize,
    pub max_cmds: usize,
    pub visits: usize,
}

fn hidden(node: &Node) -> bool {
    let style_none = node.attr("style").is_some_and(|s| {
        s.split(';').any(|decl| {
            decl.split_once(':').is_some_and(|(property, value)| {
                // CSS property names and keywords are case-insensitive. An importance
                // suffix must never make a hidden contour eligible for cutting.
                property.trim().eq_ignore_ascii_case("display")
                    && value.split('!').next().is_some_and(|keyword| keyword.trim().eq_ignore_ascii_case("none"))
            })
        })
    });
    style_none || node.attr("display").is_some_and(|d| d.trim() == "none")
}

/// Recognize unsupported effects in presentation attributes and inline declarations.
fn has_property(node: &Node, name: &str) -> bool {
    node.attr(name).is_some()
        || node.attr("style").is_some_and(|style| {
            style.split(';').any(|declaration| {
                declaration
                    .split_once(':')
                    .is_some_and(|(property, _)| property.trim().eq_ignore_ascii_case(name))
            })
        })
}

impl<'a> Walk<'a> {
    fn note(&mut self, kind: &str) -> Result<(), SvgError> {
        self.count()?;
        *self.summary.skipped.entry(kind.to_owned()).or_default() += 1;
        Ok(())
    }

    fn count(&mut self) -> Result<(), SvgError> {
        if self.summary.entities >= self.max_entities {
            return Err(invalid(format!(
                "more than {} elements: the file is larger than the import budget",
                self.max_entities
            )));
        }
        self.summary.entities += 1;
        Ok(())
    }

    fn node(&self, index: usize) -> Option<&'a Node> {
        self.tree.nodes.get(index)
    }

    /// Walk the children of the root element under the root transform.
    pub fn root(&mut self, root_map: Affine) -> Result<(), SvgError> {
        if let Some(root) = self.node(0) {
            if hidden(root) {
                return self.note("hidden");
            }
            for attr in ["clip-path", "mask"] {
                if has_property(root, attr) {
                    self.note(attr)?;
                }
            }
        }
        let children = self.node(0).map(|n| n.children.clone()).unwrap_or_default();
        for child in children {
            let layer = self.node(child).filter(|n| n.name == "g").map(|n| {
                n.attr("inkscape:label").or_else(|| n.attr("id")).unwrap_or(DEFAULT_LAYER).to_owned()
            });
            self.sink.layer = layer.unwrap_or_else(|| DEFAULT_LAYER.to_owned());
            self.element(child, root_map, 0, 0)?;
        }
        Ok(())
    }

    /// Draw element `index` whose parent maps user space through `parent`.
    fn element(&mut self, index: usize, parent: Affine, uses: usize, depth: usize) -> Result<(), SvgError> {
        if depth >= MAX_ACTIVE_DEPTH {
            return Err(invalid(format!(
                "active traversal depth exceeds {MAX_ACTIVE_DEPTH} elements across groups and references"
            )));
        }
        if self.visits >= self.max_entities {
            return Err(invalid(format!(
                "traversal visits more than {} elements: reference expansion exceeds the import budget",
                self.max_entities
            )));
        }
        self.visits += 1;
        let Some(node) = self.node(index) else { return Ok(()) };
        let name = node.name.clone();
        if SILENT.contains(&name.as_str()) {
            return Ok(());
        }
        if hidden(node) {
            return self.note("hidden");
        }
        let map = match node.attr("transform") {
            Some(t) => parent.then(transform::parse(t)?),
            None => parent,
        };
        for attr in ["clip-path", "mask"] {
            if has_property(node, attr) {
                self.note(attr)?;
            }
        }
        match name.as_str() {
            "g" | "a" => {
                for child in node.children.clone() {
                    self.element(child, map, uses, depth + 1)?;
                }
                Ok(())
            }
            "use" => self.reference(node, map, uses, depth),
            "path" | "rect" | "circle" | "ellipse" | "line" | "polyline" | "polygon" => {
                self.count()?;
                let saved = std::mem::replace(&mut self.sink.map, map);
                let drawn = shapes::draw(node, &mut self.sink, self.max_cmds);
                self.sink.map = saved;
                match drawn? {
                    Drawn::Shape => Ok(()),
                    Drawn::Empty => {
                        self.summary.entities -= 1;
                        self.note(&format!("{name} (empty)"))
                    }
                }
            }
            // Text, images, nested viewports, clipping and masking content, styles and the
            // like are not cut; each is counted so the caller can say what was left out.
            _ => self.note(&name),
        }
    }

    /// A `use` of an element in this file: drawn at its x, y; any other reference is noted.
    fn reference(&mut self, node: &'a Node, map: Affine, uses: usize, depth: usize) -> Result<(), SvgError> {
        let href = node.attr("href").or_else(|| node.attr("xlink:href")).unwrap_or("").trim();
        let tree = self.tree;
        let target = href.strip_prefix('#').and_then(|id| tree.nodes.iter().position(|n| n.attr("id") == Some(id)));
        let Some(target) = target else { return self.note("use") };
        if uses >= MAX_USE_DEPTH {
            return Err(invalid(format!("<use> references nest more than {MAX_USE_DEPTH} deep (a reference loop?)")));
        }
        let at = Affine::translate(
            super::number::user_length(node.attr("x").unwrap_or("0"), "<use> x")?,
            super::number::user_length(node.attr("y").unwrap_or("0"), "<use> y")?,
        );
        let map = map.then(at);
        match self.node(target) {
            // A symbol needs its own viewport, including the dimensions of this use.
            Some(sym) if sym.name == "symbol" => Err(invalid("<symbol> viewport references are not supported")),
            _ => self.element(target, map, uses + 1, depth + 1),
        }
    }
}
