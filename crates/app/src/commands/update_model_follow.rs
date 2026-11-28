//! Picks follow their faces by SolidCraft name when a model imports again
//! (#3915): the undoable half of the follow, inside `UpdateModel`.
//!
//! Provenance: our own design (#4213, `docs/decisions/4213-sc-face-name-follow.md`);
//! the match itself is `camcraft_geometry::source_follow`. No vendor
//! material consulted.
//!
//! ## The law here
//!
//! - The follow runs only when the old model and the new mesh both hold a
//!   source face record, and only from `UpdateModel::with_source_faces` -
//!   the step that brings the new record - never from `UpdateModel::new`.
//! - It rewrites only `Face` and `Patch` picks on this model, stored
//!   inside operations (`stores`); a carried pick is untouched. Setup WCS
//!   and alignment picks, fixture-locate picks and electrodes are not
//!   followed (D7).
//! - It is visible, never assumed (`revision`'s law): each operation with
//!   a followed pick gets one `needs_review` line, which holds the post
//!   until the user marks the operation reviewed; each pick that could
//!   not follow is named in the re-import report and stays as it was, so
//!   generation refuses it exactly as before.
//! - Everything lands in the update's own undo step: the operations are
//!   snapshotted whole at construction and written back by `invert`.

use std::sync::Arc;

use camcraft_geometry::source_faces::SourceFaces;
use camcraft_geometry::source_follow::{self, PickKind, Revision};
use flecs_ecs::prelude::World;

use crate::geometry::Triangle;
use crate::geometry::feature;
use crate::model;
use crate::operation::FeatureSel;
use crate::schema::{Operation, OperationMeta};

mod report;
mod stores;

pub use report::{FollowReport, PickNote};
use report::{noun, review_line};

/// Why the pick of a Probe or a Slot does not follow (`stores::seeds_from_picks`).
const SEEDED: &str = "its pick set the operation's numbers - pick it again";

/// One operation the follow rewrites: as it stood, and after.
struct OpChange {
    key: String,
    prev: Operation,
    next: Operation,
    prev_meta: OperationMeta,
    next_meta: OperationMeta,
}

/// An operation that holds a `Face` or `Patch` pick on the model, as it
/// stood at construction: key, label, definition and meta.
type Held = (String, String, Operation, OperationMeta);

/// The follow of one `UpdateModel`: the snapshot taken at construction,
/// then what [`Follow::plan`] decided once the new record arrived.
#[derive(Default)]
pub(crate) struct Follow {
    old: Option<Arc<SourceFaces>>,
    held: Vec<Held>,
    changes: Vec<OpChange>,
    report: FollowReport,
}

impl Follow {
    /// Snapshot the old record of model `name` and every operation that
    /// holds a `Face` or `Patch` pick on it. A model with no record
    /// snapshots nothing: no follow can come of it.
    pub(crate) fn snapshot(world: &World, name: &str) -> Self {
        let Some(entity) = model::find_by_name(world, name) else {
            return Self::default();
        };
        let Some(old) = crate::source_faces::of(world, &entity) else {
            return Self::default();
        };
        let held = crate::setup::operation_order(world)
            .into_iter()
            .filter_map(|(key, e)| {
                let op = model::operation_of(world, &e)?;
                stores::holds_face_pick(&op, name).then(|| {
                    let label = model::operation_label(world, &key, &e);
                    (key, label, op, model::operation_meta_of(world, &e))
                })
            })
            .collect();
        Follow {
            old: Some(old),
            held,
            ..Self::default()
        }
    }

    /// Decide the follow from the old facets to the new ones, with the
    /// new record. Called again, it starts over. Nothing follows unless
    /// both records tile the facets they ride.
    pub(crate) fn plan(
        &mut self,
        name: &str,
        old_facets: Option<&Arc<[Triangle]>>,
        new_facets: &Arc<[Triangle]>,
        new: Option<&Arc<SourceFaces>>,
    ) {
        self.changes.clear();
        self.report = FollowReport::default();
        let (Some(old), Some(new), Some(old_facets)) =
            (self.old.as_deref(), new.map(Arc::as_ref), old_facets)
        else {
            return;
        };
        if self.held.is_empty() {
            return;
        }
        if !old.tiles(old_facets.len()) || !new.tiles(new_facets.len()) {
            tracing::warn!(
                "model '{name}': a source face record does not tile its facets; no pick follows by name"
            );
            return;
        }
        // Built here, never from the features caches: the content digest
        // of `model::features_of_local` is order-free, so a cache hit can
        // carry another mesh's facet order, and the record would then name
        // the wrong facets. The ids come out the same (they hash positions,
        // never indexes), so the stored picks still match.
        let (fa, fb) = (
            feature::features_of_facets(old_facets),
            feature::features_of_facets(new_facets),
        );
        let (Some(ra), Some(rb)) = (Revision::new(&fa, old), Revision::new(&fb, new)) else {
            return;
        };
        let mut lost: Vec<PickNote> = Vec::new();
        let mut followed_all: Vec<PickNote> = Vec::new();
        let mut changes: Vec<OpChange> = Vec::new();
        for (key, label, prev, prev_meta) in &self.held {
            // A Probe's or a Slot's pick set its numbers; it never follows.
            let seeded = stores::seeds_from_picks(prev);
            let mut next = prev.clone();
            let mut followed: Vec<PickNote> = Vec::new();
            stores::for_each_pick_mut(&mut next, |g| {
                if g.entity != name {
                    return;
                }
                let (kind, id) = match g.feature {
                    FeatureSel::Face(id) => (PickKind::Face, id),
                    FeatureSel::Patch(id) => (PickKind::Patch, id),
                    _ => return,
                };
                let Some(v) = source_follow::follow(&ra, &rb, kind, id) else {
                    return;
                };
                let mut note = PickNote {
                    operation: key.clone(),
                    label: label.clone(),
                    kind: noun(kind),
                    names: v.names,
                    why: None,
                };
                match v.to {
                    Ok(_) if seeded => {
                        note.why = Some(SEEDED.to_owned());
                        lost.push(note);
                    }
                    Ok(to) => {
                        g.feature = match kind {
                            PickKind::Face => FeatureSel::Face(to),
                            PickKind::Patch => FeatureSel::Patch(to),
                        };
                        followed.push(note);
                    }
                    Err(why) => {
                        note.why = Some(why.reason());
                        lost.push(note);
                    }
                }
            });
            if followed.is_empty() {
                continue;
            }
            let mut next_meta = prev_meta.clone();
            let line = review_line(name, &followed);
            if !next_meta.needs_review.contains(&line) {
                next_meta.needs_review.push(line);
            }
            changes.push(OpChange {
                key: key.clone(),
                prev: prev.clone(),
                next,
                prev_meta: prev_meta.clone(),
                next_meta,
            });
            followed_all.extend(followed);
        }
        self.changes = changes;
        self.report = FollowReport {
            followed: followed_all,
            lost,
        };
    }

    /// `true`: land the followed picks and the review lines (apply);
    /// `false`: put the operations back as they stood (invert).
    pub(crate) fn write(&self, world: &World, forward: bool) {
        for c in &self.changes {
            if let Some(e) = model::find_by_name(world, &c.key) {
                let (op, meta) = if forward {
                    (&c.next, &c.next_meta)
                } else {
                    (&c.prev, &c.prev_meta)
                };
                e.entity_view(world).set(op.clone());
                e.entity_view(world).set(meta.clone());
            }
        }
    }

    pub(crate) fn report(&self) -> &FollowReport {
        &self.report
    }
}
