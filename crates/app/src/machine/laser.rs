//! Laser cutter program checks (#4081). In the kind slice no laser
//! operation exists, so a laser machine refuses each operation by name;
//! the Laser Profile slice judges its own moves here.
//! Provenance: our own machine-limit law, on the waterjet checks (#1881).

use super::{say, Machine, MachineCheck};

pub(super) fn check_laser_ops(machine: &Machine, ops: &[(String, flecs_ecs::prelude::Entity)]) -> MachineCheck {
    let mut out = MachineCheck::default();
    out.notes.push(format!("{}: no spindle; spindle rpm and power are not judged", machine.model));
    for (name, _) in ops {
        say(&mut out, format!("{name}: a laser machine admits only laser operations"));
    }
    out
}
