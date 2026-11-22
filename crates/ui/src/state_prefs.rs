//! The theme and colour-vision preferences on the State (issue #877):
//! read once at startup from the per-user `camcraft_app::prefs` file,
//! written back on every pick. They are the person's, not the
//! document's: a pick is not on the history (Ctrl+Z never undoes it),
//! it does not mark the document changed, and neither New nor Open
//! touches it. Each pick is still validated before it is set - a missing
//! or corrupt theme file and a colour-vision mode that fails its gate on
//! the live theme are refused by name on the status line.
//! Provenance: our own wiring over our own `app::prefs`, shaped like
//! `state_recent.rs` (#662); no HSMWorks material.

use std::path::{Path, PathBuf};

use camcraft_app::commands::{check_theme, color_vision_of, color_vision_set, theme_of, theme_set};
use camcraft_app::prefs::{self, ViewPrefs};
use camcraft_app::schema::{ColorVision, ThemeRef};

use super::{ErrorScope, State, theme_label};
use crate::egui_app::{colorvision, theme, theme_files};

impl State {
    /// Startup: the per-user preferences file (the real window only - the
    /// headless suites never touch the user's file).
    pub(crate) fn load_prefs(&mut self) {
        let Some(path) = prefs::store_path() else {
            diag_warn!("preferences: no config folder - the theme and colour vision last for this session only");
            return;
        };
        self.load_prefs_from(path);
    }

    /// Read the preferences from `path`, wear them, and keep writing back
    /// there. A stored colour-vision mode is gated against the theme it
    /// lands on, as a pick is; one that fails stays Off, named.
    pub fn load_prefs_from(&mut self, path: PathBuf) {
        let (stored, note) = prefs::load(&path);
        self.prefs_store = Some(path);
        self.import_tool_holders = !stored.skip_tool_holders;
        self.ai_chat.offline_only = stored.ai_offline_only;
        theme_set(&self.world, &stored.theme);
        let tokens = theme_files::resolve(&mut self.theme_cache, &stored.theme)
            .map_or_else(|_| theme::current(), |(t, _)| t);
        let mut notes: Vec<String> = note.into_iter().collect();
        match colorvision::gate(stored.color_vision, &tokens) {
            Some(why) => notes.push(camcraft_i18n::tf("colour vision {} not restored: {why}", &[format!("{}", stored.color_vision.label()), format!("{}", why)])),
            None => color_vision_set(&self.world, stored.color_vision),
        }
        // #853: the pinned language applies before the first frame, unless
        // CAMCRAFT_LANG named one for this run (an explicit override wins).
        if !camcraft_i18n::env_override()
            && let Some(tag) = &stored.language
        {
            match camcraft_i18n::Lang::from_tag(tag) {
                Some(l) if camcraft_i18n::set_lang(l) == l => self.language = Some(l),
                Some(_) => notes.push(camcraft_i18n::tf("language '{tag}' could not load - following the system locale", &[format!("{}", tag)])),
                None => notes.push(camcraft_i18n::tf("language '{tag}' is not one this build speaks - following the system locale", &[format!("{}", tag)])),
            }
        }
        if !notes.is_empty() {
            self.status = notes.join("; ");
        }
    }

    /// Where the preferences are written back, if anywhere.
    #[must_use]
    pub fn prefs_path(&self) -> Option<&Path> {
        self.prefs_store.as_deref()
    }

    pub(crate) fn persist_prefs(&self) {
        let Some(path) = &self.prefs_store else { return };
        let now = ViewPrefs {
            theme: theme_of(&self.world),
            color_vision: color_vision_of(&self.world),
            skip_tool_holders: !self.import_tool_holders,
            ai_offline_only: self.ai_chat.offline_only,
            // #1873: no control sets this path yet (#1876), so a save keeps
            // the path that the file states.
            cadmpeg_path: prefs::load(path).0.cadmpeg_path,
            language: self.language.map(|l| l.tag().to_owned()),
        };
        // The refusal is logged where it happens (app::prefs::save); the
        // choice in force stays right for this session either way.
        let _ = prefs::save(path, &now);
    }

    /// Choose the interface language (#853): `None` follows the system
    /// locale. The pick is the person's, so it is written to the preferences
    /// file at once and never touches the undo history; the surface reinstalls
    /// the font and repaints, and every label is looked up again next frame.
    pub(crate) fn select_language(&mut self, next: Option<camcraft_i18n::Lang>) {
        let wanted = next.unwrap_or_else(camcraft_i18n::system_lang);
        let in_force = camcraft_i18n::set_lang(wanted);
        self.language = next;
        self.persist_prefs();
        // The native name is never translated - it names the language in
        // its own words, so the line reads the same in either language.
        self.status = camcraft_i18n::tf("language: {}", &[format!("{}", in_force.native_name())]);
    }

    /// Choose the theme. The cheap existence law (`check_theme`) and the
    /// full parse - which the app cannot name (layering: the JSON reader
    /// lives on the surface side) - both run HERE, so a corrupt theme is a
    /// visible refusal at the pick and is never set or stored. The look
    /// itself lands on the next `sync_theme`.
    pub(super) fn select_theme(&mut self, next: ThemeRef) {
        if let ThemeRef::File(path) = &next
            && let Err(why) = &*self.theme_cache.get(Path::new(path))
        {
            self.refuse(ErrorScope::Setup, camcraft_i18n::tf("theme not selected: {why}", &[format!("{}", why)]));
            return;
        }
        if let Err(why) = check_theme(&next) {
            self.refuse(ErrorScope::Setup, camcraft_i18n::tf("theme not selected: {why}", &[format!("{}", why)]));
            return;
        }
        if theme_of(&self.world) != next {
            theme_set(&self.world, &next);
            self.persist_prefs();
        }
        self.status = camcraft_i18n::tf("theme: {}", &[format!("{}", theme_label(&self.world))]);
    }

    /// Choose the colour-vision remap, `select_theme`'s twin: the surface
    /// owns the palette and the gate (layering - app holds only the
    /// reference), so the gate runs HERE against the live table and a
    /// failing mode is a visible refusal at the pick.
    pub(super) fn select_color_vision(&mut self, next: ColorVision) {
        if let Some(why) = colorvision::gate(next, &theme::current()) {
            self.refuse(ErrorScope::Setup, camcraft_i18n::tf("colour vision not selected: {why}", &[format!("{}", why)]));
            return;
        }
        if color_vision_of(&self.world) != next {
            color_vision_set(&self.world, next);
            self.persist_prefs();
        }
        self.status = camcraft_i18n::tf("colour vision: {}", &[format!("{}", next.label())]);
    }
}
