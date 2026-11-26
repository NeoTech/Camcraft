//! #3915: SolidCraft face names over MCP. When a model holds the source
//! face record of a `.sc` import, the `list_features` face and patch rows,
//! and each face or patch pick that `picks_view` shows
//! (`get_operation_geometry`, `get_selection`, the area and "Do not touch"
//! views, `pick_in_box`), get `source_name` (one name) or `source_names`
//! (several, sorted), and `source_unverified: true` when SolidCraft has not
//! verified one of them. Optional keys only: a model with no record, a
//! record that does not fit its facets, or a pick that is no longer in the
//! model shows its rows as before.
//!
//! The names need facet indexes in the model's own facet order, so the
//! features here are built from the model's facets (`features_of_facets`)
//! and never taken from the content-keyed caches: a cache hit from a mesh
//! of the same content in another facet order has other facet indexes. The
//! feature ids are order-free, so they are the ids of the listed rows.
//!
//! Provenance: our own read-only MCP view over `geometry::source_names` and
//! the model's source face record (`crate::source_faces`); no vendor
//! material consulted.

use std::collections::HashMap;
use std::sync::Arc;

use flecs_ecs::prelude::{Entity, World};
use serde_json::{Value, json};

use super::pick::parse_feature_id;
use crate::geometry::feature::{self, Features, Patch};
use crate::geometry::source_names::{NameSet, SourceNames};
use crate::model;
use crate::operation::{FeatureSel, GeoSel};

type Record = Arc<crate::geometry::source_faces::SourceFaces>;

/// The model's source face record: the one call into `crate::source_faces`.
fn record_of(w: &World, e: &Entity) -> Option<Record> {
    crate::source_faces::of(w, e)
}

/// One model's record, its features built from its own facets, and its
/// patches by id when they are asked for.
struct Named {
    record: Record,
    features: Features,
    patches: HashMap<u64, Patch>,
}

impl Named {
    fn of(w: &World, e: &Entity, with_patches: bool) -> Option<Self> {
        let record = record_of(w, e)?;
        let tris = model::facets_of(w, e)?;
        let features = feature::features_of_facets(&tris);
        let patches = if with_patches {
            feature::patches(&features).into_iter().map(|p| (p.id, p)).collect()
        } else {
            HashMap::new()
        };
        Some(Self { record, features, patches })
    }

    fn lookup(&self) -> Option<SourceNames<'_>> {
        SourceNames::new(&self.record, &self.features)
    }

    fn names_of<'a>(&'a self, names: &SourceNames<'a>, sel: &FeatureSel) -> Option<NameSet<'a>> {
        match sel {
            FeatureSel::Face(id) => names.face(*id),
            FeatureSel::Patch(id) => names.patch(self.patches.get(id)?),
            _ => None,
        }
    }
}

/// Write a name set into one row; an empty set writes nothing.
fn put(row: &mut Value, set: &NameSet<'_>) {
    let Some(obj) = row.as_object_mut() else {
        return;
    };
    let (key, value) = match set.names.as_slice() {
        [] => return,
        [one] => ("source_name", json!(one)),
        many => ("source_names", json!(many)),
    };
    obj.insert(key.to_owned(), value);
    if set.unverified {
        obj.insert("source_unverified".to_owned(), json!(true));
    }
}

/// The names for one `list_features` call, built once for all its kinds.
pub(super) struct Listing(Option<Named>);

impl Listing {
    /// Nothing for a model, fixture or stock that holds no record, and no
    /// work when `kinds` lists neither faces nor patches.
    pub(super) fn of(w: &World, e: &Entity, kinds: &[&str]) -> Self {
        if !kinds.iter().any(|k| matches!(*k, "faces" | "patches")) {
            return Self(None);
        }
        Self(Named::of(w, e, kinds.contains(&"patches")))
    }

    /// The `kind` rows with the names added: face and patch rows only, found
    /// by their `id`. Other rows pass through as they are.
    pub(super) fn annotate(&self, kind: &str, mut rows: Vec<Value>) -> Vec<Value> {
        if !matches!(kind, "faces" | "patches") {
            return rows;
        }
        let Some(named) = &self.0 else {
            return rows;
        };
        let Some(names) = named.lookup() else {
            return rows;
        };
        for row in &mut rows {
            let sel = row.get("id").and_then(Value::as_str).and_then(|s| parse_feature_id(s).ok());
            if let Some(set) = sel.and_then(|s| named.names_of(&names, &s)) {
                put(row, &set);
            }
        }
        rows
    }
}

/// `picks_view` rows (one for each pick, in order) with the names added to
/// the face and patch picks that are still in a model with a record.
pub(super) fn annotate_picks(w: &World, items: &[GeoSel], mut rows: Vec<Value>) -> Vec<Value> {
    let named_kind = |f: &FeatureSel| matches!(f, FeatureSel::Face(_) | FeatureSel::Patch(_));
    // Each model once, with its patches only when a patch is picked on it.
    let mut wants: HashMap<&str, bool> = HashMap::new();
    for g in items.iter().filter(|g| named_kind(&g.feature)) {
        *wants.entry(g.entity.as_str()).or_insert(false) |= matches!(g.feature, FeatureSel::Patch(_));
    }
    if wants.is_empty() {
        return rows;
    }
    let models: HashMap<&str, Named> = wants
        .into_iter()
        .filter_map(|(name, with_patches)| {
            let e = model::find_by_name(w, name)?;
            Some((name, Named::of(w, &e, with_patches)?))
        })
        .collect();
    let lookups: HashMap<&str, (&Named, SourceNames<'_>)> =
        models.iter().filter_map(|(name, m)| Some((*name, (m, m.lookup()?)))).collect();
    if lookups.is_empty() {
        return rows;
    }
    for (row, g) in rows.iter_mut().zip(items) {
        if let Some((m, names)) = lookups.get(g.entity.as_str())
            && let Some(set) = m.names_of(names, &g.feature)
        {
            put(row, &set);
        }
    }
    rows
}
