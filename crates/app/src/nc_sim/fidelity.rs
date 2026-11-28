//! NC verdict preparation from the selected document meshes. Provenance:
//! CamCraft's model-fidelity law (#864, #1056): no verdict from unknown
//! facets, and a strict tolerance budget for reduced-fidelity surfaces.

use flecs_ecs::prelude::World;

use super::NcSimOptions;
use crate::{model, model_fidelity as fidelity};

/// Guard the same inputs for both simulation and playback. Caller-supplied
/// target facets and fixture boxes replace the corresponding document inputs;
/// the setup's stock always comes from the document.
pub(super) fn prepare(world: &World, opts: &NcSimOptions, requested: f64) -> Result<f64, String> {
    let setup = opts.setup.as_deref();
    let stock = crate::setup::stock_entity_name(world, setup);
    if let Some(entity) = model::find_by_name(world, &stock)
        && fidelity::chord_of(world, &entity).is_none()
        && !fidelity::stock_fidelity_known(world, setup)
    {
        return Err(format!(
            "stock '{stock}' has unknown mesh fidelity - re-import its source and recreate the stock"
        ));
    }
    let mut judged = vec![stock];
    if opts.target.is_none() {
        let skip = crate::stock_source::stock_only_model(world);
        judged.extend(
            model::model_entities(world)
                .into_iter()
                .filter(|(_, e)| crate::stock_source::judged(world, e, skip))
                .filter(|(_, e)| {
                    crate::setup_align::facets_in(world, e, setup).is_some_and(|f| !f.is_empty())
                })
                .map(|(n, _)| n),
        );
    }
    let mut worst = 0.0_f64;
    let mut coarsest = String::new();
    for name in judged {
        if let Some(chord) = fidelity::chord_behind(world, &name) {
            if fidelity::is_unknown(chord) {
                return Err(format!(
                    "model or stock '{name}' was {}",
                    crate::mesh_open::unknown_note(world, &name)
                ));
            }
            let excess = fidelity::excess(chord);
            if excess > worst {
                worst = excess;
                coarsest = name;
            }
        }
    }
    // Known coarse fixtures are already grown by their chord in steel_box.
    // The unknown sentinel deliberately cannot grow a box: refuse it here.
    if opts.fixtures.is_none() {
        for (name, entity) in crate::fixture::tagged(world, setup) {
            if fidelity::chord_of(world, &entity).is_some_and(fidelity::is_unknown) {
                return Err(format!(
                    "fixture '{name}' was {}",
                    crate::mesh_open::unknown_note(world, &name)
                ));
            }
        }
    }
    if worst == 0.0 {
        return Ok(requested);
    }
    // This runner checks shanks and holders as well as cutting edges, so
    // use the strict law, without the cutting-only file-tolerance widening.
    // An explicit tolerance cannot buy back a depleted fidelity budget.
    crate::verify::excess_floor(requested.min(crate::stock_sim::DEV_TOL), worst)
        .ok_or_else(|| format!("model or stock '{coarsest}' was {}", fidelity::TOO_COARSE))
}
