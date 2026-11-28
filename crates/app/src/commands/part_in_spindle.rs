//! `SetPartInSpindle` and `SetTableStation` (#3406): state, change or
//! clear a mill setup's part-in-spindle mode, and put or remove one table
//! tool station. Construction validates the whole next record
//! ([`crate::part_in_spindle::invalid`]: the holding, each work offset in
//! range, no two tools on one work offset), snapshots the previous one and
//! refuses a no-op; `apply` / `invert` are pure writes of
//! `SetupDef::part_in_spindle`, so undo puts the previous record back
//! exactly.
//!
//! Provenance: our own design (the `SetSetupWorkholdingRpm` shape, #1031);
//! no vendor implementation inspected.

use flecs_ecs::core::EntityViewGet; // try_get is not re-exported in prelude (0.2.2)
use flecs_ecs::prelude::World;

use crate::commands::{Command, CommandError};
use crate::machine::Kind;
use crate::model;
use crate::part_in_spindle::{self, Holding, PartInSpindle, TableStation};
use crate::schema::SetupDef;

/// The setup's current record; refuses a name that is no setup.
fn record_of(world: &World, setup: &str) -> Result<Option<PartInSpindle>, CommandError> {
    let entity = model::find_by_name(world, setup).ok_or_else(|| CommandError::NotFound(setup.to_string()))?;
    entity
        .entity_view(world)
        .try_get::<&SetupDef>(|d| d.part_in_spindle.clone())
        .ok_or_else(|| CommandError::Invalid(format!("'{setup}' is not a setup")))
}

fn write(world: &World, setup: &str, value: Option<&PartInSpindle>) {
    let Some(entity) = model::find_by_name(world, setup) else {
        return;
    };
    let view = entity.entity_view(world);
    if let Some(mut def) = view.try_get::<&SetupDef>(|d| d.clone()) {
        def.part_in_spindle = value.cloned();
        view.set(def);
    }
}

fn checked(next: PartInSpindle) -> Result<PartInSpindle, CommandError> {
    match part_in_spindle::invalid(&next) {
        Some(why) => Err(CommandError::Invalid(why)),
        None => Ok(next),
    }
}

/// State, change or clear (`None`) a setup's part-in-spindle mode.
#[derive(Debug)]
pub struct SetPartInSpindle {
    setup: String,
    prev: Option<PartInSpindle>,
    next: Option<PartInSpindle>,
}

impl SetPartInSpindle {
    /// `holding` states the mode (a change keeps the stations); `None`
    /// clears it. The mode is for a mill only; it is not stated over a
    /// milling operation, and not cleared under any operation.
    pub fn new(world: &World, setup: &str, holding: Option<Holding>) -> Result<Self, CommandError> {
        if let Some(why) = holding.as_ref().and_then(part_in_spindle::invalid_holding) {
            return Err(CommandError::Invalid(why));
        }
        let prev = record_of(world, setup)?;
        let next = match (&prev, holding) {
            (None, None) => {
                return Err(CommandError::Invalid(format!("setup '{setup}' states no part-in-spindle mode")));
            }
            (Some(p), Some(h)) if p.holding == h => {
                return Err(CommandError::Invalid(format!("setup '{setup}' already holds the part in the spindle this way")));
            }
            (Some(_), None) => {
                if let Some((op, _)) = part_in_spindle::ops_of(world, setup).first() {
                    return Err(CommandError::Invalid(format!(
                        "setup '{setup}' still has operation {op} - delete or move its operations before you clear part-in-spindle mode"
                    )));
                }
                None
            }
            (_, Some(h)) => {
                let kind = crate::machine::kind_of_setup(world, Some(setup));
                if kind != Kind::Mill {
                    return Err(CommandError::Invalid(format!(
                        "part-in-spindle mode is for a mill (the part in its spindle, lathe tools on its table); setup '{setup}''s machine is a {}",
                        kind.word()
                    )));
                }
                if prev.is_none()
                    && let Some((op, o)) = part_in_spindle::ops_of(world, setup).into_iter().find(|(_, o)| !part_in_spindle::runs(&o.params))
                {
                    return Err(CommandError::Invalid(format!(
                        "setup '{setup}' has {} operation {op}, which part-in-spindle mode does not run - delete or move it first",
                        o.params.kind()
                    )));
                }
                let stations = prev.as_ref().map(|p| p.stations.clone()).unwrap_or_default();
                Some(checked(PartInSpindle { holding: h, stations })?)
            }
        };
        Ok(Self { setup: setup.to_owned(), prev, next })
    }

    #[must_use]
    pub fn setup(&self) -> &str {
        &self.setup
    }
}

impl Command for SetPartInSpindle {
    fn apply(&self, world: &World) {
        write(world, &self.setup, self.next.as_ref());
    }

    fn invert(&self, world: &World) {
        write(world, &self.setup, self.prev.as_ref());
    }
}

/// Put or remove one table tool station of a part-in-spindle setup.
#[derive(Debug)]
pub struct SetTableStation {
    setup: String,
    prev: PartInSpindle,
    next: PartInSpindle,
}

fn stated(world: &World, setup: &str) -> Result<PartInSpindle, CommandError> {
    record_of(world, setup)?.ok_or_else(|| {
        CommandError::Needs(format!("setup '{setup}' states no part-in-spindle mode - state it first"))
    })
}

impl SetTableStation {
    /// Put `station` on the table: a new tool, or the tool's station
    /// replaced. Refuses a tool that does not exist and a work offset
    /// that another tool has.
    pub fn put(world: &World, setup: &str, station: TableStation) -> Result<Self, CommandError> {
        let prev = stated(world, setup)?;
        match model::find_by_name(world, &station.tool) {
            Some(e) if model::is_tool(world, &e) => {}
            Some(_) => return Err(CommandError::Invalid(format!("'{}' is not a tool", station.tool))),
            None => return Err(CommandError::NotFound(station.tool.clone())),
        }
        if prev.stations.contains(&station) {
            return Err(CommandError::Invalid(format!("tool {} already has this table station", station.tool)));
        }
        let mut next = prev.clone();
        match next.stations.iter_mut().find(|s| s.tool == station.tool) {
            Some(s) => *s = station,
            None => next.stations.push(station),
        }
        Ok(Self { setup: setup.to_owned(), prev, next: checked(next)? })
    }

    /// Take `tool`'s station off the table.
    pub fn remove(world: &World, setup: &str, tool: &str) -> Result<Self, CommandError> {
        let prev = stated(world, setup)?;
        let mut next = prev.clone();
        next.stations.retain(|s| s.tool != tool);
        if next.stations.len() == prev.stations.len() {
            return Err(CommandError::Invalid(format!("tool {tool} has no table station in setup '{setup}'")));
        }
        Ok(Self { setup: setup.to_owned(), prev, next })
    }

    #[must_use]
    pub fn setup(&self) -> &str {
        &self.setup
    }
}

impl Command for SetTableStation {
    fn apply(&self, world: &World) {
        write(world, &self.setup, Some(&self.next));
    }

    fn invert(&self, world: &World) {
        write(world, &self.setup, Some(&self.prev));
    }
}
