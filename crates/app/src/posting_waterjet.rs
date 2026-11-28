//! Provenance: our own safety refusal for incomplete waterjet output (#3637).

use flecs_ecs::prelude::World;

pub(super) fn problems(world: &World) -> Vec<String> {
    // A program that cannot be listed is refused by the sim clause with
    // its own reason; repeating it here would name the fault twice.
    let Ok(ops) = super::program_ops(world) else {
        return Vec::new();
    };
    use crate::operation::Family;
    ops.into_iter()
        .filter_map(|(name, entity)| {
            match crate::model::operation_of(world, &entity)?.params.kind().info().family() {
                Family::Waterjet => Some(format!(
                    "{name}: waterjet posting is unavailable: standoff height, jet on/off records, pierce dwell records, and waterjet laws are not implemented"
                )),
                // #4081: the laser post is the next slice.
                Family::Laser => Some(format!(
                    "{name}: laser posting is unavailable: beam on/off records, power records and pierce dwell records are not implemented"
                )),
                _ => None,
            }
        })
        .collect()
}
