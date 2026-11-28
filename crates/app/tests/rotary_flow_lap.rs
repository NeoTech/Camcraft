//! #2829: the Rotary Flow single-chain and set-refusal laws end to
//! end, on the turned bar. ONE meridian profile chain machines the
//! full revolution - the kernel morphs it to its own next lap, and
//! the pass family sweeps every azimuth of the turned surface - and
//! a pick set whose chains meet (the meridian crosses the station-2
//! ring at their shared corner) is refused by name, with no path
//! cached.
//!
//! Provenance: our own fixture over our own machine files and
//! commands; nothing of vendor binaries.
//!
//! Gate map (M4.4): covers app::operation (the RotaryFlow arm),
//! camcraft-motion (rotary_flow, rotary_flow_set), app::fixture.
//! File note (#3187): the rotary domain keeps one top-level file
//! per topic; a second file holds these laws so each stays under
//! the 400-line ceiling.

mod common;

use flecs_ecs::prelude::World;

use camcraft_app::cl::MoveKind;
use camcraft_app::commands::{
    AddTool, CreateOperation, GenerateOperation, History, ImportModel, LoadMachine, SetStock,
};
use camcraft_app::geometry::{feature, Mesh, Point3, Triangle};
use camcraft_app::model;
use camcraft_app::operation::{FeatureSel, GeoSel, Height, OpParams, OpStatus, SrcRef};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::operation;

const TAU: f64 = std::f64::consts::TAU;
/// The tormach-770mx-microarc4.mch registry entry: an A about X,
/// stated `[1, 0, 0]`, work-turning.
const MICROARC: &str = "Tormach 770MX + microARC 4";
/// The turned bar's meridian (rho, s): wall r6 with an interior
/// cylindrical band (the rings at -8 and 2), shoulder, neck r5.
const MERIDIAN: [(f64, f64); 7] = [(0.0, -16.0), (6.0, -16.0), (6.0, -8.0), (6.0, 2.0), (5.0, 8.0), (5.0, 16.0), (0.0, 16.0)];

/// The bar laid on world X (the microARC's stated A line), 72
/// facets around, centred on the origin.
fn bar() -> Vec<Triangle> {
    let pt = |rho: f64, s: f64, j: usize| {
        let th = TAU * j as f64 / 72.0;
        Point3::new(s, rho * th.cos(), rho * th.sin())
    };
    let mut f = Vec::new();
    for w in MERIDIAN.windows(2) {
        let ((ra, za), (rb, zb)) = (w[0], w[1]);
        for j in 0..72 {
            let k = (j + 1) % 72;
            let (a1, a2, b1, b2) = (pt(ra, za, j), pt(ra, za, k), pt(rb, zb, j), pt(rb, zb, k));
            if ra <= 1e-12 {
                f.push(Triangle::new(a1, b2, b1));
            } else if rb <= 1e-12 {
                f.push(Triangle::new(a1, a2, b2));
            } else {
                f.push(Triangle::new(a1, a2, b2));
                f.push(Triangle::new(a1, b2, b1));
            }
        }
    }
    f
}

fn flow() -> OpParams {
    OpParams::RotaryFlow {
        tolerance: 0.1,
        stepover: Some(2.0),
        clearance_z: Height::fixed(25.0),
        retract_z: None,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        lead_deg: 0.0,
        lag_deg: 0.0,
        max_tilt_deg: 60.0,
        stated_axis: None,
    }
}

/// A world with the ball, the microARC, a snug round stock and the
/// bar; the op is added by the caller, because its picks read the
/// imported model's own chains.
fn bar_world() -> World {
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    let tool = catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", tool).expect("tool"));
    h.execute(&world, LoadMachine::builtin(&world, MICROARC).expect("machine"));
    let loc = Location {
        pos: Point3::new(-16.0, 0.0, 0.0),
        rot: camcraft_app::geometry::quat_from_axis_angle(1, std::f64::consts::FRAC_PI_2),
    };
    h.execute(
        &world,
        SetStock::new(&world, StockShape::Cylinder { diameter: 12.0, height: 32.0 }, loc).expect("stock"),
    );
    h.execute(&world, ImportModel::from_mesh("bar", Mesh::from_facets(bar())));
    world
}

/// The bar's meridian profile chain: the short chain that runs the
/// whole axial length at one azimuth (its first and last points
/// stand at the two end faces).
fn meridian_chain(world: &World) -> GeoSel {
    let e = model::find_by_name(world, "bar").expect("bar");
    let f = model::features_of(world, &e).expect("features");
    for c in feature::chains(&f) {
        let pts = feature::chain_points(&f, &c);
        if pts.len() < 3 {
            continue;
        }
        let spans = pts.first().is_some_and(|p| (p[0] + 16.0).abs() < 1e-6)
            && pts.last().is_some_and(|p| (p[0] - 16.0).abs() < 1e-6);
        if spans {
            return GeoSel {
                entity: "bar".into(),
                feature: FeatureSel::Chain(c.id),
            };
        }
    }
    panic!("the bar's meridian profile chain was not found");
}

/// The bar's ring chain at axial station 2 (the shoulder ring).
fn ring_chain(world: &World, station: f64) -> GeoSel {
    let e = model::find_by_name(world, "bar").expect("bar");
    let f = model::features_of(world, &e).expect("features");
    for c in feature::chains(&f) {
        let pts = feature::chain_points(&f, &c);
        let at = pts.len() >= 10 && pts.iter().all(|p| (p[0] - station).abs() < 1e-6);
        if at {
            return GeoSel {
                entity: "bar".into(),
                feature: FeatureSel::Chain(c.id),
            };
        }
    }
    panic!("the bar's ring chain at station {station} was not found");
}

fn add_op(world: &World, picks: SrcRef, params: OpParams) {
    let cmd = CreateOperation::new(world, "OP", "BN-0600", picks, params).expect("create");
    History::default().execute(world, cmd);
}

/// The generation refusal's own words, when the command refuses.
fn generate_err_text(world: &World, name: &str) -> String {
    match GenerateOperation::new(world, name) {
        Err(e) => e.to_string(),
        Ok(cmd) => {
            History::default().execute(world, cmd);
            String::from("generated")
        }
    }
}

fn is_ok(world: &World) -> bool {
    let e = model::find_by_name(world, "OP").expect("op");
    OpStatus::Ok == operation::status_of(world, &e)
}

#[test]
fn one_meridian_chain_machines_the_full_revolution() {
    // The single-chain case (#2829): ONE meridian profile chain,
    // picked alone. The kernel morphs it to its own next lap, so
    // the developed band is one full revolution at the meridian's
    // own outermost radius (6 mm): ceil(6.0 * TAU / 2.0) = 19 spans,
    // one pass each - the meridian itself at the opening edge, the
    // closing pass dropped because the lap's two edges are one
    // world curve. The passes sweep every azimuth of the turned
    // surface: each lands at its own azimuth about the stated A.
    let world = bar_world();
    add_op(&world, SrcRef::Selection(vec![meridian_chain(&world)]), flow());
    let cmd = GenerateOperation::new(&world, "OP").expect("generate");
    History::default().execute(&world, cmd);
    assert!(is_ok(&world), "a single meridian edge machines the full revolution");
    let e = model::find_by_name(&world, "OP").expect("op");
    let path = model::toolpath_of(&world, &e).expect("a cached path").path;
    let ls: Vec<Point3> = path
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Plunge)
        .filter_map(|m| {
            let a = m.axis?;
            Some(Point3::new(
                m.to.x + a[0] * 3.0,
                m.to.y + a[1] * 3.0,
                m.to.z + a[2] * 3.0,
            ))
        })
        .collect();
    assert_eq!(ls.len(), 19, "the full revolution did not divide into 19 passes");
    // One landing per azimuth: the pass family covers the whole
    // revolution, and no two passes land on one station.
    let azs: Vec<f64> = ls.iter().map(|c| c.z.atan2(c.y).rem_euclid(TAU)).collect();
    for (i, &x) in azs.iter().enumerate() {
        for &y in azs.iter().skip(i + 1) {
            let d = (x - y).abs().min(TAU - (x - y).abs());
            assert!(d > 1e-6, "two passes land at one azimuth: {x:.3} ~ {y:.3}");
        }
    }
    assert!(
        path.notes.iter().any(|n| n.contains("morphed a full revolution")),
        "the notes do not state the lap morph: {:?}",
        path.notes
    );
}

#[test]
fn a_pick_set_whose_chains_meet_is_refused_by_name() {
    // The meridian chain runs through the shoulder ring's own
    // corner at station 2: the set law refuses the pair at the
    // shared corner, and no path is cached.
    let world = bar_world();
    add_op(
        &world,
        SrcRef::Selection(vec![meridian_chain(&world), ring_chain(&world, 2.0)]),
        flow(),
    );
    let err = generate_err_text(&world, "OP");
    assert!(err.contains("meet or cross each other"), "{err}");
    let e = model::find_by_name(&world, "OP").expect("op");
    assert!(model::toolpath_of(&world, &e).is_none(), "a refused set leaves no path");
}
