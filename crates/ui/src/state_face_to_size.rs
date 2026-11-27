//! Face to size from the Stock page (#3196): the group queues
//! `CommandKind::FaceToSize`, and the drain lands it here as one undoable
//! `FaceToSize` (the stock and a Face on it); a refusal is the command's
//! own text on the setup's error line. The new Face is selected.
//! Provenance: our own page law (issue #3196); no vendor material consulted.

use camcraft_app::commands::{FaceToSize, FaceToSizeSpec};

use super::{CommandKind, ErrorScope, State, UiTarget};

impl State {
    /// Add the face to size that `spec` states.
    pub(super) fn face_to_size(&mut self, spec: &FaceToSizeSpec) {
        match FaceToSize::new(&self.world, spec) {
            Ok(cmd) => {
                let name = cmd.operation().to_owned();
                self.history.execute(&self.world, cmd);
                self.doc_version += 1;
                self.pending.push_back(CommandKind::Select(Some(name.clone())));
                self.ui_target = UiTarget::Operation(name.clone());
                self.status = format!("{name}: faces Z{} down to Z{} in {} mm passes", spec.top_z, spec.final_z, spec.stepdown);
            }
            Err(why) => self.refuse(ErrorScope::Setup, format!("face to size not added: {why}")),
        }
    }
}
