//! The viewport handle drag's dispatch arm (#1198 slice b): a released
//! drag queues `CommandKind::MoveModelBy`; it lands here as the one
//! `MoveModel` the Model page's fields run, so a dragged move and a typed
//! move are the same undo step, the same staleness and the same following
//! stock. A refusal names itself on the status line.
//! Provenance: our own routing over our own command; no vendor material.

use camcraft_app::commands::MoveModel;
use camcraft_app::geometry::quat_to_euler;
use camcraft_app::model;

use super::locate_ui::with_warning;
use super::{ErrorScope, State};

impl State {
    pub(super) fn move_model_by(&mut self, name: &str, delta: [f64; 3]) {
        if !delta.iter().all(|v| v.is_finite()) {
            self.refuse(ErrorScope::Setup, "part not moved: the drag distance must be finite".to_string());
            return;
        }
        let Some(entity) = model::find_by_name(&self.world, name) else {
            self.refuse(ErrorScope::Setup, format!("part not moved: no part named '{name}'"));
            return;
        };
        let Some(location) = crate::move_drag::document_target(self, &entity, delta) else {
            self.refuse(ErrorScope::Setup, format!("part not moved: '{name}' would leave finite space"));
            return;
        };
        match MoveModel::new(&self.world, name, location) {
            Ok(command) => {
                let warning = command.warning().map(str::to_owned);
                self.history.execute(&self.world, command);
                self.doc_version += 1;
                // The Model page shows where the part now is.
                if self.model_ui.name == name {
                    let (r, p, y) = quat_to_euler(location.rot);
                    self.model_ui.x = location.pos.x;
                    self.model_ui.y = location.pos.y;
                    self.model_ui.z = location.pos.z;
                    self.model_ui.roll_deg = r.to_degrees();
                    self.model_ui.pitch_deg = p.to_degrees();
                    self.model_ui.yaw_deg = y.to_degrees();
                }
                let shown = crate::move_drag::shown_delta(delta, self.units());
                self.status = with_warning(format!("moved {name} {shown}"), warning);
            }
            Err(err) => self.refuse(ErrorScope::Setup, format!("part not moved: {err}")),
        }
    }
}
