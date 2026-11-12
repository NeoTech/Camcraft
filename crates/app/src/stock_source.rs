//! The from-model stock's source model and the program verdict (#1094).
//! A casting imported to be copied into a "from model" stock and left in
//! the tree is stock, not the finished part - but only the user can say
//! so: any rule that guesses which model is the part can be met by a job
//! it did not foresee, and a wrong guess posts a gouge. So the verdict
//! leaves a model out only when the user ticked "The source model is raw
//! stock only" (`schema::StockSourceOnly`, `SetStockSourceOnly`); the one
//! place this module infers anything is the refusal's hint, which changes
//! no verdict.
//!
//! Provenance: our own design notes (a stock is a copy of its source
//! model, M3.1 B; a CAM choice is a user parameter with a safe default);
//! see `docs/decisions/1094-stock-source-not-part.md`.

use flecs_ecs::prelude::*;

use crate::model;
use crate::schema::{Operation, StockSourceOnly};
use crate::stock::StockShape;

/// The label of the stock page's checkbox, said once (UI, MCP, hints).
pub const RAW_STOCK_ONLY: &str = "The source model is raw stock only (not judged as the part)";

/// Does `op` name the model `name` anywhere - its source (one entity or
/// a feature selection) or a boundary pick of its area controls?
fn names(op: &Operation, name: &str) -> bool {
    let in_src = match &op.src {
        crate::operation::SrcRef::Stock => false,
        crate::operation::SrcRef::Entity(n) => n == name,
        crate::operation::SrcRef::Selection(items) => items.iter().any(|g| g.entity == name),
    };
    in_src || op.area.picks().any(|g| g.entity == name)
}

fn all_ops(world: &World) -> Vec<Operation> {
    crate::setup::operation_order(world)
        .iter()
        .filter_map(|(_, e)| model::operation_of(world, e))
        .collect()
}

/// The stock entities the program starts from: the own stock of every
/// setup its unsuppressed operations machine in (a setup that takes what
/// the one before leaves starts from no stock of its own); with no
/// operation, the active setup's.
fn program_stocks(world: &World, ops: &[Operation]) -> Vec<Entity> {
    let mut contexts: Vec<Option<String>> = Vec::new();
    for op in ops.iter().filter(|op| !op.suppressed) {
        let eff = crate::setup::effective(world, op.setup.as_deref());
        if !contexts.contains(&eff) {
            contexts.push(eff);
        }
    }
    if contexts.is_empty() {
        contexts.push(crate::setup::effective(world, crate::setup::active(world).as_deref()));
    }
    contexts
        .iter()
        .filter(|c| {
            !c.as_deref().is_some_and(|s| {
                crate::setup_stock::effective(world, s) == Some(crate::setup_stock::StockSource::FromPrevious)
            })
        })
        .filter_map(|c| model::find_by_name(world, &crate::setup::stock_entity_name(world, c.as_deref())))
        .collect()
}

/// The source model of a from-model stock, and whether the user marked
/// it raw stock only (the mark honoured only while it names that source).
fn source_of(world: &World, stock: &Entity) -> Option<(String, bool)> {
    let Some(StockShape::FromModel { source }) = model::stock_def_of(world, stock) else {
        return None;
    };
    let marked = stock
        .entity_view(world)
        .try_get::<&StockSourceOnly>(|f| f.source == source)
        .unwrap_or(false);
    Some((source, marked))
}

/// The model the program verdict leaves out, if any: the source of a
/// from-model stock the user marked raw stock only. All must hold, else
/// every non-fixture model is judged (the default):
///
/// 1. every stock the program starts from is a copy of this one model,
///    and every one of them carries the mark;
/// 2. no operation of the document - any setup, suppressed or not - names
///    it in its source or its boundary picks: an op that machines it
///    makes it the part again, mark or no mark;
/// 3. it is a model and not workholding.
///
/// Read from the live document on every call, so undo/redo of the mark,
/// of `SetStock` (a new source drops the mark's hold) and of any op land
/// on the same answer.
#[must_use]
pub fn stock_only_model(world: &World) -> Option<Entity> {
    let ops = all_ops(world);
    let mut source: Option<String> = None;
    for stock in program_stocks(world, &ops) {
        let (s, marked) = source_of(world, &stock)?;
        if !marked || source.as_ref().is_some_and(|prev| *prev != s) {
            return None;
        }
        source = Some(s);
    }
    let source = source?;
    if ops.iter().any(|op| names(op, &source)) {
        return None;
    }
    let e = model::find_by_name(world, &source)?;
    (model::is_model(world, &e) && !model::is_fixture(world, &e)).then_some(e)
}

/// Is the model `entity` one of the parts the program verdict judges: not
/// workholding, and not [`stock_only_model`]? `skip` is that model,
/// computed once per walk by the caller.
#[must_use]
pub fn judged(world: &World, entity: &Entity, skip: Option<Entity>) -> bool {
    !model::is_fixture(world, entity) && skip != Some(*entity)
}

/// The refusal when the mark leaves nothing to judge: the stock's source
/// is the only non-fixture model, so no finished part is in the document.
/// A program judged against nothing is never passed silently.
#[must_use]
pub fn nothing_judged(world: &World) -> Option<String> {
    let skip = stock_only_model(world)?;
    let any_part = model::model_entities(world)
        .into_iter()
        .any(|(_, e)| judged(world, &e, Some(skip)));
    if any_part {
        return None;
    }
    let name = model::name_of(world, &skip).unwrap_or_default();
    Some(format!(
        "nothing is judged as the part: '{name}' is marked raw stock only and the document has no other model - import the finished part, or untick \"{RAW_STOCK_ONLY}\" on the stock page"
    ))
}

/// One plain sentence for a gouge refusal when the model it is read
/// against includes the program's stock source: how to tell the verdict
/// that model is raw stock - or why the mark did not hold. The only
/// inference in this module; it changes no verdict.
#[must_use]
pub fn gouge_hint(world: &World) -> Option<String> {
    let ops = all_ops(world);
    let skip = stock_only_model(world);
    for stock in program_stocks(world, &ops) {
        let Some((source, marked)) = source_of(world, &stock) else { continue };
        let Some(e) = model::find_by_name(world, &source) else { continue };
        if skip == Some(e) || !model::is_model(world, &e) || model::is_fixture(world, &e) {
            continue;
        }
        if ops.iter().any(|op| names(op, &source)) {
            return marked.then(|| format!(
                " - '{source}' is marked raw stock only, but an operation machines it, so it is judged as the part"
            ));
        }
        return Some(format!(
            " - the stock is made from '{source}', which is judged as the part: if '{source}' is only the raw stock (a casting or blank) and the finished part is its own model, tick \"{RAW_STOCK_ONLY}\" on the stock page"
        ));
    }
    None
}

/// What the stock page's checkbox shows for setup context `setup`:
/// `None` when its stock is not made from a model (no checkbox), else
/// whether the mark holds on the source the stock copies now.
#[must_use]
pub fn raw_stock_only_of(world: &World, setup: Option<&str>) -> Option<bool> {
    let stock = model::find_by_name(world, &crate::setup::stock_entity_name(world, setup))?;
    source_of(world, &stock).map(|(_, marked)| marked)
}
