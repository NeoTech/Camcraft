//! The source faces of an imported mesh (#3915): for a model read from a
//! Solidcraft design, SolidCraft's name of each face of the body record,
//! and the runs of facets that each record face became. Plain data, held
//! beside the mesh: nothing here reads a file, the kernel or the world, so
//! it builds for the browser too.
//! Provenance: our own design (#3915). The names are SolidCraft's, read by
//! `camcraft_kernel::solidcraft` from its published model-cache layout; the
//! record face of each facet is the machining mesh's `tri_face`. No
//! HSMWorks material consulted.

/// One record face's name as SolidCraft gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceName {
    /// The face's name, for example `F2:side:l4`.
    pub name: String,
    /// The name follows exact history; `false`: worked out from geometry.
    pub verified: bool,
    /// The name a Split Face piece had before, when it had one.
    pub former: Option<String>,
}

/// Facets `start..end` of the mesh came from record face `face`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FacetRange {
    pub start: usize,
    pub end: usize,
    pub face: u32,
}

/// The names of a body's record faces (`names[f]` is record face `f`) and
/// the facets each record face became. The ranges run in facet order; one
/// record face can have several ranges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFaces {
    pub names: Vec<FaceName>,
    pub ranges: Vec<FacetRange>,
}

impl SourceFaces {
    /// The record of `names` over a mesh whose facet `i` came from record
    /// face `tri_face[i]`: one range for each run of equal faces. `None`
    /// when a face is not below `names.len()`.
    #[must_use]
    pub fn from_tri_face(tri_face: &[u32], names: Vec<FaceName>) -> Option<Self> {
        let mut ranges: Vec<FacetRange> = Vec::new();
        for (i, &face) in tri_face.iter().enumerate() {
            if !usize::try_from(face).is_ok_and(|f| f < names.len()) {
                return None;
            }
            let next = i.checked_add(1)?;
            match ranges.last_mut() {
                Some(r) if r.face == face && r.end == i => r.end = next,
                _ => ranges.push(FacetRange {
                    start: i,
                    end: next,
                    face,
                }),
            }
        }
        Some(SourceFaces { names, ranges })
    }

    /// True when the ranges cover `0..facet_count` once, in order, each
    /// one non-empty, and each names a face below `names.len()`. A record
    /// is only kept beside a mesh it tiles.
    #[must_use]
    pub fn tiles(&self, facet_count: usize) -> bool {
        let mut at = 0_usize;
        for r in &self.ranges {
            let named = usize::try_from(r.face).is_ok_and(|f| f < self.names.len());
            if r.start != at || r.end <= r.start || r.end > facet_count || !named {
                return false;
            }
            at = r.end;
        }
        at == facet_count
    }

    /// The record face of facet `facet`, by a binary search over the
    /// ranges. Trust it only for a record that [`tiles`](Self::tiles) the
    /// facets it is held beside.
    #[must_use]
    pub fn face_of(&self, facet: usize) -> Option<u32> {
        let i = self.ranges.partition_point(|r| r.end <= facet);
        self.ranges
            .get(i)
            .filter(|r| r.start <= facet && facet < r.end)
            .map(|r| r.face)
    }

    /// The SolidCraft name of facet `facet`'s record face.
    #[must_use]
    pub fn name_of(&self, facet: usize) -> Option<&FaceName> {
        let face = usize::try_from(self.face_of(facet)?).ok()?;
        self.names.get(face)
    }
}
