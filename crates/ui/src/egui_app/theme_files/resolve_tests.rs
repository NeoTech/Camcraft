//! The document-to-kit theme map (#4394): each `ThemeRef` arm lands the
//! same table and the same refusal through the kit's `resolve_source`.
//! Provenance: our own tests for the UI kit split of #4245.

use camcraft_app::schema::ThemeRef;

use super::{Cache, resolve};
use crate::egui_app::theme::Tokens;

#[test]
fn builtin_references_land_their_compiled_columns() {
    let mut cache = Cache::new();
    let dark = resolve(&mut cache, &ThemeRef::Dark).expect("dark is built in");
    assert_eq!(dark, (Tokens::DARK, Vec::new()));
    let named_dark = resolve(&mut cache, &ThemeRef::Named("dark".into())).expect("dark by name");
    assert_eq!(named_dark, (Tokens::DARK, Vec::new()), "Dark and Named(dark) are one table");
    let light = resolve(&mut cache, &ThemeRef::Named("light".into())).expect("light by name");
    assert_eq!(light, (Tokens::LIGHT, Vec::new()));
}

#[test]
fn a_stranger_name_refuses_by_name() {
    let mut cache = Cache::new();
    let err = resolve(&mut cache, &ThemeRef::Named("solarpunk".into())).expect_err("not a built-in");
    assert!(err.contains("solarpunk"), "{err}");
    assert!(err.contains("dark, light"), "{err}");
}

#[test]
fn a_file_reference_reads_through_the_kit_cache() {
    let dir = std::env::temp_dir().join(format!("camcraft_ui_theme_resolve_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("make the fixture dir");
    let path = dir.join("teal.json");
    std::fs::write(
        &path,
        r##"{"format":"camcraft-theme/1","descriptor":{"name":"Teal"},"base":"light","colors":{"accent":"#2A7F79"}}"##,
    )
    .expect("write the fixture file");
    let mut cache = Cache::new();
    let reference = ThemeRef::File(path.display().to_string());
    let (tokens, notes) = resolve(&mut cache, &reference).expect("a legal file lands");
    assert_eq!(tokens.text, Tokens::LIGHT.text, "the light base carries through the map");
    assert!(!notes.is_empty(), "the derived accent states say so");
    assert_eq!(cache.peek_label(&path).as_deref(), Some("Teal"), "the parse went into the cache");
    let gone = ThemeRef::File(dir.join("vanished.json").display().to_string());
    assert!(resolve(&mut cache, &gone).is_err(), "a vanished file refuses");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The old module paths name the kit's own items (#4394), so code that
/// still writes `egui_app::theme::...` or `ground_ink::light_ground` gets
/// the same table and the same threshold.
#[test]
fn the_old_paths_name_the_kit_items() {
    let kit: camcraft_ui_kit::theme::Tokens = crate::egui_app::theme::Tokens::LIGHT;
    assert_eq!(kit, camcraft_ui_kit::theme::Tokens::LIGHT);
    let light = crate::egui_app::ground_ink::light_ground;
    assert!(light(Tokens::LIGHT.ink) && !light(Tokens::DARK.ink));
    assert_eq!(crate::egui_app::contrast::BODY, camcraft_ui_kit::contrast::BODY);
    assert_eq!(super::FORMAT, camcraft_ui_kit::theme_files::FORMAT);
}
