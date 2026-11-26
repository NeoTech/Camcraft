//! #1036: the milled Bore ride's way into an undrilled hole with a cutter
//! that is not centre-cutting. The Bore arm hands the tool's
//! `center_cutting` to the planar ladder (`lk.approach.center_cutting`,
//! M11.3 B-D, 874507a8) and every descent of that ladder asks the
//! centre-cutting law (515525c4): a plunge into the level's solid becomes
//! a helix or a ramp along the first cut, or the operation is refused by
//! name. These tests pin that the law covers the Bore ride: an undrilled
//! hole in solid stock is entered by a ramp along the radial sweep, never
//! an axial plunge into solid, and a hole too tight for any ramp or helix
//! is refused by name. A centre-cutting cutter keeps its axial plunge.
//!
//! Provenance: standard CNC entry practice (a cutter without centre
//! teeth ramps or helixes into solid, it never plunges) per public
//! machining handbooks; the laws are this project's (M11.3). No vendor
//! material consulted.
//!
//! Gate map (#1036): covers app::operation (generate, the Bore arm),
//! motion::machining2d (bore, descent), motion::plunge_law; and (#1560)
//! app::operation::hole_digest through app::project (save, load).

use flecs_ecs::prelude::World;

use camcraft_app::cl::{MoveKind, Toolpath};
use camcraft_app::commands::{AddTool, History, SetStock};
use camcraft_app::drilling::Hole;
use camcraft_app::geo2d::Point2;
use camcraft_app::operation::{self, Height, OpParams, OpStatus, SrcRef};
use camcraft_app::schema::{Location, Operation};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::{catalog, Tool};
use camcraft_app::world::new_world;
use camcraft_motion::approach::{CENTER_CUT_REFUSAL, NOT_CENTER_CUTTING};
use camcraft_motion::plunge_law::{first_solid_plunge, Solid};

/// A solid 60 x 60 x 20 block (nothing drilled) and an 8 mm end mill
/// twice: "CC" as catalogued, "NC" the same body not centre-cutting.
fn world() -> World {
    let world = new_world();
    let mut h = History::default();
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Location::default()).expect("stock"));
    let t = catalog().into_iter().find(|(n, _)| *n == "EM-0800-4FL").map(|(_, t)| t).expect("catalog entry");
    h.execute(&world, AddTool::new(&world, "CC", t.clone()).expect("tool"));
    h.execute(&world, AddTool::new(&world, "NC", Tool { center_cutting: false, ..t }).expect("tool"));
    world
}

fn bore(radius: f64) -> OpParams {
    OpParams::Bore {
        entry: camcraft_app::machining3d::Entry3D::Plunge,
        pass_start: camcraft_app::machining2d::PassStart::Auto,
        break_through: None,
        links: camcraft_app::links::Links::PLANAR,
        clearance_z: Height::fixed(25.0),
        retract_z: None,
        top_z: Height::fixed(20.0),
        bottom_z: None,
        stock_to_leave: 0.0,
        stepdown: 5.0,
        feed: 500.0,
        plunge_feed: 150.0,
        climb: true,
        tolerance: 0.01,
        holes: vec![Hole { xy: Point2::new(15.0, 15.0), radius, entry_z: 20.0, floor_z: 5.0, through: false, stepped: false }],
        picks: Default::default(),
    }
}

fn generate(world: &World, tool: &str, params: OpParams) -> Result<Toolpath, String> {
    let op = Operation {
        tool: tool.into(),
        src: SrcRef::Stock,
        params,
        stamp: None,
        setup: None,
        area: Default::default(),
        coolant: Default::default(),
        after: Default::default(),
        suppressed: false,
    };
    operation::generate(world, &op, "B").map(|(p, _)| p).map_err(|e| e.to_string())
}

/// The block's solid: everything under its top, no footprint (the
/// whole hole is undrilled stock).
const SOLID: Solid = Solid { top: 20.0, footprint: None };

#[test]
fn a_cutter_that_is_not_centre_cutting_ramps_into_an_undrilled_bore_never_plunges() {
    let world = world();
    let path = generate(&world, "NC", bore(10.0)).expect("the ramp stands in for the plunge");
    assert!(path.all_finite());
    // The after-the-fact audit finds no axial descent onto an uncut column.
    assert_eq!(first_solid_plunge(&path, 4.0, &SOLID), None, "an axial plunge into solid stock");
    assert!(
        path.notes.iter().any(|n| n.starts_with(NOT_CENTER_CUTTING) && n.contains("ramp along the first cut")),
        "the substitution is named: {:?}",
        path.notes
    );
    // Every Plunge move it does post lands on a column the ride already cut.
    let plunges = path.moves.iter().filter(|m| m.kind == MoveKind::Plunge).count();
    assert!(plunges > 0, "the ring re-entries are plunges into cut columns");
    // The ride still reaches the floor of the hole.
    let low = path.moves.iter().filter(|m| m.kind != MoveKind::Rapid).map(|m| m.to.z).fold(f64::INFINITY, f64::min);
    assert!((low - 5.0).abs() < 1e-6, "the ride bottoms at the floor: {low}");
}

#[test]
fn a_centre_cutting_cutter_keeps_its_axial_plunge() {
    let world = world();
    let path = generate(&world, "CC", bore(10.0)).expect("cc bore");
    // The same audit convicts the centre-cutting path - it plunges on the
    // hole's axis, which its centre teeth may - so the NC pass above is
    // the law's work, not a blind audit.
    assert!(first_solid_plunge(&path, 4.0, &SOLID).is_some(), "the centre-cutting ride dives on the axis");
    assert!(!path.notes.iter().any(|n| n.starts_with(NOT_CENTER_CUTTING)), "{:?}", path.notes);
}

#[test]
fn an_undrilled_hole_too_tight_for_a_ramp_or_helix_is_refused_by_name() {
    let world = world();
    // An 8 mm cutter in a 9 mm hole: the centre disk is 0.5 mm, no helix
    // fits and the radial sweep is too short to ramp the 5 mm rung.
    let msg = generate(&world, "NC", bore(4.5)).err().unwrap_or_default();
    assert!(msg.contains(CENTER_CUT_REFUSAL), "refused by the centre-cutting law: {msg}");
    // The centre-cutting cutter bores the same hole.
    assert!(generate(&world, "CC", bore(4.5)).is_ok());
}

/// #1560 through the commands: a Bore op whose hole list is edited to a
/// floor off the 1 um lattice (3.8099999999999996 mm, the file stores
/// 3.81) is generated Ok, saved, loaded into a fresh world, and still
/// reads Ok - the stamp digests the hole as the file keeps it.
#[test]
fn an_edited_hole_floor_off_the_lattice_stays_ok_through_save_and_load() {
    use camcraft_app::commands::{CreateOperation, EditOperationParams, GenerateOperation};
    use camcraft_app::{model, project};

    let world = world();
    let mut h = History::default();
    h.execute(&world, CreateOperation::new(&world, "B", "CC", SrcRef::Stock, bore(8.0)).expect("create"));
    let typed: f64 = 0.190 * 25.4 - 0.040 * 25.4;
    assert_ne!(typed.to_bits(), 3.81f64.to_bits(), "the probe must be off the lattice");
    let mut edited = bore(8.0);
    if let OpParams::Bore { holes, .. } = &mut edited {
        holes[0].floor_z = typed;
    }
    h.execute(&world, EditOperationParams::new(&world, "B", edited).expect("edit"));
    h.execute(&world, GenerateOperation::new(&world, "B").expect("generate"));
    let op = model::find_by_name(&world, "B").expect("op");
    assert_eq!(operation::status_of(&world, &op), OpStatus::Ok, "generated Ok");

    let path = std::env::temp_dir().join(format!("camcraft_f1560_{}.camproj", std::process::id()));
    project::save(&world, &path).expect("save");
    let world2 = new_world();
    let loaded = project::load(&world2, &path);
    let _ = std::fs::remove_file(&path);
    loaded.expect("load");
    let op2 = model::find_by_name(&world2, "B").expect("loaded op");
    assert_eq!(operation::status_of(&world2, &op2), OpStatus::Ok, "saved Ok, loaded Ok - not Stale");
}

/// The same bore with a chosen `entry` and both approach angles at `deg`
/// (the links' `ramp_angle` and `helix_angle`: the cap on every descent).
fn bore_entering(radius: f64, entry: camcraft_app::machining3d::Entry3D, deg: f64) -> OpParams {
    let mut p = bore(radius);
    if let OpParams::Bore { links, entry: e, stepdown, .. } = &mut p {
        *stepdown = 1.0;
        links.approach.ramp_angle = deg;
        links.approach.helix_angle = deg;
        *e = entry;
    }
    p
}

/// A 1/8 in (3.175 mm) end mill beside the 8 mm pair.
fn world_with_eighth_inch() -> World {
    let world = world();
    let mut h = History::default();
    let t = catalog().into_iter().find(|(n, _)| *n == "EM-0800-4FL").map(|(_, t)| t).expect("catalog entry");
    h.execute(&world, AddTool::new(&world, "EIGHTH", Tool { diameter: 3.175, ..t }).expect("tool"));
    world
}

/// The steepest feed descent (deg) of a path, and its axial feed descents.
fn steepest_and_axial(path: &Toolpath) -> (f64, usize) {
    let mut worst: f64 = 0.0;
    let mut axial = 0;
    for m in path.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        let dz = m.from.z - m.to.z;
        if dz <= 1e-9 {
            continue;
        }
        let xy = match m.arc {
            None => (m.to.x - m.from.x).hypot(m.to.y - m.from.y),
            Some(a) => {
                let ang = |p: camcraft_app::geometry::Point3| (p.y - a.center.y).atan2(p.x - a.center.x);
                let d = if a.ccw { ang(m.to) - ang(m.from) } else { ang(m.from) - ang(m.to) };
                (m.from.x - a.center.x).hypot(m.from.y - a.center.y) * d.rem_euclid(std::f64::consts::TAU)
            }
        };
        if xy <= 1e-9 {
            axial += 1;
        } else {
            worst = worst.max((dz / xy).atan().to_degrees());
        }
    }
    (worst, axial)
}

/// One Bore op, a 1/8 in cutter, a 3.2 mm hole (a ring 0.0125 mm in radius)
/// and a 6 mm one: entry ramp or helix with 5 deg posts no descent steeper
/// than 5 deg; the plunge default still dives. Axial moves left are the
/// re-plunge down the column each ring just cut, one a level, named.
#[test]
fn a_bore_entry_ramp_or_helix_caps_the_descent_on_a_hole_barely_wider_than_the_tool() {
    use camcraft_app::machining3d::Entry3D;
    let world = world_with_eighth_inch();
    for radius in [1.6, 1.75, 3.0] {
        let (_, plunge_axial) = steepest_and_axial(&generate(&world, "EIGHTH", bore_entering(radius, Entry3D::Plunge, 5.0)).expect("plunge bore"));
        assert!(plunge_axial > 0, "R{radius}: the default entry plunges");
        for entry in [Entry3D::Ramp, Entry3D::Helix] {
            let path = generate(&world, "EIGHTH", bore_entering(radius, entry, 5.0)).expect("entry bore");
            let (worst, axial) = steepest_and_axial(&path);
            assert!(worst <= 5.0 + 1e-3, "R{radius} {entry:?}: a {worst:.3} deg descent");
            // 15 mm deep at 1 mm a level: the one axial move of each level is the
            // re-plunge down the column its ring just cut, and the note names it.
            assert!(axial <= 15, "R{radius} {entry:?}: {axial} axial descents: {:?}", path.notes);
            assert!(path.notes.iter().all(|n| !n.contains("axial dive")), "R{radius} {entry:?}: an axial dive on the floor: {:?}", path.notes);
        }
    }
}

/// A bore has no arc entry: refused by name when the op is generated.
#[test]
fn a_bore_refuses_the_arc_entry_by_name() {
    let world = world_with_eighth_inch();
    let err = generate(&world, "EIGHTH", bore_entering(3.0, camcraft_app::machining3d::Entry3D::Arc, 5.0)).err().unwrap_or_default();
    assert!(err.contains("plunge, ramp or helix"), "{err}");
}

/// The command doors refuse the arc entry at creation and at edit (validate
/// early), by the same name as the generator; a refused edit leaves the
/// stored entry as it was.
#[test]
fn the_command_doors_refuse_a_bore_arc_entry_by_name() {
    use camcraft_app::commands::{CreateOperation, EditOperationParams};
    use camcraft_app::machining3d::Entry3D;
    use camcraft_app::model;
    let world = world();
    let said = |r: Result<_, camcraft_app::commands::CommandError>| match r {
        Ok(_) => String::new(),
        Err(e) => e.to_string(),
    };
    let created = said(CreateOperation::new(&world, "B", "CC", SrcRef::Stock, bore_entering(8.0, Entry3D::Arc, 4.0)).map(|_| ()));
    assert!(created.contains("plunge, ramp or helix"), "create: {created}");
    assert!(model::find_by_name(&world, "B").is_none(), "a refused create adds nothing");

    let mut h = History::default();
    h.execute(&world, CreateOperation::new(&world, "B", "CC", SrcRef::Stock, bore_entering(8.0, Entry3D::Helix, 4.0)).expect("helix is offered"));
    h.execute(&world, CreateOperation::new(&world, "R", "CC", SrcRef::Stock, bore_entering(8.0, Entry3D::Ramp, 4.0)).expect("ramp is offered"));
    let edited = said(EditOperationParams::new(&world, "B", bore_entering(8.0, Entry3D::Arc, 4.0)).map(|_| ()));
    assert!(edited.contains("plunge, ramp or helix"), "edit: {edited}");
    let op = model::find_by_name(&world, "B").and_then(|e| model::operation_of(&world, &e)).expect("op");
    assert!(matches!(op.params, OpParams::Bore { entry: Entry3D::Helix, .. }), "the stored entry is unchanged: {:?}", op.params);
}

/// The entry is file data: a Helix bore saves and loads as a Helix bore, and
/// a Plunge bore writes no `entry` word, so every earlier file reads as before.
#[test]
fn a_bore_entry_round_trips_and_the_default_is_not_written() {
    use camcraft_app::commands::CreateOperation;
    use camcraft_app::{model, project};
    let write = |entry| {
        let world = world();
        let mut h = History::default();
        h.execute(&world, CreateOperation::new(&world, "B", "CC", SrcRef::Stock, bore_entering(8.0, entry, 4.0)).expect("create"));
        let path = std::env::temp_dir().join(format!("camcraft_bore_entry_{}_{:?}.camproj", std::process::id(), entry));
        project::save(&world, &path).expect("save");
        let text = std::fs::read_to_string(&path).expect("read");
        let world2 = new_world();
        let loaded = project::load(&world2, &path);
        let _ = std::fs::remove_file(&path);
        loaded.expect("load");
        let e = model::find_by_name(&world2, "B").expect("op");
        (text, model::operation_of(&world2, &e).map(|o| o.params))
    };
    use camcraft_app::machining3d::Entry3D;
    let (helix_text, helix) = write(Entry3D::Helix);
    assert!(helix_text.contains("\"entry\": \"helix\"") || helix_text.contains("\"entry\":\"helix\""), "the helix word is written");
    assert!(matches!(helix, Some(OpParams::Bore { entry: Entry3D::Helix, .. })), "{helix:?}");
    let (plunge_text, plunge) = write(Entry3D::Plunge);
    assert!(!plunge_text.contains("\"entry\""), "the default is not written");
    assert!(matches!(plunge, Some(OpParams::Bore { entry: Entry3D::Plunge, .. })));
}
