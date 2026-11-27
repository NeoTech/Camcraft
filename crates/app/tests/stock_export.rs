//! M7.2 D (app half): what the program leaves, as a mesh. The traced
//! stock's field extracts to a closed, outward shell whose volume is
//! the stock minus what the Face program removed; it round-trips
//! through our own STL importer facet for facet; and the mesh cache
//! extracts once per field view, never serving a stale one.
//! Provenance: our own fixtures over the M7.1 field engine; nothing
//! from vendor binaries.
//!
//! Gate map (M4.4): covers app::stock_export, sim::stock_sim::iso, app::geometry::import::stl.

use std::sync::Arc;

use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock};
use camcraft_app::geometry::import::stl;
use camcraft_app::geometry::{cube, Point3, Triangle, IDENTITY_Q};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::stock_sim::iso::unpaired_edges;
use camcraft_app::stock_export::{file_bytes, machined_mesh, stl_bytes, write_stl, MeshCache};
use camcraft_app::stock_view::field_view_with;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

const HALF: f64 = 6.0;
const COARSE: f64 = (2.0 * HALF) * (2.0 * HALF) * 20.0 / 3.6;

/// A 12x12x20 box faced to z 18 over a 12x12x18 model.
fn faced_world() -> flecs_ecs::prelude::World {
    let world = new_world();
    let mut history = History::default();
    let loc = Location { pos: Point3::ZERO, rot: IDENTITY_Q };
    let cmd = SetStock::new(&world, StockShape::Box { dx: 2.0 * HALF, dy: 2.0 * HALF, dz: 20.0 }, loc);
    history.execute(&world, cmd.expect("box stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == "EM-0400-2FL").map(|(_, t)| t).expect("catalog entry");
    history.execute(&world, AddTool::new(&world, "EM-0400-2FL", tool).expect("catalog tool"));
    history.execute(&world, ImportModel::from_mesh("part", cube(Point3::new(-HALF, -HALF, 0.0), Point3::new(HALF, HALF, 18.0))));
    let params = OpParams::Face { holes: Default::default(),
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(18.0),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
        blend_turns: false, blend_radius: None,
        picks: Default::default(),
    };
    history.execute(&world, CreateOperation::new(&world, "Face1", "EM-0400-2FL", SrcRef::Stock, params).expect("create face"));
    history.execute(&world, GenerateOperation::new(&world, "Face1").expect("generate face"));
    world
}

fn volume(tris: &[Triangle]) -> f64 {
    tris.iter().map(|t| t.a.dot(t.b.cross(t.c)) / 6.0).sum()
}

#[test]
fn the_machined_stock_extracts_closed_with_the_faced_volume() {
    let world = faced_world();
    let fv = field_view_with(&world, COARSE).expect("field view");
    let tris = machined_mesh(&fv).expect("a closed shell");
    let v = volume(&tris);
    let (uncut, faced) = (12.0 * 12.0 * 20.0, 12.0 * 12.0 * 18.0);
    // Outward and closed (machined_mesh refuses open edges); the 2 mm
    // skim is gone within the cell's own chamfer of the box edges.
    assert!(v > 0.0, "outward-facing: {v}");
    assert!(v < uncut - 0.5 * 12.0 * 12.0 * 2.0, "the facing removed material: {v} vs uncut {uncut}");
    assert!((v - faced).abs() < 0.08 * faced, "volume {v} vs the faced {faced}");
}

#[test]
fn the_stl_round_trips_through_our_importer_and_the_cache_keys_on_the_view() {
    let world = faced_world();
    let fv = Arc::new(field_view_with(&world, COARSE).expect("field view"));
    let dir = std::env::temp_dir().join(format!("camcraft-stock-export-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("machined.stl");
    let n = write_stl(&fv, &path).expect("written");
    let back = stl::parse_file(&path).expect("our importer reads it");
    assert_eq!(back.facets.len(), n, "facet for facet");
    let _ = std::fs::remove_dir_all(&dir);

    let mut cache = MeshCache::default();
    let a = cache.get(&fv).expect("mesh");
    let b = cache.get(&fv).expect("mesh");
    assert!(Arc::ptr_eq(&a, &b), "the same view is not re-extracted");
    let fv2 = Arc::new(field_view_with(&world, COARSE).expect("field view"));
    let c = cache.get(&fv2).expect("mesh");
    assert!(!Arc::ptr_eq(&a, &c), "a new view is a new mesh, never the stale one");
    cache.clear();
}

/// #4182: corners read back from binary STL bytes.
fn read_back(b: &[u8]) -> Vec<Triangle> {
    b[84..]
        .chunks(50)
        .map(|r| {
            let f = |o: usize| f64::from(f32::from_le_bytes([r[o], r[o + 1], r[o + 2], r[o + 3]]));
            let c = |j: usize| Point3::new(f(12 + 12 * j), f(16 + 12 * j), f(20 + 12 * j));
            Triangle::new(c(0), c(1), c(2))
        })
        .collect()
}

#[test]
fn the_exported_file_is_closed_as_written_and_a_clean_mesh_writes_unchanged() {
    // #4182: a closed tetrahedron near 150 mm with edge 0-1 split 3 nm from
    // corner 0, under one f32 step (15.3 nm). Checked in f64 it passes;
    // written as it is, the file has two facets with a repeated corner and
    // edges not on two facets.
    let p = [
        Point3::new(150.0, 150.0, 150.0),
        Point3::new(160.0, 150.0, 150.0),
        Point3::new(150.0, 160.0, 150.0),
        Point3::new(150.0, 150.0, 160.0),
    ];
    let a = Point3::new(150.0 + 3e-6, 150.0, 150.0);
    let split = vec![
        Triangle::new(p[0], p[2], a),
        Triangle::new(a, p[2], p[1]),
        Triangle::new(p[0], a, p[3]),
        Triangle::new(a, p[1], p[3]),
        Triangle::new(p[1], p[2], p[3]),
        Triangle::new(p[0], p[3], p[2]),
    ];
    assert_eq!(unpaired_edges(&split), 0);
    assert!(unpaired_edges(&read_back(&stl_bytes(&split))) > 0, "the old write");
    let (bytes, n) = file_bytes(&split).expect("closed as written");
    assert_eq!(n, 4);
    assert_eq!(unpaired_edges(&read_back(&bytes)), 0);
    let clean = cube(Point3::new(150.0, 150.0, 150.0), Point3::new(160.0, 160.0, 160.0));
    assert_eq!(file_bytes(&clean.facets).expect("clean").0, stl_bytes(&clean.facets));
}

#[test]
fn two_solids_whose_edges_meet_in_the_file_are_refused() {
    // #4182 review: two closed boxes near 150 mm whose vertical edges stand
    // 3 nm apart in x and y, under one f32 step (15.3 nm). No facet
    // collapses and the f64 check passes, but as written the two edges are
    // one edge on four facets, so the file is refused, not written.
    let a = cube(Point3::new(148.0, 148.0, 0.0), Point3::new(150.0, 150.0, 2.0));
    let b = cube(Point3::new(150.0 + 3e-6, 150.0 + 3e-6, 0.0), Point3::new(152.0, 152.0, 2.0));
    let both: Vec<Triangle> = a.facets.iter().chain(b.facets.iter()).copied().collect();
    assert_eq!(unpaired_edges(&both), 0);
    assert!(unpaired_edges(&read_back(&stl_bytes(&both))) > 0, "the old write");
    let refused = file_bytes(&both).expect_err("one edge on four facets as written");
    assert!(refused.contains("not on two facets"), "{refused}");
}

#[test]
fn a_sliver_that_collapses_to_nothing_as_written_is_refused() {
    // #4182: the closed thin tetrahedron of the issue, its last two corners
    // 3 nm apart near 150 mm. As written two facets collapse and the other
    // two are one facet both ways, so nothing is left to export.
    let (p0, p1) = (Point3::new(150.0, 150.0, 150.0), Point3::new(160.0, 150.0, 150.0));
    let (x, y) = (Point3::new(155.0, 155.0, 150.0), Point3::new(155.0, 155.0, 150.0 + 3e-6));
    let thin = vec![Triangle::new(p0, p1, x), Triangle::new(p0, y, p1), Triangle::new(p0, x, y), Triangle::new(p1, y, x)];
    assert_eq!(unpaired_edges(&thin), 0);
    let refused = file_bytes(&thin).expect_err("nothing left as written");
    assert!(refused.contains("collapses to nothing"), "{refused}");
}
