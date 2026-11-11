//! The mod-vise pair command (#820): place a fixed and an adjustable jaw
//! block on a fixture plate's grid, fitted to the setup's stock, in ONE
//! undo unit. Each block lands through the fixture library's insert path
//! (`InsertFixture`, marked with its block parameters and its place on
//! the plate), seated by the same rules as `PlaceOnPlate` (the hole, the
//! turn, standing on the plate top, nothing hanging off). So both blocks
//! are ordinary meshed fixtures to the collision laws and riders of the
//! plate: moving the plate carries them, deleting it lets go of them.
//!
//! Provenance: our own composition of the insert and plate placement
//! commands over `modvise`'s geometry and fit.

use flecs_ecs::prelude::World;

use crate::commands::fixture_plate::{plate_of, seat};
use crate::commands::{Command, CommandError, InsertFixture};
use crate::fixture;
use crate::fixture_library::FixtureEntry;
use crate::geometry::{IDENTITY_Q, Point3};
use crate::model;
use crate::modvise::{self, JawParams, PairFit};
use crate::plate::{MountAt, PlateMount};
use crate::schema::Location;

/// Place a mod-vise pair on a plate.
pub struct PlaceModVisePair {
    fixed: InsertFixture,
    adjustable: InsertFixture,
    fit: PairFit,
}

/// One block of the pair, seated at `at` turned `yaw_deg`, as an insert.
fn block(
    world: &World,
    plate: &str,
    setup: Option<&str>,
    params: JawParams,
    at: &str,
    yaw_deg: f64,
) -> Result<InsertFixture, CommandError> {
    let def = plate_of(world, plate)?;
    let (plate_at, plate_mesh) = model::find_by_name(world, plate)
        .map(|e| (model::transform_of(world, &e).unwrap_or_default(), model::facets_of(world, &e).map(|f| f.to_vec()).unwrap_or_default()))
        .ok_or_else(|| CommandError::NotFound(plate.to_owned()))?;
    let facets = params.mesh().map_err(CommandError::Invalid)?;
    let m = PlateMount { plate: plate.to_owned(), at: MountAt::Hole(at.to_owned()), yaw_deg };
    let pose = seat(&def, &plate_mesh, plate_at, &params.base_name(), &facets, &m)?;
    let entry = FixtureEntry { name: params.base_name(), note: None, datum: Location { pos: Point3::ZERO, rot: IDENTITY_Q }, facets };
    Ok(InsertFixture::new(world, &entry, setup, pose)?.with_mod_jaw(params).with_mount(m))
}

impl PlaceModVisePair {
    /// Refused by name: an unknown plate or one with no grid, nonsense
    /// block figures, no stock in the plate's setup, and every refusal of
    /// the fit (`modvise::fit_pair`) or the seat (a block hanging off).
    pub fn new(world: &World, plate: &str, fixed: &JawParams, adjustable: &JawParams, at: &str, yaw_deg: f64) -> Result<Self, CommandError> {
        let def = plate_of(world, plate)?;
        let entity = model::find_by_name(world, plate).ok_or_else(|| CommandError::NotFound(plate.to_owned()))?;
        let plate_at = model::transform_of(world, &entity).unwrap_or_default();
        let setup = fixture::def_of(world, &entity).and_then(|d| d.setup);
        let stock = model::stock_world_aabb_for(world, setup.as_deref()).ok_or_else(|| {
            CommandError::Invalid(format!(
                "{} has no stock to fit the mod vise to",
                setup.as_deref().map_or_else(|| "the setup".to_owned(), |s| format!("setup '{s}'"))
            ))
        })?;
        let fit = modvise::fit_pair(&def, plate_at, fixed, adjustable, at, yaw_deg, &stock)
            .map_err(|e| CommandError::Invalid(format!("mod vise on '{plate}': {e}")))?;
        let fixed_cmd = block(world, plate, setup.as_deref(), fixed.with_slide(0.0), &fit.fixed_at, fit.yaw_deg)?;
        let adj = adjustable.with_slide(fit.slide);
        // The names differ by side ("Mod vise fixed ..." / "... adjustable
        // ..."), and the fit refuses two blocks of one side.
        let adjustable_cmd = block(world, plate, setup.as_deref(), adj, &fit.adjustable_at, fit.yaw_deg + 180.0)?;
        Ok(Self { fixed: fixed_cmd, adjustable: adjustable_cmd, fit })
    }

    /// The model names the blocks land as: (fixed, adjustable).
    #[must_use]
    pub fn names(&self) -> (&str, &str) {
        (self.fixed.name(), self.adjustable.name())
    }

    /// The fit: the holes, the turn, the slide, the opening.
    #[must_use]
    pub fn fit(&self) -> &PairFit {
        &self.fit
    }

    /// Where the blocks land (world): (fixed, adjustable).
    #[must_use]
    pub fn locations(&self) -> (Location, Location) {
        (self.fixed.location(), self.adjustable.location())
    }
}

impl Command for PlaceModVisePair {
    fn apply(&self, world: &World) {
        self.fixed.apply(world);
        self.adjustable.apply(world);
    }

    fn invert(&self, world: &World) {
        self.adjustable.invert(world);
        self.fixed.invert(world);
    }
}
