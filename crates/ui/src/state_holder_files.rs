//! #4389: the Holders page's Import… and Export… buttons - a file
//! dialog, then `commands::export_holders` to write the document's user
//! holders, or `ImportHolders` (one undo step) to add the holders of a
//! file. A refusal lands on the Tool pane by name, and the document does
//! not change.
//! Provenance: our own `camcraft-components/1` format
//! (`app::component_library`).

use std::path::Path;

use camcraft_app::commands::{CommandError, ImportHolders, export_holders};

use super::{ErrorScope, State};

/// The holder file extension (the component library format).
pub(crate) const HOLDER_EXTENSION: &str = "camcomponents";

impl State {
    /// "Export…": ask where, then write the user holders.
    pub(crate) fn export_holders_dialog(&mut self) {
        if self.refuse_in_browser(crate::browser_files::NO_EXPORT) {
            return;
        }
        let Some(path) = crate::file_dialog::FileDialog::new()
            .add_filter("CamCraft Holder File", &[HOLDER_EXTENSION])
            .set_file_name(format!("holders.{HOLDER_EXTENSION}"))
            .save_file()
        else {
            return;
        };
        self.export_holders_to(&path);
    }

    /// "Import…": pick a holder file, then import it.
    pub(crate) fn import_holders_dialog(&mut self) {
        if self.refuse_in_browser(crate::browser_files::NEEDS_PICKER) {
            return;
        }
        let picked = crate::file_dialog::FileDialog::new()
            .add_filter("CamCraft Holder File", &[HOLDER_EXTENSION, "json"])
            .pick_file();
        if let Some(path) = picked {
            self.import_holders_from(&path);
        }
    }

    pub(crate) fn export_holders_to(&mut self, path: &Path) {
        let written = export_holders(&self.world).and_then(|text| std::fs::write(path, text).map_err(|e| e.to_string()));
        match written {
            Ok(()) => self.status = format!("holders written to {}", path.display()),
            Err(err) => self.refuse(ErrorScope::Tool, format!("holders not exported: {err}")),
        }
    }

    pub(crate) fn import_holders_from(&mut self, path: &Path) {
        // A new id keeps clear of the component library's ids, as
        // "Copy as user holder" does.
        let reserved: Vec<String> = self.component_library().all.iter().map(|c| c.component.id().to_owned()).collect();
        let planned = std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|text| {
                ImportHolders::from_text(&self.world, &text, &reserved).map_err(|e| match e {
                    CommandError::Invalid(m) => m,
                    other => other.to_string(),
                })
            });
        match planned {
            Ok(command) => {
                let summary = command.summary();
                self.history.execute(&self.world, command);
                self.doc_version += 1;
                self.status = summary;
            }
            Err(err) => self.refuse(ErrorScope::Tool, format!("holders not imported: {err}")),
        }
    }
}
