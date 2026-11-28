//! The theme-file seam of the shell: the `camcraft-theme/1` reader, its
//! caps, the discovery walk and the parse cache live in the
//! `camcraft-ui-kit` crate (#4394) and are re-exported here, so every
//! `egui_app::theme_files::...` path keeps working. The kit cannot see the
//! document, so this module keeps the one link to it: `resolve` maps the
//! project's `ThemeRef` onto the kit's `ThemeSource`.
//! Provenance: our own module (the M6.1 B/C theme files, split for the UI
//! kit crate of #4245); no HSMWorks material.

use std::path::Path;

use camcraft_app::schema::ThemeRef;

pub use camcraft_ui_kit::theme_files::*;

use super::theme::Tokens;

/// Resolve a document's theme reference into the table to wear, and the
/// notes gathered on the way. The kit's `resolve_source` does the work and
/// owns the refusals; this is only the map from the document's words to
/// the kit's.
pub fn resolve(cache: &mut Cache, reference: &ThemeRef) -> Result<(Tokens, Vec<String>), String> {
    let source = match reference {
        ThemeRef::Dark => ThemeSource::Dark,
        ThemeRef::Named(name) => ThemeSource::Named(name.as_str()),
        ThemeRef::File(path) => ThemeSource::File(Path::new(path)),
    };
    resolve_source(cache, source)
}

#[cfg(test)]
mod resolve_tests;
