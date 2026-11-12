//! Open model / Open project where the file arrives as bytes (#31): the
//! browser's async picker (`crate::file_pick`) delivers a name and the
//! file's contents, and they run through the same commands a file opened
//! from disk runs - `ImportModel` (one undo step) for a model, the project
//! loader for a `.camproj`. A file the browser could not read, or one the
//! importer refuses, is said on the status line by name; nothing changes.
//! Native builds keep their synchronous dialogs; the inbox they drain each
//! frame stays empty there.
//! Provenance: our own routing over the app's import and project laws.

use std::path::Path;

use camcraft_app::clock::Instant;
use camcraft_app::commands::ImportModel;
use camcraft_app::project;

use super::{CommandKind, State};
use crate::file_pick::{PickFor, PickedFile};

impl State {
    /// File > Open model: rfd's dialog natively; in a browser, the async
    /// picker - its answer comes back through [`State::take_picked_files`].
    pub(crate) fn open_model_dialog(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = super::mesh_dialog().pick_file() {
            self.import_path(path);
        }
        #[cfg(target_arch = "wasm32")]
        {
            crate::file_pick::open(&super::mesh_dialog(), PickFor::Model);
            self.status = String::from("choose a model to import…");
        }
    }

    /// File > Open project, after the unsaved-changes prompt: as above.
    pub(crate) fn open_project_dialog(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = super::project_dialog().pick_file() {
            self.open_project_path(path);
        }
        #[cfg(target_arch = "wasm32")]
        {
            crate::file_pick::open(&super::project_dialog(), PickFor::Project);
            self.status = String::from("choose a project to open…");
        }
    }

    /// Queue every file the picker delivered since the last frame as one
    /// `OpenPicked` command each, so it runs where every action runs.
    pub(crate) fn take_picked_files(&mut self) {
        for file in crate::file_pick::take_picked() {
            self.run(CommandKind::OpenPicked(file));
        }
    }

    /// Land one picked file, or refuse it by name.
    pub(crate) fn open_picked(&mut self, file: PickedFile) {
        let PickedFile { purpose, name, bytes } = file;
        let data = match bytes {
            Ok(data) => data,
            Err(why) => {
                self.status = match purpose {
                    PickFor::Model => format!("import refused: {name} could not be read: {why}"),
                    PickFor::Project => format!("open failed: {name} could not be read: {why}"),
                };
                return;
            }
        };
        match purpose {
            PickFor::Model => self.import_bytes(&name, &data),
            PickFor::Project => self.open_project_bytes(&name, &data),
        }
    }

    /// The bytes twin of `import_path`: parsed here, on this thread (a
    /// browser has no worker to hand it to), then landed by
    /// `finish_import` like any import - fresh, update or refusal.
    fn import_bytes(&mut self, name: &str, data: &[u8]) {
        let budget = self.step_budget_for(Path::new(name));
        let result = ImportModel::from_bytes(name, data, budget);
        // A disk import may be in flight on native: its file, clock and
        // fixture binding are its own, so they are set aside around ours.
        let held = (self.import_file.take(), self.import_started.take(), self.import_as_fixture.take());
        self.import_started = Some(Instant::now());
        self.finish_import(result);
        (self.import_file, self.import_started, self.import_as_fixture) = held;
    }

    /// The bytes twin of `open_project_path`. The document has no path
    /// afterwards (a browser file has none), so Save asks where.
    fn open_project_bytes(&mut self, name: &str, data: &[u8]) {
        let Ok(text) = std::str::from_utf8(data) else {
            self.status = format!("open failed: {name} is not a CamCraft project (not text)");
            return;
        };
        self.reset_for_open();
        match project::open_noted(&self.world, text) {
            Ok(notes) => {
                self.history.mark_clean();
                self.doc_version += 1;
                self.status = if notes.is_empty() {
                    format!("opened {name}")
                } else {
                    format!("opened {name} - {}", notes.join("; "))
                };
            }
            Err(err) => self.status = format!("open failed: {name}: {err}"),
        }
    }
}
