//! The thread-mill golden's gate (#894): `thread_mill_m8_multi_form`
//! regenerated through the real operation - the M8 x 1.25 thread in a
//! blind 96-flat round hole of a Ø36 disc, the catalog's Ø6 multi-form
//! mill - and judged by every law its post runs: the generator's own
//! blocking warnings (the crest region, the root solid law, #1205's
//! round-wall test) and the post's verification gate. The program is
//! first proved byte-identical to its committed CL golden.
//!
//! Provenance: our own harness over our own golden (the fixture of
//! `thread_mill.rs`); no vendor material consulted.

use crate::common::gen_to_cltext;

use camcraft_app::cl::ThreadHand;
use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, SetStock};
use camcraft_app::drilling::Hole;
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::posting;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

const TOP: f64 = 20.0;
const BOTTOM: f64 = 0.0;
const FLOOR: f64 = 8.0;
/// M8 x 1.25's minor radius, as `thread_mill.rs` cuts it.
const R_M8: f64 = 3.3235;
const MULTI: &str = "TM-0600-P125";

/// `thread_mill.rs`'s disc: Ø36, z 0 to 20, the blind hole of `r` to
/// `FLOOR` in 96 flats, outward-wound.
fn disc_with_hole(r: f64) -> Mesh {
    let n = 96;
    let at = |r: f64, k: usize, z: f64| {
        let a = std::f64::consts::TAU * (k % n) as f64 / n as f64;
        Point3::new(r * a.cos(), r * a.sin(), z)
    };
    let mut f = Vec::new();
    let mut quad = |a: Point3, b: Point3, c: Point3, d: Point3| {
        f.push(Triangle::new(a, b, c));
        f.push(Triangle::new(a, c, d));
    };
    for k in 0..n {
        quad(at(18.0, k, BOTTOM), at(18.0, k + 1, BOTTOM), at(18.0, k + 1, TOP), at(18.0, k, TOP));
        quad(at(18.0, k, TOP), at(18.0, k + 1, TOP), at(r, k + 1, TOP), at(r, k, TOP));
        quad(at(r, k + 1, FLOOR), at(r, k, FLOOR), at(r, k, TOP), at(r, k + 1, TOP));
    }
    for k in 0..n {
        f.push(Triangle::new(Point3::new(0.0, 0.0, FLOOR), at(r, k, FLOOR), at(r, k + 1, FLOOR)));
        f.push(Triangle::new(Point3::new(0.0, 0.0, BOTTOM), at(18.0, k + 1, BOTTOM), at(18.0, k, BOTTOM)));
    }
    Mesh::from_facets(f)
}

/// (golden, every blocking finding) for the thread golden.
pub fn thread_verdicts() -> Vec<(&'static str, Vec<String>)> {
    let world = new_world();
    let mut history = History::default();
    let at = Location { pos: Point3::new(0.0, 0.0, BOTTOM), ..Location::default() };
    history.execute(&world, SetStock::new(&world, StockShape::Box { dx: 40.0, dy: 40.0, dz: TOP - BOTTOM }, at).expect("stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == MULTI).map(|(_, t)| t).expect("catalog tool");
    history.execute(&world, AddTool::new(&world, MULTI, tool).expect("tool"));
    history.execute(&world, ImportModel::from_mesh("disc", disc_with_hole(R_M8)));
    let params = OpParams::Thread {
        clearance_z: Height::fixed(TOP + 10.0),
        retract_z: None,
        top_z: Height::fixed(TOP),
        bottom_z: None,
        infeed: 0.25,
        feed: 300.0,
        plunge_feed: 150.0,
        climb: true,
        holes: vec![Hole { xy: Point2::new(0.0, 0.0), radius: R_M8, entry_z: TOP, floor_z: FLOOR, through: false, stepped: false }],
        pitch: None,
        hand: ThreadHand::Right,
    };
    let op = CreateOperation::new(&world, "Thread1", MULTI, SrcRef::Entity("disc".into()), params).expect("create op");
    history.execute(&world, op);
    history.execute(&world, GenerateOperation::new(&world, "Thread1").expect("generate op"));
    let e = model::find_by_name(&world, "Thread1").expect("op");
    let cache = model::toolpath_of(&world, &e).expect("cached path");
    let name = "thread_mill_m8_multi_form.cl";
    let want = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(name)).expect("golden");
    assert_eq!(gen_to_cltext(&cache.path), want.replace("\r\n", "\n"), "{name}: the regenerated program is the golden");
    let mut w: Vec<String> = cache.warnings.iter().filter(|w| w.blocks_post()).map(ToString::to_string).collect();
    w.extend(posting::verification_gate(&world));
    vec![(name, w)]
}
