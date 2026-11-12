//! The model that is only the stock's source (#1094): a casting imported
//! to be copied into a `FromModel` stock and left in the tree is stock,
//! not the finished part, so the program verdict must not read it as
//! geometry to leave standing.
//!
//! Provenance: our own design notes on the document model (a stock is a
//! copy of its source model, M3.1 B); the rule is the Prime Directive's
//! narrowest reading - see `docs/decisions/1094-stock-source-not-part.md`.

use flecs_ecs::prelude::{Entity, World};

use crate::model;
use crate::operation::SrcRef;
use crate::stock::StockShape;

/// Every model name an operation's source names (a single entity, or the
/// entities of a feature selection); the setup's stock names none.
fn named_by(src: &SrcRef) -> Vec<&str> {
    match src {
        SrcRef::Stock => Vec::new(),
        SrcRef::Entity(n) => vec![n.as_str()],
        SrcRef::Selection(items) => items.iter().map(|g| g.entity.as_str()).collect(),
    }
}

/// The model the program verdict leaves out because it is ONLY the
/// stock's source, if there is one. All three must hold, else every
/// non-fixture model stays judged (the pre-#1094 reading):
///
/// 1. the stock of EVERY setup the program machines in (the unsuppressed
///    operations' setups; with none, the active setup) is a `FromModel`
///    copy of this one model - a setup cut from other stock still judges it;
/// 2. no operation of the document - any setup, suppressed or not - names
///    it in its source: picked for an op, it is the part again;
/// 3. some operation names another non-fixture model in its source - a
///    separate part is there to be judged instead (a stock made from the
///    part itself, with every op on the stock, keeps the part judged).
///
/// Read from the live document on every call: no stored mark, so undo or
/// redo of `SetStock` or of an op's source, and a project file saved
/// before #1094, all land on the same answer.
#[must_use]
pub fn stock_only_model(world: &World) -> Option<Entity> {
    let ops: Vec<crate::schema::Operation> = crate::setup::operation_order(world)
        .iter()
        .filter_map(|(_, e)| model::operation_of(world, e))
        .collect();
    let mut stocks: Vec<String> = ops
        .iter()
        .filter(|op| !op.suppressed)
        .map(|op| crate::setup::stock_entity_name(world, op.setup.as_deref()))
        .collect();
    if stocks.is_empty() {
        stocks.push(crate::setup::stock_entity_name(world, crate::setup::active(world).as_deref()));
    }
    let mut source: Option<String> = None;
    for name in &stocks {
        let stock = model::find_by_name(world, name)?;
        let Some(StockShape::FromModel { source: s }) = model::stock_def_of(world, &stock) else {
            return None;
        };
        match &source {
            Some(prev) if *prev != s => return None,
            _ => source = Some(s),
        }
    }
    let source = source?;
    let named: Vec<&str> = ops.iter().flat_map(|op| named_by(&op.src)).collect();
    if named.contains(&source.as_str()) {
        return None;
    }
    let separate_part = named.iter().any(|n| {
        model::find_by_name(world, n).is_some_and(|e| model::is_model(world, &e) && !model::is_fixture(world, &e))
    });
    if !separate_part {
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
