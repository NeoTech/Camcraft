//! The external-thread fixtures (#985): a round base with a boss standing
//! on it, a rib beside the boss, the boss as the Thread operation lists
//! it, and a contour that machines the stock round the boss away the
//! way a shop does before thread milling it.
//!
//! Provenance: ISO 965-1 external thread sizes and public handbook
//! thread milling; our own fixtures.

use camcraft_app::cl::ThreadHand;
use camcraft_app::drilling::Hole;
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::operation::{Height, OpParams};

pub const SHOULDER: f64 = 10.0;
pub const TOP: f64 = 20.0;
pub const ENDMILL: &str = "EM-1000-4FL";

pub fn quad(a: Point3, b: Point3, c: Point3, d: Point3, f: &mut Vec<Triangle>) {
    f.push(Triangle::new(a, b, c));
    f.push(Triangle::new(a, c, d));
}

/// A Ø36 base from z 0 to the shoulder with a boss of `radius` standing
/// on it up to `TOP`, walls in 96 flats; `extra` facets ride along (a
/// rib beside the boss). Outward-wound.
pub fn stud(radius: f64, extra: Vec<Triangle>) -> Mesh {
    let n = 96;
    let at = |r: f64, k: usize, z: f64| {
        let a = std::f64::consts::TAU * (k % n) as f64 / n as f64;
        Point3::new(r * a.cos(), r * a.sin(), z)
    };
    let mut f = extra;
    for k in 0..n {
        let (o, i) = (18.0, radius);
        quad(at(o, k, 0.0), at(o, k + 1, 0.0), at(o, k + 1, SHOULDER), at(o, k, SHOULDER), &mut f);
        quad(at(o, k, SHOULDER), at(o, k + 1, SHOULDER), at(i, k + 1, SHOULDER), at(i, k, SHOULDER), &mut f);
        quad(at(i, k, SHOULDER), at(i, k + 1, SHOULDER), at(i, k + 1, TOP), at(i, k, TOP), &mut f);
        f.push(Triangle::new(Point3::new(0.0, 0.0, TOP), at(i, k, TOP), at(i, k + 1, TOP)));
        f.push(Triangle::new(Point3::new(0.0, 0.0, 0.0), at(o, k + 1, 0.0), at(o, k, 0.0)));
    }
    Mesh::from_facets(f)
}

/// A closed box, outward-wound: a rib standing on the shoulder.
pub fn rib(x0: f64, x1: f64, y: f64, z1: f64) -> Vec<Triangle> {
    camcraft_app::geometry::cube(Point3::new(x0, -y, SHOULDER), Point3::new(x1, y, z1)).facets.to_vec()
}

pub fn boss(radius: f64) -> Hole {
    Hole { xy: Point2::new(0.0, 0.0), radius, entry_z: TOP, floor_z: SHOULDER, through: false, stepped: false }
}

pub fn external(bosses: Vec<Hole>, pitch: Option<f64>, hand: ThreadHand, climb: bool) -> OpParams {
    OpParams::Thread {
        clearance_z: Height::fixed(TOP + 10.0),
        retract_z: None,
        top_z: Height::fixed(TOP),
        bottom_z: None,
        infeed: 0.3,
        feed: 300.0,
        plunge_feed: 150.0,
        climb,
        holes: bosses,
        pitch,
        hand,
        external: true,
        passes: camcraft_app::machining2d::ThreadPasses::default(),
    }
}

/// The roughing ahead: a contour round the boss with the Ø10 end mill
/// clears a 10 mm ring of stock from the top down to `bottom`, leaving
/// `leave` on the boss's wall.
pub fn clearing(bottom: f64, leave: f64) -> OpParams {
    OpParams::Contour {
        tilt: None,
        stock_to_leave: leave,
        clearance_z: Height::fixed(TOP + 10.0),
        retract_z: Height::fixed(TOP + 5.0),
        top_z: Height::fixed(TOP),
        bottom_z: Height::fixed(bottom),
        stepdown: 5.0,
        feed: 800.0,
        plunge_feed: 200.0,
        climb: true,
        side: camcraft_app::machining2d::Side::Outside,
        lead: camcraft_app::machining2d::Lead::None,
        tolerance: 0.01,
        tab_count: 0,
        tab_width: 0.0,
        tab_height: 0.0,
        entry: camcraft_app::machining3d::Entry3D::Plunge,
        helix_radius: 1.0,
        blend_turns: false,
        links: camcraft_app::links::Links::PLANAR,
    }
}
