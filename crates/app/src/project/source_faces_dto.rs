//! The file mirror of a `.sc` model's source face record (#3915,
//! `schema::SourceFaces`): SolidCraft's name for each record face, and the
//! facet ranges each record face became.
//!
//! No version step (decision D4, the `solids` #917 precedent): an old
//! reader drops the key and reads the model exactly as it does today; it
//! loses only the follow by name at the next re-import. So the key never
//! refuses a file: a record that does not read, or does not tile the
//! facets it rides, is dropped at load with a logged note, never trusted.
//! Provenance: our own DTO, the pattern of the other entity keys.

use std::sync::Arc;

use flecs_ecs::prelude::{Entity, World};
use serde::{Deserialize, Deserializer, Serialize};

use camcraft_geometry::source_faces::{FaceName, FacetRange, SourceFaces};

use crate::model;

/// The longest face name a file may hold, in bytes: the `.sc` reader's
/// own bound (a longer name is damage, and the record is dropped).
const NAME_MAX: usize = 1 << 14;

/// One record face's SolidCraft name. `verified` absent is false: a lost
/// flag can only stop a follow, never start one.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FaceNameDto {
    pub name: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub former: Option<String>,
}

/// The record: the names in record order, and `[start, end, face]`
/// triples - facets `start..end` came from record face `face`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SourceFacesDto {
    pub names: Vec<FaceNameDto>,
    pub ranges: Vec<[usize; 3]>,
}

impl SourceFacesDto {
    #[must_use]
    pub fn of(record: &SourceFaces) -> Self {
        SourceFacesDto {
            names: record
                .names
                .iter()
                .map(|n| FaceNameDto {
                    name: n.name.clone(),
                    verified: n.verified,
                    former: n.former.clone(),
                })
                .collect(),
            ranges: record
                .ranges
                .iter()
                .map(|r| [r.start, r.end, r.face as usize])
                .collect(),
        }
    }

    /// The record for a model of `len` facets, or `None` with a logged
    /// note naming `owner`: a face index past `u32`, a name past
    /// [`NAME_MAX`], or ranges that do not tile the facets.
    #[must_use]
    pub fn to_record(&self, len: usize, owner: &str) -> Option<SourceFaces> {
        let dropped = |why: &str| {
            tracing::warn!(
                "project: model '{owner}' has a source face record that {why}; it is dropped, and its picks will not follow SolidCraft names"
            );
            None
        };
        let long = |s: &str| s.len() > NAME_MAX;
        if self
            .names
            .iter()
            .any(|n| long(&n.name) || n.former.as_deref().is_some_and(long))
        {
            return dropped("holds a name too long to be one");
        }
        let mut ranges = Vec::with_capacity(self.ranges.len());
        for &[start, end, face] in &self.ranges {
            let Ok(face) = u32::try_from(face) else {
                return dropped("names a face past the list");
            };
            ranges.push(FacetRange { start, end, face });
        }
        let record = SourceFaces {
            names: self
                .names
                .iter()
                .map(|n| FaceName {
                    name: n.name.clone(),
                    verified: n.verified,
                    former: n.former.clone(),
                })
                .collect(),
            ranges,
        };
        if !record.tiles(len) {
            return dropped(&format!("does not tile its {len} facets"));
        }
        Some(record)
    }
}

/// Read the key without ever refusing the file: a value that is not a
/// record is dropped with a logged note (the key carries no version step,
/// so a file must open as it would without it).
pub(super) fn lenient<'de, D: Deserializer<'de>>(d: D) -> Result<Option<SourceFacesDto>, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    match serde_json::from_value::<SourceFacesDto>(value) {
        Ok(dto) => Ok(Some(dto)),
        Err(e) => {
            tracing::warn!("project: a source face record does not read ({e}); it is dropped");
            Ok(None)
        }
    }
}

/// The saved form of `entity`'s record: only a record that tiles the
/// facets it rides now.
#[must_use]
pub(super) fn collect(world: &World, entity: &Entity) -> Option<SourceFacesDto> {
    let record = crate::source_faces::of(world, entity)?;
    let len = model::facets_of(world, entity).map_or(0, |f| f.len());
    record.tiles(len).then(|| SourceFacesDto::of(&record))
}

/// Land a loaded record on model `entity`, after its facets and its
/// revision (the record is read only at the revision it lands with).
pub(super) fn land(world: &World, entity: &Entity, dto: Option<&SourceFacesDto>, owner: &str) {
    let len = model::facets_of(world, entity).map_or(0, |f| f.len());
    let record = dto.and_then(|d| d.to_record(len, owner)).map(Arc::new);
    crate::source_faces::set(world, entity, record.as_ref());
}
