//! #3915, for #3814: the `import_model` answer for an update says what
//! moved between the two revisions of the model. `changes` gives, for the
//! faces, the welded faces (patches) and the edge chains, how many ids the
//! update carried, added and lost, and `summary` is the phrase the window
//! shows after a re-import (`FeatureChanges::describe`). The diff is
//! `revision::changes`, the same one `UpdateModel::changes` gives the
//! window. An import of a new model, or a call that did not update the
//! model it names, adds nothing: the key is optional.
//!
//! Provenance: our own read-only MCP view over `geometry::revision` (this
//! repo's M3.1 design); no vendor material consulted.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::geometry::Triangle;
use crate::geometry::revision::{self, FeatureChanges, IdDiff};
use crate::model;
use crate::script::Env;

/// One model's facets (shared, so no triangle is copied) and its revision
/// counter, as they stood.
type Snapshot = (Option<Arc<[Triangle]>>, u64);

/// Each model's snapshot before an import: the side of the diff that an
/// update replaces.
pub(super) struct Before(HashMap<String, Snapshot>);

/// Snapshot every model now. The import names its model only when it runs
/// (the given name, or the file stem), so the answer's own `model` selects
/// the entry afterwards, and the stem rule is not copied here.
pub(super) fn before(env: &Env) -> Before {
    let w = &env.world;
    Before(
        model::named_entities(w)
            .into_iter()
            .filter(|(_, e)| model::is_model(w, e))
            .map(|(name, e)| (name, (model::facets_of(w, &e), model::rev_of(w, &e))))
            .collect(),
    )
}

/// Add `changes` to an `import_model` answer when the call updated a model
/// that was there before it (its revision counter moved). Any other answer
/// stays as it is.
pub(super) fn add(env: &Env, before: &Before, out: &mut Value) {
    let Some(name) = out.get("model").and_then(Value::as_str).map(str::to_owned) else {
        return;
    };
    let Some((old, rev)) = before.0.get(&name) else {
        return;
    };
    let w = &env.world;
    let Some(e) = model::find_by_name(w, &name) else {
        return;
    };
    if model::rev_of(w, &e) <= *rev {
        return;
    }
    let features = |tris: Option<&Arc<[Triangle]>>| match tris {
        Some(t) => model::features_of_shared(t),
        None => model::features_of_local(&[]),
    };
    let now = model::facets_of(w, &e);
    let changes = revision::changes(&features(old.as_ref()), &features(now.as_ref()));
    out["changes"] = view(&changes);
}

/// The `changes` object: the carried, added and missing counts of each id
/// domain, and the window's one-line phrase.
fn view(ch: &FeatureChanges) -> Value {
    let counts = |d: &IdDiff| json!({ "carried": d.carried.len(), "added": d.added.len(), "missing": d.missing.len() });
    json!({
        "faces": counts(&ch.faces),
        "patches": counts(&ch.patches),
        "chains": counts(&ch.chains),
        "summary": ch.describe(),
    })
}
