//! Picks that follow their face by SolidCraft name (#3915): given two
//! revisions of one model, each with its source face record
//! ([`crate::source_faces`]), find the one new face or patch that carries
//! the SolidCraft names of a face or patch whose geometry-hash id went
//! missing.
//!
//! Provenance: our own design (#4213, `docs/decisions/4213-sc-face-name-follow.md`).
//! The names are SolidCraft's own face identity, read from its published
//! model cache (SolidCraft `crates/doc/src/model_cache.rs`, v0.50.0); the
//! match is plain set equality. No vendor material consulted.
//!
//! ## The law here
//!
//! `revision` pairs no missing face with an added one by guessing:
//! re-picking is "made visible to the user, never assumed". A SolidCraft
//! name is not a guess - it is SolidCraft's identity for the face - but a
//! pick follows only when the names leave no doubt:
//!
//! - the old feature is the only one of its kind with its id and with its
//!   names, and every one of its names is verified (it follows exact
//!   history in SolidCraft);
//! - exactly one new feature of the same kind has the same names (a
//!   record face's `former` name counts as its name too, so the pieces of
//!   a Split Face still answer to the face they came from);
//! - that new feature was not in the old revision, its names are
//!   verified, a face keeps its plane kind, and it answers to the names
//!   of no other old feature.
//!
//! Anything else is a [`Lost`] verdict with its reason: the pick stays as
//! it is and generation refuses it, exactly as before #3915. A wrong
//! follow is the failure this module exists to prevent; a missed follow
//! is only today's behaviour.

use std::collections::{BTreeSet, HashMap};

use crate::geometry::feature::{FaceKind, Features, patches};
use crate::source_faces::SourceFaces;

/// The kind of feature a pick names: a planar `Face` or a smooth `Patch`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickKind {
    Face,
    Patch,
}

/// Why a pick did not follow its names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lost {
    /// Two features of the old revision carry the pick's id.
    TwinId,
    /// A facet of the old feature has no record face.
    Unnamed,
    /// SolidCraft did not verify this name of the old feature.
    Unverified(String),
    /// Another feature of the same kind had the same names before.
    TwinNames,
    /// No new feature has these names.
    NoMatch,
    /// This many new features of the same kind have these names.
    Many(usize),
    /// The only new feature with these names is of another kind: a flat
    /// face that is now curved, a curved face that is now flat, or a face
    /// that has no plane now.
    KindChanged,
    /// The only new feature with these names was in the old revision too,
    /// under other names.
    NotNew,
    /// The only new feature with these names also answers to the names of
    /// another old feature (through a `former` name): two picks would meet
    /// on one face.
    Shared,
    /// SolidCraft did not verify the name of the only new feature.
    NewUnverified,
}

impl Lost {
    /// The reason in a few words, for the re-import report.
    #[must_use]
    pub fn reason(&self) -> String {
        match self {
            Lost::TwinId => "two faces had its id".into(),
            Lost::Unnamed => "part of it had no SolidCraft name".into(),
            Lost::Unverified(name) => format!("SolidCraft did not verify the name '{name}'"),
            Lost::TwinNames => "another face had the same names".into(),
            Lost::NoMatch => "no face has these names now".into(),
            Lost::Many(n) => format!("{n} faces have these names now"),
            Lost::KindChanged => "the face with these names changed kind".into(),
            Lost::NotNew => "the face with these names was in the model before".into(),
            Lost::Shared => "the face with these names also carries another face's name".into(),
            Lost::NewUnverified => "SolidCraft did not verify the new face's name".into(),
        }
    }
}

/// What the follow says about one pick whose id went missing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    /// The old feature's SolidCraft names, sorted; empty when it had none
    /// ([`Lost::Unnamed`], [`Lost::TwinId`]).
    pub names: Vec<String>,
    /// The new id, or why there is none.
    pub to: Result<u64, Lost>,
}

/// One feature of a revision: its id, its plane kind (`None` for a
/// patch) and its record faces, sorted; `None` when a facet has none.
struct Entry {
    id: u64,
    kind: Option<FaceKind>,
    record: Option<Vec<u32>>,
}

/// One revision of a model, indexed for the follow.
pub struct Revision<'a> {
    record: &'a SourceFaces,
    faces: Vec<Entry>,
    patches: Vec<Entry>,
}

impl<'a> Revision<'a> {
    /// Index `features` with `record`. `None` when the record does not
    /// tile the facets the features were built from: a record that
    /// speaks for other facets is never read.
    #[must_use]
    pub fn new(features: &Features, record: &'a SourceFaces) -> Option<Self> {
        if !record.tiles(features.facet_face.len()) {
            return None;
        }
        let faces: Vec<Entry> = features
            .faces
            .iter()
            .map(|f| Entry {
                id: f.id,
                kind: Some(f.kind),
                record: record_faces(record, f.facets.iter().copied()),
            })
            .collect();
        let mut by_id: HashMap<u64, usize> = HashMap::with_capacity(faces.len());
        for (i, e) in faces.iter().enumerate() {
            by_id.entry(e.id).or_insert(i);
        }
        let patches = patches(features)
            .into_iter()
            .map(|p| {
                let mut union: BTreeSet<u32> = BTreeSet::new();
                let mut whole = true;
                for fid in &p.faces {
                    match by_id
                        .get(fid)
                        .and_then(|&i| faces.get(i))
                        .and_then(|e| e.record.as_ref())
                    {
                        Some(r) => union.extend(r.iter().copied()),
                        None => whole = false,
                    }
                }
                Entry {
                    id: p.id,
                    kind: None,
                    record: (whole && !union.is_empty()).then(|| union.into_iter().collect()),
                }
            })
            .collect();
        Some(Revision {
            record,
            faces,
            patches,
        })
    }

    fn entries(&self, kind: PickKind) -> &[Entry] {
        match kind {
            PickKind::Face => &self.faces,
            PickKind::Patch => &self.patches,
        }
    }

    fn count(&self, kind: PickKind, id: u64) -> usize {
        self.entries(kind).iter().filter(|e| e.id == id).count()
    }

    /// The names of `faces`, sorted and deduplicated; `None` when one
    /// index passes the name list.
    fn names_of(&self, faces: &[u32]) -> Option<Vec<&str>> {
        let mut out: Vec<&str> = Vec::with_capacity(faces.len());
        for &r in faces {
            out.push(self.name(r)?.name.as_str());
        }
        out.sort_unstable();
        out.dedup();
        Some(out)
    }

    fn name(&self, r: u32) -> Option<&crate::source_faces::FaceName> {
        usize::try_from(r)
            .ok()
            .and_then(|i| self.record.names.get(i))
    }

    fn verified(&self, faces: &[u32]) -> bool {
        faces
            .iter()
            .all(|&r| self.name(r).is_some_and(|n| n.verified))
    }

    /// The names `key` and the record faces `faces` say the same face:
    /// each name is a name or the former name of one of the faces, and
    /// each face answers to one of the names.
    fn matches(&self, key: &[&str], faces: &[u32]) -> bool {
        let alias = |r: u32, n: &str| {
            self.name(r)
                .is_some_and(|f| f.name == n || f.former.as_deref() == Some(n))
        };
        !faces.is_empty()
            && key.iter().all(|n| faces.iter().any(|&r| alias(r, n)))
            && faces.iter().all(|&r| key.iter().any(|n| alias(r, n)))
    }

    fn candidates(&self, kind: PickKind, key: &[&str]) -> Vec<&Entry> {
        self.entries(kind)
            .iter()
            .filter(|e| e.record.as_deref().is_some_and(|r| self.matches(key, r)))
            .collect()
    }
}

/// The record faces under `facets`, sorted; `None` when one facet has none.
fn record_faces(record: &SourceFaces, facets: impl Iterator<Item = u32>) -> Option<Vec<u32>> {
    let mut set: BTreeSet<u32> = BTreeSet::new();
    for t in facets {
        set.insert(record.face_of(usize::try_from(t).ok()?)?);
    }
    (!set.is_empty()).then(|| set.into_iter().collect())
}

/// Where a pick of `kind` with `id` goes from `old` to `new`. `None` when
/// it has nothing to follow: the id is still in `new` (the face is
/// carried, and the pick is untouched), or it was not in `old` either
/// (the pick was lost before this import).
#[must_use]
pub fn follow(old: &Revision, new: &Revision, kind: PickKind, id: u64) -> Option<Verdict> {
    if new.count(kind, id) > 0 {
        return None;
    }
    let lost = |names: Vec<String>, why: Lost| {
        Some(Verdict {
            names,
            to: Err(why),
        })
    };
    let mut mine = old.entries(kind).iter().filter(|e| e.id == id);
    let entry = mine.next()?;
    if mine.next().is_some() {
        return lost(Vec::new(), Lost::TwinId);
    }
    let Some(faces) = entry.record.as_deref() else {
        return lost(Vec::new(), Lost::Unnamed);
    };
    let Some(key) = old.names_of(faces) else {
        return lost(Vec::new(), Lost::Unnamed);
    };
    let names: Vec<String> = key.iter().map(|n| (*n).to_owned()).collect();
    if let Some(r) = faces
        .iter()
        .find(|&&r| !old.name(r).is_some_and(|n| n.verified))
    {
        let name = old.name(*r).map(|n| n.name.clone()).unwrap_or_default();
        return lost(names, Lost::Unverified(name));
    }
    let twins = old
        .entries(kind)
        .iter()
        .filter(|e| e.id != id)
        .filter(|e| {
            e.record
                .as_deref()
                .and_then(|r| old.names_of(r))
                .is_some_and(|n| n == key)
        })
        .count();
    if twins > 0 {
        return lost(names, Lost::TwinNames);
    }
    let found = new.candidates(kind, &key);
    let to = match found.as_slice() {
        [] => {
            let other = match kind {
                PickKind::Face => PickKind::Patch,
                PickKind::Patch => PickKind::Face,
            };
            Err(if new.candidates(other, &key).is_empty() {
                Lost::NoMatch
            } else {
                Lost::KindChanged
            })
        }
        [one] if one.kind != entry.kind => Err(Lost::KindChanged),
        [one] if old.count(kind, one.id) > 0 => Err(Lost::NotNew),
        [one] if !one.record.as_deref().is_some_and(|r| new.verified(r)) => {
            Err(Lost::NewUnverified)
        }
        [one] if claimants(old, new, kind, one) > 1 => Err(Lost::Shared),
        [one] => Ok(one.id),
        many => Err(Lost::Many(many.len())),
    };
    Some(Verdict { names, to })
}

/// How many old features of `kind` the new feature `one` answers to by
/// name: the pick's own, and any other whose names it carries through a
/// `former` name. More than one, and two picks would meet on one face.
fn claimants(old: &Revision, new: &Revision, kind: PickKind, one: &Entry) -> usize {
    let Some(faces) = one.record.as_deref() else {
        return 0;
    };
    old.entries(kind)
        .iter()
        .filter(|e| {
            e.record
                .as_deref()
                .and_then(|r| old.names_of(r))
                .is_some_and(|k| new.matches(&k, faces))
        })
        .count()
}

#[cfg(test)]
mod tests;
