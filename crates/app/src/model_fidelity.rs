//! Reduced-fidelity models (#864): a STEP model whose kept mesh is coarser
//! than the importer promises - the flag, what the gouge verdicts do about
//! it, and the program gate's refusal once they cannot be honest.
//!
//! Provenance: our own design (maintainer decision on #864). A facet of a
//! curved wall is a chord of the true surface and stands inside it by its
//! sagitta (elementary geometry), so against the mesh a convex wall looks
//! farther from the tool than the real one is. No vendor material consulted.
//!
//! ## The law here
//!
//! - A model carries [`ReducedFidelity`] exactly when its import kept a
//!   chord error above `step::CHORD_CAP`. Its EXCESS is that error less
//!   the cap: how much more a verdict against the mesh can forgive than
//!   the same verdict against a model that met the promise.
//! - While a verdict tolerance less the excess stays positive, the gouge
//!   laws judge with that smaller tolerance (`VerifyContext::model_excess`
//!   at generation, [`verdict_tol`] for the program sim): stricter, never
//!   looser. False gouges on a concave wall are the safe side.
//! - Once the excess uses a whole tolerance, a tightened law would convict
//!   every exactly finished flat face - a misleading verdict. The laws
//!   then read untightened and [`gate_problems`] refuses the operation by
//!   name at the post; the program sim does not judge at all.
//! - A kind whose own laws do not read the excess yet
//!   (`operation::fidelity_law::excess_aware`) is refused on any reduced
//!   model: a refusal is the only verdict that cannot be looser. #904
//!   took turning, the rotary kinds and the multi-axis ball kinds off
//!   that list; `fidelity_law`'s header says which law judges which.

use flecs_ecs::core::EntityViewGet;
use flecs_ecs::prelude::{Entity, World};

use crate::geometry::import::step;
use crate::model;
use crate::operation::{fidelity_law, SrcRef};
use crate::schema::{Operation, ReducedFidelity};
use crate::stock::StockShape;
use crate::stock_sim::DEV_TOL;
use crate::verify::excess_floor;

/// What every refusal here says, in the user's words.
pub const TOO_COARSE: &str = "imported too coarsely to verify; re-import with a larger budget";

/// The chord error a reduced-fidelity model was imported at, mm; `None`
/// for a model at full fidelity. A stored value that is not a coarse
/// chord (a hand-edited file) reads as none.
#[must_use]
pub fn chord_of(world: &World, entity: &Entity) -> Option<f64> {
    entity.entity_view(world).try_get::<&ReducedFidelity>(|r| r.chord_mm).and_then(step::coarse)
}

/// Write or clear the flag. For the import/update commands, their undo
/// and the project loader only - it describes the facets it lands with.
pub fn set(world: &World, entity: &Entity, coarse_mm: Option<f64>) {
    let view = entity.entity_view(world);
    match coarse_mm.and_then(step::coarse) {
        Some(chord_mm) => {
            view.set(ReducedFidelity { chord_mm });
        }
        None => {
            view.remove(world.component::<ReducedFidelity>());
        }
    }
}

/// The reduced-fidelity model behind the entity `name`: itself, or - for
/// a stock that is a copy of a model - the model it copies (same facets).
fn coarse_source(world: &World, name: &str) -> Option<(String, f64)> {
    let entity = model::find_by_name(world, name)?;
    if let Some(c) = chord_of(world, &entity) {
        return Some((name.to_string(), c));
    }
    let Some(StockShape::FromModel { source }) = model::stock_def_of(world, &entity) else {
        return None;
    };
    let src = model::find_by_name(world, &source)?;
    chord_of(world, &src).map(|c| (source, c))
}

/// How much of a verdict tolerance the named entity's mesh has used up,
/// mm: its chord error less the cap, 0 at full fidelity.
#[must_use]
pub fn excess_of(world: &World, name: &str) -> f64 {
    coarse_source(world, name).map_or(0.0, |(_, c)| excess(c))
}

/// Whether the excess of `setup`'s stock is known (#1091 review): a box
/// or cylinder has no mesh to be coarse; a from-model copy is known only
/// while its source model stands, since its chord is read through the
/// source (`coarse_source`) and reads 0 once the source is gone. A model
/// with no `ReducedFidelity` record is known exact - the importer writes
/// the record for every coarse import. No stock entity: nothing to know.
/// #1056: once a from-model stock carries its own chord (#1127), read it
/// here instead of requiring the source.
#[must_use]
pub fn stock_fidelity_known(world: &World, setup: Option<&str>) -> bool {
    let Some(stock) = model::find_by_name(world, &crate::setup::stock_entity_name(world, setup)) else {
        return true;
    };
    match model::stock_def_of(world, &stock) {
        // #1056: the chord lives on the source only - gone source, unknown chord.
        Some(StockShape::FromModel { source }) => model::find_by_name(world, &source).is_some(),
        _ => true,
    }
}

/// A coarse chord error's excess over the importer's promise.
#[must_use]
pub fn excess(chord_mm: f64) -> f64 {
    (chord_mm - step::CHORD_CAP).max(0.0)
}

/// The plain-words line the model page and the import report show.
#[must_use]
pub fn summary(world: &World, chord_mm: f64) -> String {
    format!("Reduced fidelity: surfaces are approximated to within {}", within(world, chord_mm))
}

/// The same facts for MCP and the script API: the achieved error in mm,
/// the importer's promise, and the sentence a person would be shown.
#[must_use]
pub fn facts(world: &World, chord_mm: f64) -> serde_json::Value {
    serde_json::json!({
        "approximation_mm": chord_mm,
        "promised_mm": step::CHORD_CAP,
        "note": format!(
            "{}. Gouge checks against this model are stricter; if it is too coarse to check, its operations are refused at the post. Re-import with a larger budget to clear this.",
            summary(world, chord_mm)
        ),
    })
}

/// "0.012 mm" in the document's unit.
fn within(world: &World, chord_mm: f64) -> String {
    let u = crate::units::of(world);
    format!("{} {}", crate::units::len(chord_mm, u, 3), u.suffix())
}

/// The coarsest model an operation is judged against, by name: the
/// entities its source names (one entity, or every entity of a
/// selection) and the setup's stock - the collision laws read clearance
/// against it (#904; `operation::material::judged_material` is the same
/// pair at generation).
fn op_coarse(world: &World, op: &Operation) -> Option<(String, f64)> {
    let mut names = vec![crate::setup::stock_entity_name(world, op.setup.as_deref())];
    match &op.src {
        SrcRef::Stock => {}
        SrcRef::Entity(n) => names.push(n.clone()),
        SrcRef::Selection(items) => names.extend(items.iter().map(|g| g.entity.clone())),
    }
    names
        .iter()
        .filter_map(|n| coarse_source(world, n))
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// Why `op` cannot be verified against the model it is judged on, or
/// `None` when it can (full fidelity, or tightened verdicts).
#[must_use]
pub fn op_problem(world: &World, name: &str, op: &Operation) -> Option<String> {
    let (model, chord) = op_coarse(world, op)?;
    let said = format!("surfaces are approximated to within {}", within(world, chord));
    if !fidelity_law::excess_aware(&op.params) {
        return Some(format!(
            "{name}: model '{model}' was imported at reduced fidelity ({said}) and this kind of operation cannot be verified against it; re-import with a larger budget"
        ));
    }
    excess_floor(fidelity_law::floor_of(&op.params), excess(chord))
        .is_none()
        .then(|| format!("{name}: model '{model}' was {TOO_COARSE} ({said})"))
}

/// The program gate's clause: one line per operation of the program that
/// cannot be verified against its model, else one for the program when a
/// part is too coarse for the program verdict. A program the post would
/// refuse anyway (a stale op) is that refusal's to name.
#[must_use]
pub fn gate_problems(world: &World) -> Vec<String> {
    let Ok(ops) = crate::posting::program_ops(world) else {
        return Vec::new();
    };
    let mut out: Vec<String> = ops
        .iter()
        .filter_map(|(name, e)| op_problem(world, name, &model::operation_of(world, e)?))
        .collect();
    // A part no operation names is still what the program verdict is read
    // against: when it is too coarse for that verdict, the program is
    // refused rather than posted without it.
    if out.is_empty()
        && let Err(why) = verdict_tol(world)
    {
        out.push(why);
    }
    out
}

/// The program sim's verdict tolerance for this document: `DEV_TOL`, less
/// the worst excess among the parts the verdict is read against. `Err`
/// names the model once nothing honest is left - the sim then judges
/// nothing rather than paint a misleading map.
pub fn verdict_tol(world: &World) -> Result<f64, String> {
    let worst = model::model_entities(world)
        .into_iter()
        .filter(|(_, e)| !model::is_fixture(world, e))
        .filter_map(|(n, e)| chord_of(world, &e).map(|c| (n, c)))
        .max_by(|a, b| a.1.total_cmp(&b.1));
    let Some((name, chord)) = worst else {
        return Ok(DEV_TOL);
    };
    excess_floor(DEV_TOL, excess(chord)).ok_or_else(|| {
        format!("model '{name}' was {TOO_COARSE} (surfaces are approximated to within {})", within(world, chord))
    })
}
