//! The source faces beside an imported mesh (#3915): the facet ranges are
//! the runs of the record face of each facet, a record is only good for
//! the facets it tiles, and a Solidcraft import hands out the record of the
//! very facets it made, in their order. That the names sit on the right
//! faces is pinned in the kernel (`solidcraft_import/names.rs`, by plane);
//! here, that the geometry layer keeps facet `i` on record face
//! `tri_face[i]` of the kernel's mesh.
//! Provenance: our own tests; the Solidcraft file is SolidCraft 0.46.2's
//! testdata/models/hole_on_pocket_wall.sc.

use camcraft_geometry::source_faces::{FaceName, FacetRange, SourceFaces};

fn named(names: &[&str]) -> Vec<FaceName> {
    names
        .iter()
        .map(|n| FaceName {
            name: (*n).to_owned(),
            verified: true,
            former: None,
        })
        .collect()
}

#[test]
fn ranges_follow_the_runs_of_the_record_face_of_each_facet() {
    // Faces out of order, and face 0 in two runs.
    let tri_face = [0, 0, 1, 1, 1, 0, 2, 2];
    let record = match SourceFaces::from_tri_face(&tri_face, named(&["a", "b", "c"])) {
        Some(r) => r,
        None => panic!("every face is named"),
    };
    let want = [(0, 2, 0), (2, 5, 1), (5, 6, 0), (6, 8, 2)].map(|(start, end, face)| FacetRange {
        start,
        end,
        face,
    });
    assert_eq!(record.ranges, want);
    assert!(record.tiles(8));
    assert!(!record.tiles(7), "a facet count short of the ranges");
    assert!(!record.tiles(9), "a facet count past the ranges");
    for (i, f) in tri_face.iter().enumerate() {
        assert_eq!(record.face_of(i), Some(*f), "facet {i}");
    }
    assert_eq!(record.face_of(8), None);
    assert_eq!(record.name_of(5).map(|n| n.name.as_str()), Some("a"));
    assert_eq!(record.name_of(7).map(|n| n.name.as_str()), Some("c"));
}

#[test]
fn a_face_past_the_names_gives_no_record() {
    assert_eq!(
        SourceFaces::from_tri_face(&[0, 2], named(&["a", "b"])),
        None
    );
    // No facets: no ranges, and it tiles nothing but an empty mesh.
    let empty = SourceFaces::from_tri_face(&[], named(&["a"]));
    assert!(
        empty
            .as_ref()
            .is_some_and(|r| r.ranges.is_empty() && r.tiles(0) && !r.tiles(1))
    );
}

#[test]
fn a_record_that_does_not_tile_is_told_apart() {
    let r = |ranges: &[(usize, usize, u32)]| SourceFaces {
        names: named(&["a", "b"]),
        ranges: ranges
            .iter()
            .map(|&(start, end, face)| FacetRange { start, end, face })
            .collect(),
    };
    assert!(r(&[(0, 2, 0), (2, 4, 1)]).tiles(4));
    for (case, ranges) in [
        ("a gap", vec![(0, 2, 0), (3, 4, 1)]),
        ("an overlap", vec![(0, 3, 0), (2, 4, 1)]),
        ("an empty range", vec![(0, 2, 0), (2, 2, 1), (2, 4, 0)]),
        ("a range past the facets", vec![(0, 2, 0), (2, 5, 1)]),
        ("a face with no name", vec![(0, 2, 0), (2, 4, 2)]),
        ("out of order", vec![(2, 4, 1), (0, 2, 0)]),
        ("no ranges", vec![]),
    ] {
        assert!(!r(&ranges).tiles(4), "{case}");
    }
}

/// The record of a Solidcraft import is the record of its own facets: for
/// each facet, the record face that the kernel's machining mesh gives for
/// the triangle with the same corners.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_solidcraft_import_hands_out_the_record_of_its_own_facets() {
    use camcraft_kernel::CancelToken;
    use camcraft_kernel::machining_mesh::machining_mesh;
    use camcraft_kernel::solidcore_impl::SolidcoreKernel;
    use camcraft_kernel::solidcraft::import_solidcraft;

    const CHORD: f64 = 0.005;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata/models/hole_on_pocket_wall.sc");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let (mesh, notes, record) =
        match camcraft_geometry::solidcraft::parse(&bytes, usize::MAX, CHORD) {
            Ok(parsed) => parsed,
            Err(e) => panic!("the design is refused: {e}"),
        };
    let record = record.unwrap_or_else(|| panic!("a format-10 design has names: {notes:?}"));
    assert_eq!(record.names.len(), 13);
    assert!(record.tiles(mesh.facets.len()));
    assert!(!notes.iter().any(|n| n.contains("face names")), "{notes:?}");
    // The kernel's own mesh of the same body, at the same chord.
    let design =
        match import_solidcraft(&mut SolidcoreKernel::default(), &bytes, &CancelToken::new()) {
            Ok(d) => d,
            Err(e) => panic!("the kernel refuses the design: {e}"),
        };
    let kernel = match machining_mesh(&design.solid, CHORD, &CancelToken::new()) {
        Ok(m) => m,
        Err(e) => panic!("the kernel does not mesh the design: {e}"),
    };
    assert_eq!(kernel.triangles.len(), mesh.facets.len());
    for (i, (t, f)) in kernel.triangles.iter().zip(&kernel.tri_face).enumerate() {
        let corner = |v: u32| kernel.positions[v as usize];
        let facet = mesh.facets[i];
        let got = [facet.a, facet.b, facet.c].map(|p| [p.x, p.y, p.z]);
        assert_eq!(
            got,
            t.map(corner),
            "facet {i} is not the kernel's triangle {i}"
        );
        assert_eq!(record.face_of(i), Some(*f), "facet {i}");
    }
}
