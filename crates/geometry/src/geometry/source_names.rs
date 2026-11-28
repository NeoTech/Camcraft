//! The SolidCraft names under a CamCraft feature (#3915): the record faces
//! of a `.sc` import's source face record (`crate::source_faces`) that one
//! planar face, or one welded patch, is made of, read as SolidCraft's own
//! face names.
//!
//! Law: a record that does not describe the facets gives no names, never a
//! wrong one. Its ranges must tile every facet that the features were built
//! from, and every index it gives must be in range. The caller builds the
//! features from the very facet array that the record indexes
//! (`features_of_facets` on the model's own facets): a features cache keyed
//! by content can answer with a mesh of the same content in another facet
//! order, and then every facet index points at another facet.
//!
//! Provenance: our own design over this repo's `geometry::feature` faces and
//! patches and the source face record that the `.sc` import keeps (read
//! from SolidCraft's published model cache format); no vendor material
//! consulted.

use std::collections::{BTreeSet, HashMap};

use super::feature::{FaceId, Features, Patch};
use crate::source_faces::SourceFaces;

/// The SolidCraft names of one feature, sorted and deduplicated. One name
/// for a feature on one record face; more for a feature that spans record
/// faces that CamCraft welds into one (coplanar neighbours, or a flat side,
/// a fillet and a flat side that meet tangentially). `unverified` is true
/// when SolidCraft has not verified one of the names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameSet<'a> {
    pub names: Vec<&'a str>,
    pub unverified: bool,
}

/// The names of the faces and patches of one mesh, from its record.
pub struct SourceNames<'a> {
    record: &'a SourceFaces,
    features: &'a Features,
    /// Face id -> indexes into `features.faces` (one face in a sound mesh).
    by_id: HashMap<FaceId, Vec<usize>>,
}

impl<'a> SourceNames<'a> {
    /// `None` when the record does not describe the facets that `features`
    /// were built from: its ranges do not tile them, or it has no names.
    #[must_use]
    pub fn new(record: &'a SourceFaces, features: &'a Features) -> Option<Self> {
        let facets = features.facet_face.len();
        if facets == 0 || record.names.is_empty() || !record.tiles(facets) {
            return None;
        }
        let mut by_id: HashMap<FaceId, Vec<usize>> = HashMap::new();
        for (i, f) in features.faces.iter().enumerate() {
            by_id.entry(f.id).or_default().push(i);
        }
        Some(Self { record, features, by_id })
    }

    /// The names of the face with `id`; `None` when no face has that id.
    #[must_use]
    pub fn face(&self, id: FaceId) -> Option<NameSet<'a>> {
        self.names_of(self.record_faces(&[id])?)
    }

    /// The names of `patch`: the names of all its member faces. `None` when
    /// one of its faces is not in these features.
    #[must_use]
    pub fn patch(&self, patch: &Patch) -> Option<NameSet<'a>> {
        self.names_of(self.record_faces(&patch.faces)?)
    }

    /// The record faces under the faces with these ids. `None` when an id
    /// names no face, or a facet has no record face.
    fn record_faces(&self, ids: &[FaceId]) -> Option<BTreeSet<u32>> {
        let mut out = BTreeSet::new();
        for id in ids {
            for &i in self.by_id.get(id)? {
                for &facet in &self.features.faces.get(i)?.facets {
                    out.insert(self.record.face_of(usize::try_from(facet).ok()?)?);
                }
            }
        }
        (!out.is_empty()).then_some(out)
    }

    /// The names of record faces. `None` when an index passes the name list
    /// or a name is empty: no blank and no partial name sets.
    fn names_of(&self, faces: BTreeSet<u32>) -> Option<NameSet<'a>> {
        let record: &'a SourceFaces = self.record;
        let mut names = Vec::with_capacity(faces.len());
        let mut unverified = false;
        for face in faces {
            let n = record.names.get(usize::try_from(face).ok()?)?;
            if n.name.is_empty() {
                return None;
            }
            unverified |= !n.verified;
            names.push(n.name.as_str());
        }
        names.sort_unstable();
        names.dedup();
        Some(NameSet { names, unverified })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::feature::{FaceKind, features_of_facets, patches};
    use crate::geometry::{Point3, Triangle, cylinder};
    use crate::source_faces::{FaceName, FacetRange};

    /// The sides of the 10 mm box, in the order `box_facets` writes them.
    const SIDES: [&str; 6] = ["bottom", "top", "y-", "y+", "x-", "x+"];

    /// A 10 mm box from the origin: six quads of two facets each, in the
    /// order of `SIDES`.
    fn box_facets() -> Vec<Triangle> {
        let v = |i: usize| {
            let c = |bit: usize| if i & bit == 0 { 0.0 } else { 10.0 };
            Point3::new(c(1), c(2), c(4))
        };
        let quads = [[0, 2, 3, 1], [4, 5, 7, 6], [0, 1, 5, 4], [2, 6, 7, 3], [0, 4, 6, 2], [1, 3, 7, 5]];
        quads
            .iter()
            .flat_map(|q| [Triangle::new(v(q[0]), v(q[1]), v(q[2])), Triangle::new(v(q[0]), v(q[2]), v(q[3]))])
            .collect()
    }

    fn names(list: &[&str]) -> Vec<FaceName> {
        list.iter().map(|n| FaceName { name: (*n).to_owned(), verified: true, former: None }).collect()
    }

    /// A record from the record face of each facet (PR 1's builder).
    fn record(per_facet: &[u32], list: Vec<FaceName>) -> SourceFaces {
        SourceFaces::from_tri_face(per_facet, list).expect("every face index is below the name count")
    }

    fn one_per_quad() -> Vec<u32> {
        (0..12).map(|i| i / 2).collect()
    }

    /// The box side that a face lies on, read from its plane alone.
    fn side_of(normal: [f64; 3], offset: f64) -> &'static str {
        let slot = match normal.iter().position(|c| c.abs() > 0.5) {
            Some(2) => 0,
            Some(1) => 2,
            Some(0) => 4,
            _ => return "?",
        };
        SIDES[slot + usize::from(offset.abs() > 5.0)]
    }

    #[test]
    fn each_box_side_reads_its_own_name() {
        let tris = box_facets();
        let f = features_of_facets(&tris);
        let rec = record(&one_per_quad(), names(&SIDES));
        let sn = SourceNames::new(&rec, &f).expect("the record tiles the 12 facets");
        let planar: Vec<_> = f.faces.iter().filter(|fa| fa.kind == FaceKind::Planar).collect();
        assert_eq!(planar.len(), 6);
        for fa in planar {
            let want = side_of(fa.normal, fa.offset);
            assert_eq!(sn.face(fa.id), Some(NameSet { names: vec![want], unverified: false }), "{want}");
        }
    }

    #[test]
    fn a_face_over_two_record_faces_has_both_names_sorted() {
        let tris = box_facets();
        let f = features_of_facets(&tris);
        // The top's two facets (2 and 3) are two SolidCraft faces; CamCraft
        // welds them into one planar face.
        let mut per_facet = one_per_quad();
        per_facet[3] = 6;
        let mut list = names(&SIDES);
        list[1].name = "top:b".to_owned();
        list.extend(names(&["top:a"]));
        let rec = record(&per_facet, list);
        let sn = SourceNames::new(&rec, &f).expect("tiles");
        let top = f.faces.iter().find(|fa| side_of(fa.normal, fa.offset) == "top").expect("a top face");
        assert_eq!(sn.face(top.id).map(|s| s.names), Some(vec!["top:a", "top:b"]));
    }

    #[test]
    fn an_unverified_name_is_flagged() {
        let tris = box_facets();
        let f = features_of_facets(&tris);
        let mut list = names(&SIDES);
        list[0].verified = false;
        let rec = record(&one_per_quad(), list);
        let sn = SourceNames::new(&rec, &f).expect("tiles");
        for fa in &f.faces {
            let side = side_of(fa.normal, fa.offset);
            assert_eq!(sn.face(fa.id).expect("a name").unverified, side == "bottom", "{side}");
        }
    }

    #[test]
    fn a_record_that_does_not_fit_gives_no_names() {
        let tris = box_facets();
        let f = features_of_facets(&tris);
        let fits = |rec: SourceFaces| SourceNames::new(&rec, &f).is_some();
        assert!(fits(record(&one_per_quad(), names(&SIDES))), "the control fits");
        let short: Vec<u32> = (0..10).map(|i| i / 2).collect();
        assert!(!fits(record(&short, names(&SIDES))), "two facets with no record face");
        let long: Vec<u32> = (0..14).map(|i| (i / 2).min(5)).collect();
        assert!(!fits(record(&long, names(&SIDES))), "ranges past the facets");
        let past = SourceFaces { names: names(&SIDES), ranges: vec![FacetRange { start: 0, end: 12, face: 9 }] };
        assert!(!fits(past), "a record face past the name list");
        assert!(!fits(SourceFaces { names: Vec::new(), ranges: Vec::new() }), "an empty record");
    }

    #[test]
    fn an_id_that_is_not_in_the_mesh_has_no_name() {
        let tris = box_facets();
        let f = features_of_facets(&tris);
        let rec = record(&one_per_quad(), names(&SIDES));
        let sn = SourceNames::new(&rec, &f).expect("tiles");
        let ids: Vec<FaceId> = f.faces.iter().map(|fa| fa.id).collect();
        let ghost = (1..).map(|k: u64| k.wrapping_mul(0x9e37_79b9_7f4a_7c15)).find(|id| !ids.contains(id));
        assert_eq!(sn.face(ghost.expect("an unused id")), None);
    }

    #[test]
    fn a_blank_name_gives_no_name() {
        let tris = box_facets();
        let f = features_of_facets(&tris);
        let mut list = names(&SIDES);
        list[1].name.clear();
        let rec = record(&one_per_quad(), list);
        let sn = SourceNames::new(&rec, &f).expect("tiles");
        for fa in &f.faces {
            let side = side_of(fa.normal, fa.offset);
            assert_eq!(sn.face(fa.id).is_none(), side == "top", "{side}");
        }
    }

    #[test]
    fn a_patch_reads_the_names_of_all_its_strips() {
        // `cylinder` writes four facets for each of its 16 segments: two
        // wall facets, a top cap facet, a bottom cap facet.
        let m = cylinder(20.0, 10.0, 16);
        let f = features_of_facets(&m.facets);
        let ps = patches(&f);
        assert_eq!(ps.len(), 1, "the wall is one patch");
        let per_facet: Vec<u32> = (0..m.facets.len())
            .map(|i| match i % 4 {
                0 | 1 if i / 4 < 8 => 0,
                0 | 1 => 1,
                2 => 2,
                _ => 3,
            })
            .collect();
        let rec = record(&per_facet, names(&["wall:a", "wall:b", "top", "bottom"]));
        let sn = SourceNames::new(&rec, &f).expect("tiles");
        assert_eq!(sn.patch(&ps[0]).map(|s| s.names), Some(vec!["wall:a", "wall:b"]));
        // Each strip is one face, inside one segment: one name.
        for id in &ps[0].faces {
            let one = sn.face(*id).expect("a strip has a name").names;
            assert!(one == vec!["wall:a"] || one == vec!["wall:b"], "{one:?}");
        }
    }
}
