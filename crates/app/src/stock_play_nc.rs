//! A playback seed from a plan the caller resolved (#2218: an external NC
//! program's sections, `nc_sim::play_seed`). Provenance: the same display
//! inputs as `PlaySeed::gather` (`stock_view::display_inputs_of`).

use crate::geometry::Triangle;
use crate::stock_sim::Planned;
use crate::stock_view::display_inputs_of;

use super::PlaySeed;

impl PlaySeed {
    /// A seed over `planned`, drawn on `setup`'s stock at `texels` cells,
    /// judged against `part` at `tol` when a part is given. The whole plan
    /// plays: nothing waits for generation.
    pub(crate) fn of_planned(
        world: &flecs_ecs::prelude::World,
        planned: Vec<Planned>,
        setup: Option<String>,
        part: Option<Vec<Triangle>>,
        tol: f64,
        texels: f64,
    ) -> Result<Self, String> {
        let (stock_facets, planned, cell) = display_inputs_of(world, texels, planned, setup)?;
        Ok(PlaySeed { stock_facets, planned, cell, part, tol: Ok(tol), rest: Vec::new() })
    }
}
