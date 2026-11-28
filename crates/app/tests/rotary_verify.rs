//! The rotary verdict (M13): the three rotary kinds over the setup a
//! user builds for them - the turned bar, a coaxial round bar stock a
//! hair over the part's OD, a 6 mm ball - verify SILENT and post on
//! the tilting head; the same programs over steel they really drive
//! through (an oversized bar, a clearance plane inside the stock) still
//! convict and block, while a rough over an oversized bar peels from
//! that bar's radius and stays silent (M13.8). The link laws (M13.8):
//! every program opens on its clearance pose, and Rotary Parallel's
//! links are radial. And the pose law the verdicts stand on: the ball the stream
//! puts at the tip-plus-r-along-the-axis is the ball the ladder
//! planned, tangent to the skin - not one radius off it. The radial door
//! (#2774): a clearance, retract or band radius the bar would drive the
//! tool through is refused by name when the command is made.
//! Provenance: our own fixtures (the operations suite's turned bar);
//! the law is the Prime Directive's "correct, or visibly refused".
//!
//! Gate map (M4.4): covers app::operation, app::posting, sim::verify, motion::multiaxis.

use flecs_ecs::prelude::World;

use camcraft_app::cl::MoveKind;
use camcraft_app::commands::{
    AddTool, CreateOperation, EditOperationParams, GenerateOperation, History, ImportModel,
    LoadMachine, SetStock, SetWcsOrigin,
};
use camcraft_app::geometry::{quat_from_axis_angle, Mesh, Point3, Triangle, IDENTITY_Q};
use camcraft_app::model;
use camcraft_app::operation::{self, Height, OpParams, OpStatus, RadialHeight, RadialSource, SrcRef};
use camcraft_app::posting;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::verify::Warning;
use camcraft_app::world::new_world;

const R: f64 = 3.0; // the BN-0600's ball
/// The turned bar's meridian (rho, z): wall r6, shoulder, neck r5.
const MERIDIAN: [(f64, f64); 6] = [(0.0, -16.0), (6.0, -16.0), (6.0, -2.0), (5.0, 2.0), (5.0, 16.0), (0.0, 16.0)];

fn revolve(n: usize) -> Vec<Triangle> {
    let pt = |rho: f64, z: f64, j: usize| {
        let th = std::f64::consts::TAU * j as f64 / n as f64;
        Point3::new(rho * th.cos(), rho * th.sin(), z)
    };
    let mut f = Vec::new();
    for w in MERIDIAN.windows(2) {
        let ((ra, za), (rb, zb)) = (w[0], w[1]);
        for j in 0..n {
            let k = (j + 1) % n;
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

/// The bar laid down: axis world x, crown world z.
fn laid() -> Vec<Triangle> {
    let s = |p: Point3| Point3::new(p.z, p.x, p.y);
    revolve(16).iter().map(|t| Triangle::new(s(t.a), s(t.b), s(t.c))).collect()
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Contour,
    Parallel,
    Offset,
}

fn params(kind: Kind, clearance: f64) -> OpParams {
    match kind {
        Kind::Contour => OpParams::RotaryContour {
            tolerance: 0.1,
            clearance_z: Height::fixed(clearance),
            retract_z: None,
            top_z: Height::fixed(14.0),
            bottom_z: Height::fixed(-14.0),
            clearance_r: None,
            retract_r: None,
            top_r: None,
            bottom_r: None,
            feed: 500.0,
            plunge_feed: 100.0,
            climb: true,
            lead_deg: 0.0,
            lag_deg: 0.0,
            max_tilt_deg: 60.0,
            stated_axis: None,
            picks: Default::default(),
        },
        // The crown and upper flanks: a laid bar's passes stay on the
        // half the head can reach from above (B never past +-90).
        Kind::Parallel => OpParams::RotaryParallel {
            tolerance: 0.1,
            stepover: Some(2.0),
            clearance_z: Height::fixed(clearance),
            retract_z: None,
            top_z: Height::fixed(6.0),
            bottom_z: Height::fixed(1.0),
            clearance_r: None,
            retract_r: None,
            top_r: None,
            bottom_r: None,
            feed: 500.0,
            plunge_feed: 100.0,
            climb: true,
            lead_deg: 0.0,
            lag_deg: 0.0,
            max_tilt_deg: 60.0,
            stated_axis: None,
            picks: Default::default(),
        },
        Kind::Offset => OpParams::RotaryOffset {
            links: camcraft_app::links::Links::ROUGH3D,
            tolerance: 0.1,
            stepover: 4.0,
            stock_to_leave: 0.5,
            clearance_z: Height::fixed(clearance),
            top_z: Height::fixed(14.0),
            bottom_z: Height::fixed(-14.0),
            clearance_r: None,
            top_r: None,
            stepdown: 0.5,
            feed: 500.0,
            plunge_feed: 100.0,
            climb: true,
            strategy: camcraft_app::machining2d::PocketStrategy::Auto,
            entry: camcraft_app::machining3d::Entry3D::Plunge,
            helix_radius: 1.0,
            optimal_load: camcraft_app::load::DEFAULT_OPTIMAL_LOAD,
            lead_deg: 0.0,
            lag_deg: 0.0,
            max_tilt_deg: 60.0,
            stated_axis: None,
            picks: Default::default(),
        },
    }
}

/// A coaxial round bar of `dia` over the part's 32 mm length - the
/// stock a rotary job is clamped from (standing for the contour and
/// the rough, laid along x for the parallel).
fn world_for(kind: Kind, dia: f64, clearance: f64) -> World {
    world_with(kind, dia, params(kind, clearance))
}

fn world_with(kind: Kind, dia: f64, op: OpParams) -> World {
    let world = new_world();
    let mut h = History::default();
    let tool = catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", tool).expect("tool"));
    let lying = kind == Kind::Parallel;
    let loc = if lying {
        Location { pos: Point3::new(-16.0, 0.0, 0.0), rot: quat_from_axis_angle(1, std::f64::consts::FRAC_PI_2) }
    } else {
        Location { pos: Point3::new(0.0, 0.0, -16.0), rot: IDENTITY_Q }
    };
    let stock = SetStock::new(&world, StockShape::Cylinder { diameter: dia, height: 32.0 }, loc);
    h.execute(&world, stock.expect("bar stock"));
    let mesh = if lying { laid() } else { revolve(16) };
    h.execute(&world, ImportModel::from_mesh("bar", Mesh::from_facets(mesh)));
    let op = CreateOperation::new(&world, "OP", "BN-0600", SrcRef::Entity("bar".into()), op);
    h.execute(&world, op.expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "OP").expect("generate"));
    // The tilting head owns the letters; G54 sits above the work (the
    // tilt suite's parked-anchor law; inside its -500..500 band, #457).
    let asset = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/machines/tilthead.mch");
    h.execute(&world, LoadMachine::from_file(&world, &asset).expect("mount"));
    let g54 = Location { pos: Point3::new(0.0, 0.0, 60.0), rot: IDENTITY_Q };
    h.execute(&world, SetWcsOrigin::new(&world, g54).expect("G54"));
    world
}

fn cache(world: &World) -> camcraft_app::schema::Toolpath {
    model::find_by_name(world, "OP").and_then(|e| model::toolpath_of(world, &e)).expect("cached")
}

/// The verdicts that stop the post: the non-blocking notes (M13.9's
/// scallop reading) ride along and are judged by their own tests.
fn blocking(c: &camcraft_app::schema::Toolpath) -> Vec<&Warning> {
    c.warnings.iter().filter(|w| w.blocks_post()).collect()
}

fn status(world: &World) -> OpStatus {
    let e = model::find_by_name(world, "OP").expect("op");
    operation::status_of(world, &e)
}

fn tmp_nc(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("camcraft-rotary-{tag}-{}.nc", std::process::id()));
    let _ = std::fs::remove_file(&p);
    p
}

/// Signed distance of a world point to the bar's meridian polyline
/// (negative inside): the analytic skin the 16-turn facets approach.
fn skin(p: Point3, lying: bool) -> f64 {
    let (rho, ax) = if lying { ((p.y * p.y + p.z * p.z).sqrt(), p.x) } else { ((p.x * p.x + p.y * p.y).sqrt(), p.z) };
    let mut best = f64::INFINITY;
    for w in MERIDIAN.windows(2) {
        let ((r0, z0), (r1, z1)) = (w[0], w[1]);
        let (dx, dz) = (r1 - r0, z1 - z0);
        let t = (((rho - r0) * dx + (ax - z0) * dz) / (dx * dx + dz * dz)).clamp(0.0, 1.0);
        best = best.min(((rho - r0 - t * dx).powi(2) + (ax - z0 - t * dz).powi(2)).sqrt());
    }
    let inside = rho < 6.0 && ax.abs() < 16.0 && (rho < 5.0 || ax < 2.0);
    if inside { -best } else { best }
}

/// The ball the stream places - tip plus r ALONG THE POSE - stands on
/// the skin: within the 16-gon's chord dip (r6 * (1 - cos 11.25°) =
/// 0.115 mm, the analytic circle vs the facets the ladder rode) plus
/// the kind's 0.1 tolerance of tangency, never a radius off it.
/// `leave` is the kind's planned wall (the rough's ladder floor stands
/// up to one infeed plus its leave above the skin).
fn ball_rides_the_skin(kind: Kind, lying: bool, leave: f64) {
    const SLACK: f64 = 0.115 + 0.1 + 0.01;
    let world = world_for(kind, 12.4, 25.0);
    let t = cache(&world).path;
    let mut n = 0usize;
    let mut far = 0usize;
    for m in t.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        let Some(a) = m.axis else { continue };
        let c = Point3::new(m.to.x + R * a[0], m.to.y + R * a[1], m.to.z + R * a[2]);
        let gap = skin(c, lying) - R;
        assert!(gap > -SLACK, "the ball sinks {gap:.3} mm into the skin at {:?}", m.to);
        if gap.abs() < 21.0 {
            n += 1;
            if gap > leave + SLACK {
                far += 1;
            }
        }
    }
    assert!(n > 50, "a whole bar of contact moves, got {n}");
    // Passes run off the bar's ends into air (the raster's overrun),
    // so "far" is measured against the moves near the work only - and
    // most of them must be ON the planned wall.
    assert!(far * 4 < n, "{far} of {n} cut moves stand off the planned wall - the pose turned about the tip");
}

#[test]
fn the_rotary_ball_is_the_planned_ball() {
    ball_rides_the_skin(Kind::Contour, false, 0.0);
    ball_rides_the_skin(Kind::Parallel, true, 0.0);
    ball_rides_the_skin(Kind::Offset, false, 1.0);
}

fn posts_clean(kind: Kind, tag: &str) {
    let world = world_for(kind, 12.4, 25.0);
    assert_eq!(status(&world), OpStatus::Ok, "{tag}: fresh");
    let c = cache(&world);
    assert!(blocking(&c).is_empty(), "{tag}: a sensible rotary setup verifies silent: {:?}", blocking(&c));
    let rapids_posed = c.path.moves.iter().filter(|m| m.kind == MoveKind::Rapid).all(|m| m.axis.is_some());
    assert!(rapids_posed, "{tag}: every rapid states the pose the machine holds (the tilt is modal)");
    let report = posting::verification_report(&world);
    assert!(report.problems.is_empty(), "{tag}: {:?}", report.problems);
    let path = tmp_nc(tag);
    let posted = posting::post_setup(&world, &path).expect("a clean rotary program posts");
    assert!(posted.program.contains("G43.4"), "{tag}: RTCP armed for the tilted records");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn rotary_contour_on_a_round_bar_verifies_clean_and_posts() {
    posts_clean(Kind::Contour, "rc");
}

#[test]
fn rotary_parallel_on_a_laid_bar_verifies_clean_and_posts() {
    posts_clean(Kind::Parallel, "rp");
}

/// #2970: with Bottom at the bar's underside the passes go all the
/// way round, and the tilting head (B -120..120) cannot point the tool
/// up at the underside: the machine check refuses by axis, angle and
/// travel, and nothing posts. A Bottom over the axis (the case above)
/// keeps to the arc the head reaches.
#[test]
fn a_full_turn_rotary_parallel_refuses_on_a_head_that_cannot_reach_under() {
    let OpParams::RotaryParallel { tolerance, stepover, clearance_z, retract_z, top_z, feed, plunge_feed, climb, lead_deg, lag_deg, max_tilt_deg, .. } =
        params(Kind::Parallel, 25.0)
    else {
        unreachable!("the parallel params")
    };
    let op = OpParams::RotaryParallel { tolerance, stepover, clearance_z, retract_z, top_z, bottom_z: Height::fixed(-6.2), clearance_r: None, retract_r: None, top_r: None, bottom_r: None, feed, plunge_feed, climb, lead_deg, lag_deg, max_tilt_deg, stated_axis: None, picks: Default::default() };
    let world = world_with(Kind::Parallel, 12.4, op);
    let report = posting::verification_report(&world);
    let text = format!("{:?}", report.problems);
    assert!(text.contains(": move ") && text.contains(" commands B ") && text.contains("the travel is -120.000..120.000"), "{text}");
    blocks(&world, "rp-full");
}

#[test]
fn rotary_offset_on_a_round_bar_verifies_clean_and_posts() {
    posts_clean(Kind::Offset, "ro");
}

fn blocks(world: &World, tag: &str) {
    let path = tmp_nc(tag);
    let err = posting::post_setup(world, &path).expect_err("a convicted rotary program never posts");
    assert!(!err.is_empty(), "{tag}: the refusal names its reason");
    assert!(!path.exists(), "{tag}: nothing written");
}

#[test]
fn a_finish_ring_buried_in_an_oversized_bar_convicts_its_shank() {
    // d50 over a d12 part: the flute reaches 15 mm out from the skin,
    // the shank band starts there - inside 25 mm of uncut bar.
    let world = world_for(Kind::Contour, 50.0, 40.0);
    let w = cache(&world).warnings;
    assert!(
        w.iter().any(|w| matches!(w, Warning::ShankHit { part: false, .. })),
        "the shank in uncut bar must convict: {:?}",
        &w[..w.len().min(6)]
    );
    blocks(&world, "rc-big");
}

#[test]
fn a_clearance_plane_inside_the_bar_convicts_the_rapids() {
    // Clearance 10 under a stock top of 16: the ring-to-ring hops run
    // through the bar's top that no ring above them ever cut.
    let world = world_for(Kind::Contour, 12.4, 10.0);
    let w = cache(&world).warnings;
    assert!(
        w.iter().any(|w| matches!(w, Warning::RapidInStock { .. })),
        "rapids through uncut stock must convict: {:?}",
        &w[..w.len().min(6)]
    );
    blocks(&world, "rc-low");
}

#[test]
fn a_rough_over_an_oversized_bar_peels_from_the_stock_radius() {
    // M13.8: the wrap ladder once peeled from the PART's OD inward, so
    // a d20 bar's skin from r6 to r10 was never cut and the hops one
    // infeed and a radius outside the part's skin ran straight through
    // it (the radial field law convicted them). It now measures from
    // the STOCK's radius: the first level peels the bar's own skin,
    // the rapids ride outside it, and the program verifies silent.
    let world = world_for(Kind::Offset, 20.0, 25.0);
    let c = cache(&world);
    assert!(blocking(&c).is_empty(), "the rough over its own bar verifies silent: {:?}", blocking(&c));
    let rho = |p: Point3| (p.x * p.x + p.y * p.y).sqrt();
    let top = c.path.moves.iter().filter(|m| m.kind != MoveKind::Rapid).map(|m| rho(m.to)).fold(0.0f64, f64::max);
    assert!(top >= 10.0 - 0.5 - 1e-6, "the ladder still starts inside the bar's skin: top level {top:.3}");
    let path = tmp_nc("ro-big");
    assert!(posting::post_setup(&world, &path).is_ok(), "a clean rough posts");
    let _ = std::fs::remove_file(&path);
    // M13.8 air laps: the levels above the part's skin clear one lap,
    // not the developed strip's two (the seam copy is for planes that
    // meet the part). Before: 11,967 moves here vs 2,064 on the
    // matching bar; after: 7,082 - the seven air levels each sweep
    // half what they did. The matching bar has no air level, so its
    // program is the unsplit ladder's, move for move.
    // #3168 (PR #3496) closes each Auto/Zigzag wrap level with a seam
    // ring: the matching bar went 2,064 -> 2,110 moves (the rings), the
    // d20 bar 7,648 -> 7,578. The air-lap law above is unchanged (#3774).
    let snug = cache(&world_for(Kind::Offset, 12.4, 25.0)).path.moves.len();
    let big = c.path.moves.len();
    eprintln!("ROTARY OFFSET | d20 bar {big} moves | d12.4 bar {snug} moves");
    assert_eq!(snug, 2110, "the matching bar has no air level - its ladder must not move");
    assert!(big < 9000, "air levels still clear both laps: {big} moves (11,967 when they did)");
}

/// (station, radius, azimuth) about the bar's axis (z standing, x laid).
fn cyl(p: Point3, lying: bool) -> (f64, f64, f64) {
    let (v, a, b) = if lying { (p.x, p.y, p.z) } else { (p.z, p.x, p.y) };
    (v, (a * a + b * b).sqrt(), b.atan2(a))
}

fn dang(a: f64, b: f64) -> f64 {
    (a - b + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

#[test]
fn every_rotary_program_opens_on_the_clearance_pose() {
    // M13.8: the first record is a rapid that STANDS at the clearance
    // pose over the first cut - off the axis, on the first cut's own
    // azimuth, holding the pure radial - never a sweep in from the
    // planning frame's home column; and the verdict stays silent.
    for (kind, lying, tag) in [(Kind::Contour, false, "rc"), (Kind::Parallel, true, "rp"), (Kind::Offset, false, "ro")] {
        let c = cache(&world_for(kind, 12.4, 25.0));
        assert!(blocking(&c).is_empty(), "{tag}: {:?}", blocking(&c));
        let m0 = &c.path.moves[0];
        assert_eq!(m0.kind, MoveKind::Rapid, "{tag}: the program opens at rapid");
        assert_eq!(m0.from, m0.to, "{tag}: the opening record stands at its pose, sweeping nothing");
        let fc = c.path.moves.iter().find(|m| m.kind != MoveKind::Rapid).expect("a cut");
        let (s, f) = (cyl(m0.from, lying), cyl(fc.from, lying));
        assert!(s.1 > 0.5, "{tag}: the program opens on the axis: {:?}", m0.from);
        assert!(dang(s.2, f.2).abs() < 1e-6, "{tag}: opening azimuth {:.4} vs first cut {:.4}", s.2, f.2);
        match kind {
            Kind::Contour => assert!(s.0 >= 25.0 - 1e-9, "{tag}: opening station {} under the clearance", s.0),
            // The parallel's 25 above the laid axis and the wrap's
            // r6.2 bar + 0.5 infeed + 3 ball, as TIP radii.
            Kind::Parallel => assert!(s.1 >= 25.0 - 1e-6, "{tag}: opening radius {}", s.1),
            Kind::Offset => assert!(s.1 >= 6.2 + 0.5 + R - 1e-3, "{tag}: opening radius {}", s.1),
        }
        let a = m0.axis.expect("the opening pose is stated");
        let ctr = Point3::new(m0.to.x + R * a[0], m0.to.y + R * a[1], m0.to.z + R * a[2]);
        let (_, l, az) = cyl(ctr, lying);
        let want = if lying { [0.0, az.cos(), az.sin()] } else { [az.cos(), az.sin(), 0.0] };
        assert!(l > 0.5 && a.iter().zip(want).map(|(x, y)| x * y).sum::<f64>() > 1.0 - 1e-9, "{tag}: the opening pose is not the radial: {a:?}");
    }
}

#[test]
fn rotary_parallel_retracts_along_the_radius_and_verifies_clean() {
    // M13.8: every pass leaves along its radius to the clearance
    // cylinder (25 above the laid axis), indexes on it, and comes back
    // in along the next pass's radius - no world-Z lift.
    let c = cache(&world_for(Kind::Parallel, 12.4, 25.0));
    assert!(blocking(&c).is_empty(), "{:?}", blocking(&c));
    let radial = |m: &camcraft_app::cl::Move| {
        let (a, b) = (cyl(m.from, true), cyl(m.to, true));
        (a.0 - b.0).abs() < 1e-6 && dang(a.2, b.2).abs() < 1e-6
    };
    let mut retracts = 0usize;
    for (i, m) in c.path.moves.iter().enumerate() {
        let (from, to) = (cyl(m.from, true).1, cyl(m.to, true).1);
        match m.kind {
            MoveKind::Rapid => {
                assert!(to >= 25.0 - 1e-6, "rapid {i} ends inside the clearance radius: {to:.4}");
                assert!(from >= 25.0 - 1e-6 || radial(m), "rapid {i} climbs off the radius");
            }
            MoveKind::Plunge => assert!(radial(m), "plunge {i} comes in off the radius"),
            MoveKind::Feed if to >= 25.0 - 1e-6 => {
                assert!(radial(m), "retract {i} is not radial: {:?} -> {:?}", m.from, m.to);
                retracts += 1;
            }
            MoveKind::Feed => {}
        }
    }
    assert!(retracts > 3, "passes retract, got {retracts}");
}

/// A standing bar with one clean rotary contour on it, for the radial
/// door's tests (#2774). The bar: part r6, stock d12.4, so the work floor
/// about the axis is 6.2 mm.
fn bar_with_contour(radial: [Option<RadialHeight>; 4]) -> World {
    let world = new_world();
    let mut h = History::default();
    let tool = catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", tool).expect("tool"));
    let loc = Location { pos: Point3::new(0.0, 0.0, -16.0), rot: IDENTITY_Q };
    h.execute(&world, SetStock::new(&world, StockShape::Cylinder { diameter: 12.4, height: 32.0 }, loc).expect("bar stock"));
    h.execute(&world, ImportModel::from_mesh("bar", Mesh::from_facets(revolve(16))));
    let mut params = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { clearance_r, retract_r, top_r, bottom_r, .. } = &mut params {
        *clearance_r = radial[0];
        *retract_r = radial[1];
        *top_r = radial[2];
        *bottom_r = radial[3];
    }
    let op = CreateOperation::new(&world, "OP", "BN-0600", SrcRef::Entity("bar".into()), params).expect("create");
    h.execute(&world, op);
    world
}

#[test]
fn a_radial_clearance_inside_the_work_is_refused_by_name() {
    // #2774: a rapid cylinder at r5.9 about the axis - 0.3 mm inside the
    // d12.4 bar's own skin - would drive the tool through metal on every
    // hop. The door refuses it when the command is made, not at Generate.
    let world = bar_with_contour([None, None, None, None]);
    let mut p = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { clearance_r, .. } = &mut p {
        *clearance_r = Some(RadialHeight::of(RadialSource::StockOd, -0.3));
    }
    let e = CreateOperation::new(&world, "OP2", "BN-0600", SrcRef::Entity("bar".into()), p)
        .map(|_| ())
        .expect_err("a clearance radius inside the bar must refuse the command");
    let msg = e.to_string();
    assert!(msg.contains("Clearance radius") && msg.contains("stands inside"), "{msg}");
}

#[test]
fn a_radial_edit_inside_the_work_keeps_the_stored_heights() {
    // The edit path answers the same door: the bad retract radius is
    // refused at the command, and the op keeps the heights it stored.
    let world = bar_with_contour([
        Some(RadialHeight::of(RadialSource::StockOd, 5.0)),
        Some(RadialHeight::of(RadialSource::StockOd, 10.0)),
        None,
        None,
    ]);
    let mut bad = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { clearance_r, retract_r, .. } = &mut bad {
        *clearance_r = Some(RadialHeight::of(RadialSource::StockOd, 5.0));
        *retract_r = Some(RadialHeight::of(RadialSource::StockOd, -0.3));
    }
    let e = EditOperationParams::new(&world, "OP", bad).map(|_| ()).expect_err("a retract radius inside the bar must refuse the edit");
    let msg = e.to_string();
    assert!(msg.contains("Retract radius") && msg.contains("stands inside"), "{msg}");
    let op = model::operation_of(&world, &model::find_by_name(&world, "OP").expect("op")).expect("params");
    if let OpParams::RotaryContour { retract_r, .. } = op.params {
        assert_eq!(retract_r, Some(RadialHeight::of(RadialSource::StockOd, 10.0)), "the refused edit must not touch the stored heights");
    }
}

#[test]
fn a_radial_band_crossed_is_refused_by_name() {
    // #2774: Top r8 under Bottom r9 - the rho window empty - is refused
    // at the command, with the two names in the words.
    let world = bar_with_contour([None, None, None, None]);
    let mut p = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { top_r, bottom_r, .. } = &mut p {
        *top_r = Some(RadialHeight::absolute(8.0));
        *bottom_r = Some(RadialHeight::absolute(9.0));
    }
    let e = CreateOperation::new(&world, "OP2", "BN-0600", SrcRef::Entity("bar".into()), p)
        .map(|_| ())
        .expect_err("a top radius under the bottom radius must refuse the command");
    let msg = e.to_string();
    assert!(msg.contains("Top radius") && msg.contains("Bottom radius"), "{msg}");
}

#[test]
fn a_band_at_the_clearance_cylinder_is_refused_by_name() {
    // #2774: the rapids ride the clearance cylinder. A top radius at or
    // above it puts a cut where the hop travels - refused when the
    // command is made.
    let world = bar_with_contour([None, None, None, None]);
    let mut p = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { clearance_r, top_r, .. } = &mut p {
        *clearance_r = Some(RadialHeight::absolute(8.0));
        *top_r = Some(RadialHeight::absolute(8.0));
    }
    let e = CreateOperation::new(&world, "OP2", "BN-0600", SrcRef::Entity("bar".into()), p)
        .map(|_| ())
        .expect_err("a band at the rapids cylinder must refuse the command");
    let msg = e.to_string();
    assert!(msg.contains("Top radius") && msg.contains("Clearance radius"), "{msg}");
}

#[test]
fn a_stated_axis_resolves_the_radii_generation_judges() {
    // #2774 review: the resolver must read the radii about the SAME axis
    // generation turns about - a stated axis judged against the model's
    // faces (`axis_stated::judged_any`), never the free fit alone. On the
    // R5 sphere (12 latitudes, 24 longitudes) the fit refuses - every
    // axis fits the model equally - but the op's stated Z axis is judged
    // fine by the very law generation runs, so a stated clearance radius
    // must resolve (and store), not refuse on a fit the generator never
    // runs.
    let world = new_world();
    let mut h = History::default();
    let tool = catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", tool).expect("tool"));
    h.execute(&world, ImportModel::from_mesh("ball", Mesh::from_facets(sphere(5.0, 24, 12))));
    let mut p = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { stated_axis, clearance_r, .. } = &mut p {
        *stated_axis = Some([0.0, 0.0, 1.0]);
        *clearance_r = Some(RadialHeight::absolute(8.0));
    }
    CreateOperation::new(&world, "OP", "BN-0600", SrcRef::Entity("ball".into()), p)
        .expect("the stated axis carries the radial resolution");
}

/// A UV sphere of radius `r` about the origin, `nlon` x `nlat` quads - the
/// model whose free fit refuses ("every axis fits this model equally")
/// while a stated axis through its centre is judged fine.
fn sphere(r: f64, nlon: usize, nlat: usize) -> Vec<Triangle> {
    let pt = |i: usize, j: usize| {
        let ph = std::f64::consts::PI * (i as f64 / nlat as f64 - 0.5);
        let th = std::f64::consts::TAU * j as f64 / nlon as f64;
        Point3::new(r * ph.cos() * th.cos(), r * ph.cos() * th.sin(), r * ph.sin())
    };
    let mut s = Vec::new();
    for i in 0..nlat {
        for j in 0..nlon {
            if i > 0 {
                s.push(Triangle::new(pt(i, j), pt(i + 1, j), pt(i + 1, j + 1)));
            }
            if i + 1 < nlat {
                s.push(Triangle::new(pt(i, j), pt(i + 1, j + 1), pt(i, j + 1)));
            }
        }
    }
    s
}

#[test]
fn a_radial_radius_that_is_no_radius_is_refused_at_the_command() {
    // #2774 review: every set radial field is validated before it is
    // stored - on create, edit, paste and retype through the same
    // `is_sane` door. A NaN offset and an absolute radius under the axis
    // are no radius at all: refused here, never stored for Generate to
    // trip over.
    let world = bar_with_contour([None, None, None, None]);
    let mut p = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { clearance_r, .. } = &mut p {
        *clearance_r = Some(RadialHeight::of(RadialSource::StockOd, f64::NAN));
    }
    let e = CreateOperation::new(&world, "OP2", "BN-0600", SrcRef::Entity("bar".into()), p)
        .map(|_| ())
        .expect_err("a NaN radial offset must refuse the command");
    assert!(e.to_string().contains("not usable"), "{e}");
    let mut p = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { bottom_r, .. } = &mut p {
        *bottom_r = Some(RadialHeight::absolute(-1.0));
    }
    let e = CreateOperation::new(&world, "OP2", "BN-0600", SrcRef::Entity("bar".into()), p)
        .map(|_| ())
        .expect_err("an absolute Bottom radius under the axis must refuse the command");
    assert!(e.to_string().contains("not usable"), "{e}");
}

#[test]
fn a_radial_radius_its_source_cannot_answer_is_refused_by_name() {
    // #2774 review: the gate no longer discards a resolution failure - a
    // stock-Od radius with no stock names the setup page at the command,
    // never passes silent for the generator to refuse later.
    let world = new_world();
    let mut h = History::default();
    let tool = catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", tool).expect("tool"));
    h.execute(&world, ImportModel::from_mesh("bar", Mesh::from_facets(revolve(16))));
    let mut p = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { clearance_r, .. } = &mut p {
        *clearance_r = Some(RadialHeight::of(RadialSource::StockOd, 5.0));
    }
    let e = CreateOperation::new(&world, "OP", "BN-0600", SrcRef::Entity("bar".into()), p)
        .map(|_| ())
        .expect_err("a stock-Od radius with no stock must refuse the command");
    let msg = e.to_string();
    assert!(msg.contains("Clearance radius") && msg.contains("no stock is set"), "{msg}");
}

#[test]
fn a_band_that_crosses_no_skin_is_refused_by_name() {
    // #2774's Prime Directive door at the generator: the tool ball's
    // reach about this bar's axis spans 0..6.0 mm - its skin stands at
    // r5..r6. A band r6.05..r6.5 above that reach cuts nothing: the
    // generate refuses with both radii in the words, never an empty
    // path and never the uncut one.
    let e = parallel_generate([None, None, Some(RadialHeight::absolute(6.5)), Some(RadialHeight::absolute(6.05))])
        .expect_err("a band that crosses no skin must refuse the generate");
    assert!(e.contains("6.05") && e.contains("6.5") && e.contains("crosses no part skin"), "{e}");
}

#[test]
fn the_radial_heights_survive_the_project_round_trip() {
    // #2774: the file stores the expression, never a resolved radius. The
    // read-back is equal field for field, and a v84 writer (no radial
    // keys) reads back as the Z-plane law - every v84 op's byte-identical
    // behaviour, the no-op step the v85 rung records.
    let mut p = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { clearance_r, retract_r, top_r, bottom_r, .. } = &mut p {
        *clearance_r = Some(RadialHeight::of(RadialSource::StockOd, 5.0));
        *retract_r = Some(RadialHeight::of(RadialSource::StockOd, 10.0));
        *top_r = Some(RadialHeight::of(RadialSource::ModelMaxRadius, -0.5));
        *bottom_r = Some(RadialHeight::absolute(2.5));
    }
    let saved = camcraft_app::project::params_to_json(&p);
    assert_eq!(camcraft_app::project::params_from_json(&saved).expect("round trip"), p);
    let mut v73 = saved.clone();
    let obj = v73.get_mut("rotary_contour").and_then(serde_json::Value::as_object_mut).expect("rotary contour fields");
    for k in ["clearance_r", "retract_r", "top_r", "bottom_r"] {
        assert!(obj.remove(k).is_some(), "the save names {k}");
    }
    let mut plain = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { clearance_r, .. } = &mut plain {
        *clearance_r = Some(RadialHeight::of(RadialSource::StockOd, 5.0));
    }
    let mut half = v73.clone();
    let obj = half.get_mut("rotary_contour").and_then(serde_json::Value::as_object_mut).expect("rotary contour fields");
    obj.insert("clearance_r".to_string(), saved["rotary_contour"]["clearance_r"].clone());
    assert_eq!(camcraft_app::project::params_from_json(&half).expect("a v73 file plus one radial key"), plain);
    assert_eq!(camcraft_app::project::params_from_json(&v73).expect("a v73 file"), params(Kind::Contour, 25.0));
}

/// The bar laid along x with one Rotary Parallel that STATES its radial
/// heights (#2774): clearance r8, retract r6.5, band r4.0..r5.5. The Z
/// fields stay at the plain fixture's numbers (clearance 25, band
/// 1.0..6.0) - the test reads which numbers the path actually moved to.
fn parallel_world(radii: [Option<RadialHeight>; 4]) -> World {
    parallel_generate(radii).expect("generate")
}

/// The same build, with the generator's answer in hand instead of a panic:
/// a band that crosses no skin must refuse there.
fn parallel_generate(radii: [Option<RadialHeight>; 4]) -> Result<World, String> {
    let world = new_world();
    let mut h = History::default();
    let tool = catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", tool).expect("tool"));
    let loc = Location { pos: Point3::new(-16.0, 0.0, 0.0), rot: quat_from_axis_angle(1, std::f64::consts::FRAC_PI_2) };
    h.execute(&world, SetStock::new(&world, StockShape::Cylinder { diameter: 12.4, height: 32.0 }, loc).expect("bar stock"));
    h.execute(&world, ImportModel::from_mesh("bar", Mesh::from_facets(laid())));
    let mut p = params(Kind::Parallel, 25.0);
    if let OpParams::RotaryParallel { stepover, clearance_r, retract_r, top_r, bottom_r, .. } = &mut p {
        *stepover = Some(0.5);
        *clearance_r = radii[0];
        *retract_r = radii[1];
        *top_r = radii[2];
        *bottom_r = radii[3];
    }
    h.execute(&world, CreateOperation::new(&world, "OP", "BN-0600", SrcRef::Entity("bar".into()), p).expect("create"));
    let g = GenerateOperation::new(&world, "OP").map_err(|e| e.to_string())?;
    h.execute(&world, g);
    Ok(world)
}

/// The standing bar with one Rotary Contour that STATES its radial
/// heights (#2774), generated: the contour's link law is the one that
/// converts stated TIP radii to the planning centers, so its envelope
/// answers the reviewer's probe.
fn contour_generate(radii: [Option<RadialHeight>; 4]) -> Result<World, String> {
    let world = new_world();
    let mut h = History::default();
    let tool = catalog().into_iter().find(|(n, _)| *n == "BN-0600").map(|(_, t)| t).expect("catalog ball");
    h.execute(&world, AddTool::new(&world, "BN-0600", tool).expect("tool"));
    let loc = Location { pos: Point3::new(0.0, 0.0, -16.0), rot: IDENTITY_Q };
    h.execute(&world, SetStock::new(&world, StockShape::Cylinder { diameter: 12.4, height: 32.0 }, loc).expect("bar stock"));
    h.execute(&world, ImportModel::from_mesh("bar", Mesh::from_facets(revolve(16))));
    let mut p = params(Kind::Contour, 25.0);
    if let OpParams::RotaryContour { clearance_r, retract_r, top_r, bottom_r, .. } = &mut p {
        *clearance_r = radii[0];
        *retract_r = radii[1];
        *top_r = radii[2];
        *bottom_r = radii[3];
    }
    h.execute(&world, CreateOperation::new(&world, "OP", "BN-0600", SrcRef::Entity("bar".into()), p).expect("create"));
    let g = GenerateOperation::new(&world, "OP").map_err(|e| e.to_string())?;
    h.execute(&world, g);
    Ok(world)
}

#[test]
fn the_stated_contour_radii_clear_the_work() {
    // #2774 review probe (the P1-class one): stated clearance R8 and
    // retract R6.5 on the d12.4 bar used to emit the exit at
    // R6.5 - one ball radius (the 3 mm BN-0600 ball): the verifier read
    // `Envelope { gouge_mm: 2.5 }` into the r6 wall and RapidInStock
    // warnings, and blocked the post. The link law converts each stated
    // TIP radius to the planning center one ball OUTSIDE it, so the
    // posed tips stand at the stated radii and the program verifies
    // silent - the exit at R8, every rapid at or over R6.5.
    let radii = [Some(RadialHeight::absolute(8.0)), Some(RadialHeight::absolute(6.5)), None, None];
    let world = contour_generate(radii).expect("generate");
    let c = cache(&world);
    assert!(blocking(&c).is_empty(), "{:?}", blocking(&c));
    let rho = |q: Point3| (q.x * q.x + q.y * q.y).sqrt();
    for (i, m) in c.path.moves.iter().enumerate() {
        if m.kind == MoveKind::Rapid {
            assert!(rho(m.to) >= 6.5 - 1e-6, "rapid {i} ends at {}, under the stated hop 6.5", rho(m.to));
        }
    }
    let last = c.path.moves.last().expect("the exit rapid");
    assert_eq!(last.kind, MoveKind::Rapid);
    assert!((rho(last.to) - 8.0).abs() < 1e-6, "the exit stands at the stated clearance radius 8, got {}", rho(last.to));
    let opens = rho(c.path.moves.first().expect("the opening record").from);
    assert!((opens - 8.0).abs() < 0.02, "the program opens at the stated clearance radius 8, got {opens:.3}");
}

#[test]
fn the_stated_radial_heights_move_the_generated_path() {
    // #2774's Done-means at the document level: the radii the operation
    // states are the radii the generator moves to, not the Z numbers.
    // Clearance 8 and retract 6.5 replace the plane's 25; the band
    // 4.0..5.5 cuts a shell off the r6 bar's crown and leaves its
    // bottom. The Z-plane twin answers with its own numbers.
    let radii = [
        Some(RadialHeight::absolute(8.0)),
        Some(RadialHeight::absolute(6.5)),
        Some(RadialHeight::absolute(5.5)),
        Some(RadialHeight::absolute(4.0)),
    ];
    let c = cache(&parallel_world(radii));
    assert!(blocking(&c).is_empty(), "{:?}", blocking(&c));
    let rho = |q: Point3| (q.y * q.y + q.z * q.z).sqrt();
    // The rapids ride the cylinders, never the Z-plane's 25.
    let mut rapid_max = 0.0_f64;
    let mut on_hop = 0usize;
    for m in c.path.moves.iter() {
        if m.kind == MoveKind::Rapid {
            let r = rho(m.to);
            assert!(r >= 6.5 - 1e-6, "a rapid drops under the hop radius: {r:.3}");
            rapid_max = rapid_max.max(r);
        } else if m.kind == MoveKind::Feed && (rho(m.to) - 6.5).abs() < 0.02 && rho(m.from) < 6.4 {
            on_hop += 1; // a pass retracts to the hop 6.5, not to 25
        }
    }
    let opens = rho(c.path.moves.first().expect("the opening record").from);
    assert!((opens - 8.0).abs() < 0.02, "the program opens at the clearance radius 8, got {opens:.3}");
    assert!(rapid_max < 20.0, "the rapids still ride the Z-plane's 25: {rapid_max:.3}");
    assert!(on_hop > 3, "passes retract to the hop cylinder, got {on_hop}");
    // #2774's band law: the cut tips stand BETWEEN the two cylinders about
    // the axis (rho), not under a Z plane. The r5 neck's skin sits inside
    // the band 4.0..5.5 and stays finished; the r6 wall's crown (rho ~6)
    // and the lower flank rows (rho ~1) stand left.
    let band = |r: f64| (4.0 - 0.05..=5.5 + 0.05).contains(&r);
    let (mut cut_max, mut cut_min) = (f64::NEG_INFINITY, f64::INFINITY);
    let mut cut_tips = 0usize;
    for m in c.path.moves.iter() {
        match m.kind {
            MoveKind::Plunge => {
                let r = rho(m.to);
                assert!(band(r), "a plunge lands off the band: {r:.3}");
                cut_tips += 1;
            }
            MoveKind::Feed => {
                let (a, b) = (rho(m.from), rho(m.to));
                if a < 6.4 && b < 6.4 {
                    assert!(band(a) && band(b), "a cut tip leaves the band: {a:.3} -> {b:.3}");
                    cut_max = cut_max.max(a.max(b));
                    cut_min = cut_min.min(a.min(b));
                    cut_tips += 1;
                }
            }
            MoveKind::Rapid => {}
        }
    }
    assert!(cut_tips > 50, "the band left no cut at all, got {cut_tips}");
    assert!(cut_max <= 5.5 + 0.05, "a cut crosses the top cylinder: rho {cut_max:.3}");
    assert!(cut_min >= 4.0 - 0.05, "a cut drops under the bottom cylinder: rho {cut_min:.3}");
    assert!(cut_max >= 4.95, "the band must still finish the r5 neck skin: {cut_max:.3}");
    // The Z-plane twin cuts its window on down to rho ~1 - the band's
    // floor at 4.0 left those flank rows standing.
    let plain = cache(&world_for(Kind::Parallel, 12.4, 25.0));
    let twin_max = plain.path.moves.iter().filter(|m| m.kind == MoveKind::Rapid).map(|m| rho(m.to)).fold(0.0_f64, f64::max);
    assert!(twin_max >= 25.0 - 1e-6, "the twin lost its Z clearance: {twin_max:.3}");
    let twin_min = plain
        .path
        .moves
        .iter()
        .filter(|m| m.kind != MoveKind::Rapid && rho(m.to) < 6.4 && rho(m.to) > 0.1)
        .map(|m| rho(m.to))
        .fold(f64::INFINITY, f64::min);
    assert!(twin_min < 3.0, "the twin does not cut the lower flank to rho ~1: {twin_min:.3}");
}
