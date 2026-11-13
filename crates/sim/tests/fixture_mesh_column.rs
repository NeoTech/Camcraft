//! #1530: the steel a closed fixture mesh puts on one vertical line -
//! the carried-workholding gate's per-column reading. A block reads one
//! span, a miss reads none, a nest reads its floor in the pocket and its
//! floor and wall beside it (shells sharing a face merge), and a line on
//! a facet diagonal or in a wall face is nudged off it and still read.
//! Provenance: our own geometry.
//!
//! Gate map (#1530): covers sim::fixture_mesh::column (steel_on_line).
//! Lane shape (M4.4): all tests here run in milliseconds.

use camcraft_sim::fixture_mesh::FixtureMesh;
use camcraft_sim::geometry::{Point3, Triangle, cube};

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn mesh(boxes: &[(Point3, Point3)]) -> FixtureMesh {
    let tris: Vec<Triangle> = boxes.iter().flat_map(|(a, b)| cube(*a, *b).facets.iter().copied().collect::<Vec<_>>()).collect();
    FixtureMesh::build(&tris).expect("closed")
}

/// One line read with an unbounded budget.
fn read(m: &FixtureMesh, x: f64, y: f64) -> Option<Vec<(f64, f64)>> {
    let mut budget = u64::MAX;
    m.steel_on_line(x, y, &mut budget)
}

fn near(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x.0 - y.0).abs() < 1e-9 && (x.1 - y.1).abs() < 1e-9)
}

/// An 80 x 80 slab Z -10..0 with walls Z 0..5 round a 50 x 50 pocket.
fn nest() -> FixtureMesh {
    mesh(&[
        (p(-40.0, -40.0, -10.0), p(40.0, 40.0, 0.0)),
        (p(-40.0, -40.0, 0.0), p(-25.0, 40.0, 5.0)),
        (p(25.0, -40.0, 0.0), p(40.0, 40.0, 5.0)),
        (p(-25.0, -40.0, 0.0), p(25.0, -25.0, 5.0)),
        (p(-25.0, 25.0, 0.0), p(25.0, 40.0, 5.0)),
    ])
}

#[test]
fn a_block_reads_its_span_and_a_miss_reads_none() {
    let m = mesh(&[(p(0.0, 0.0, 2.0), p(10.0, 10.0, 7.0))]);
    let s = read(&m, 3.3, 6.1).expect("read");
    assert!(near(&s, &[(2.0, 7.0)]), "{s:?}");
    assert_eq!(read(&m, 12.0, 6.1), Some(vec![]));
}

#[test]
fn a_line_on_a_facet_diagonal_is_nudged_off_it_and_read() {
    // A centred square's top and bottom faces are split on x == y.
    let m = mesh(&[(p(-40.0, -40.0, -10.0), p(40.0, 40.0, 0.0))]);
    for q in [0.0, 12.5, -33.75] {
        let s = read(&m, q, q).expect("nudged and read");
        assert!(near(&s, &[(-10.0, 0.0)]), "{q}: {s:?}");
    }
}

#[test]
fn a_nest_reads_its_floor_in_the_pocket_and_floor_and_wall_beside_it() {
    let m = nest();
    let s = read(&m, 1.1, -2.3).expect("pocket");
    assert!(near(&s, &[(-10.0, 0.0)]), "{s:?}");
    let s = read(&m, 30.2, 1.7).expect("wall");
    assert!(near(&s, &[(-10.0, 5.0)]), "shells sharing a face read as one: {s:?}");
}

#[test]
fn a_line_in_a_wall_face_is_still_read() {
    let m = nest();
    let s = read(&m, 25.0, 3.1).expect("nudged off the face");
    assert!(near(&s, &[(-10.0, 0.0)]) || near(&s, &[(-10.0, 5.0)]), "{s:?}");
}

#[test]
fn a_spent_budget_reads_nothing_and_never_guesses() {
    let m = nest();
    let mut budget = 3u64;
    assert_eq!(m.steel_on_line(30.2, 1.7, &mut budget), None, "three facets cannot read a wall");
    assert_eq!(budget, 0);
    let mut budget = 10_000u64;
    assert!(m.steel_on_line(30.2, 1.7, &mut budget).is_some());
    assert!(budget < 10_000, "each facet visited is spent");
}

#[test]
fn a_non_finite_line_is_not_read() {
    assert_eq!(read(&nest(), f64::NAN, 0.0), None);
}
