//! #661: the structural pin of the operation icon table, split out of
//! structural.rs (which sits at the 400-line law): every operation kind
//! wears its own SVG-mastered glyph.
//! Provenance: our own test; no HSMWorks material involved.

use super::*;

/// #661: every operation kind wears its own glyph - an `op-*` PNG drawn
/// from an SVG master in assets/ui/svg/ - and no two kinds share one (the
/// toolbar's Create cells and the tree's op rows tell kinds apart by it).
/// The pixels themselves are pinned against their masters by xtask's
/// `committed_icon_pngs_match_their_svg_masters`.
#[test]
fn every_op_kind_has_its_own_svg_mastered_icon() {
    use std::collections::HashSet;
    let svg = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/ui/svg");
    let mut seen = HashSet::new();
    for k in camcraft_app::operation::KindInfo::ALL {
        let icon = crate::egui_app::op_icons::op_icon(k.ty);
        assert!(seen.insert(icon), "{:?} shares {icon:?} with another kind", k.ty);
        let file = icon.file();
        assert!(file.starts_with("op-"), "{:?} wears the stand-in {file}.png", k.ty);
        assert!(svg.join(format!("{file}.svg")).is_file(), "{file}.png has no SVG master");
    }
    // No two slots of the table load the same PNG either.
    let files: HashSet<&str> = Icon::ALL.iter().map(|i| i.file()).collect();
    assert_eq!(files.len(), Icon::ALL.len(), "two icon slots share a PNG");
}
