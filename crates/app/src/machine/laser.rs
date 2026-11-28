//! Laser cutter program checks (#4081): the waterjet's X/Y gantry
//! checks with the laser's family, tool kind and words.
//! Provenance: our own machine-limit law, on the waterjet checks (#1881).

use flecs_ecs::prelude::{Entity, World};

use super::waterjet::{check_xy_ops, XyCutter};
use super::{Machine, MachineCheck};
use crate::{operation::Family, tools::ToolKind};

pub(super) fn check_laser_ops(world: &World, machine: &Machine, ops: &[(String, Entity)]) -> Result<MachineCheck, String> {
    check_xy_ops(world, machine, ops, XyCutter { family: Family::Laser, tool: ToolKind::Laser, word: "laser" })
}
