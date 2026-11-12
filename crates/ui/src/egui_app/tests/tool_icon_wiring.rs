//! #661 section 4 wiring: every Tool library catalog row and every tool
//! picker item wears its kind's glyph. The rows and items draw through
//! `catalog_row_icon` and `picker_item_icon`, the pure choices pinned here,
//! so the test reads the same mapping the painted pixels come from.
//! Provenance: our own test; no HSMWorks or SolidCraft material involved.

use super::op_icon_table::every_tool_kind;
use crate::egui_app::icons::Icon;
use crate::egui_app::tool_icons::{catalog_row_icon, picker_item_icon, tool_icon};
use camcraft_app::tools::{self, Tool};

/// A catalog tool of each kind in the editor's kind numbering: the shipped
/// catalog's first tool with its kind swapped, so every field but the kind
/// is a real, valid tool.
fn tool_of_kind(kind: camcraft_app::tools::ToolKind) -> Tool {
    let (_, mut t) = tools::catalog().into_iter().next().expect("the shipped catalog is not empty");
    t.kind = kind;
    t
}

/// Every Tool library catalog row wears the glyph of its own kind, and the
/// shipped catalog's kinds are all in the kind list (no row falls outside it).
#[test]
fn every_catalog_row_wears_its_kind_glyph() {
    let kinds = every_tool_kind();
    for (name, ct) in tools::catalog() {
        assert!(kinds.contains(&ct.kind), "catalog row {name} has a kind the editor does not number");
        assert_eq!(catalog_row_icon(&ct), tool_icon(ct.kind), "catalog row {name} wears the wrong glyph");
    }
    for k in kinds {
        let row = tool_of_kind(k);
        let icon: Icon = catalog_row_icon(&row);
        assert_eq!(icon, tool_icon(k), "a catalog row of {k:?} wears {icon:?}");
    }
}

/// Every picker item of a document tool wears the glyph of that tool's kind,
/// driven from the kind list; an item naming no tool wears nothing.
#[test]
fn every_picker_item_wears_its_kind_glyph() {
    for k in every_tool_kind() {
        let tool = tool_of_kind(k);
        assert_eq!(picker_item_icon(Some(&tool)), Some(tool_icon(k)), "a picker item of {k:?} wears the wrong glyph");
    }
    assert_eq!(picker_item_icon(None), None, "a name with no tool must not wear a glyph");
}
