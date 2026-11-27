//! Hurco WinMax (NC) built-in post and Hurco VM10i machine tests (#2573).
//!
//! Provenance: clean-room specification from Hurco WinMax Mill NC
//! Programming documentation (704-0116-101, support.hurco.com ISNC) and
//! public Hurco VM10i specifications.
//!
//! Gate map: covers app::posting_files (BUILTINS, MACHINES, embedded),
//! app::posting (post_serves, post_setup), assets/posts/hurco-winmax.cps,
//! assets/machines/hurco-vm10i.mch.

use camcraft_app::commands::{
    AddTool, CreateOperation, CreateSetup, GenerateOperation, History, SetSetupMachine,
    SetSetupPost, SetSetupWorkOffset, SetStock,
};
use camcraft_app::drilling::{CycleChoice, Hole};
use camcraft_app::geo2d::Point2;
use camcraft_app::links::Links;
use camcraft_app::machine::{kind, parse_mch, Kind};
use camcraft_app::machining2d::{Lead, PocketStrategy, Side};
use camcraft_app::machining3d::Entry3D;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::posting::{
    self, builtin_machine_text, builtin_post_text, post_serves, BUILTINS, MACHINES,
};
use camcraft_app::schema::{Location, PostRef};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::{catalog, Tool};
use camcraft_app::world::new_world;

fn cat_tool(name: &str) -> Tool {
    catalog()
        .into_iter()
        .find(|(n, _)| *n == name)
        .map(|(_, t)| t)
        .unwrap_or_else(|| panic!("missing catalog tool {name}"))
}

#[test]
fn hurco_winmax_post_and_vm10i_are_registered_and_serve_mill() {
    assert!(
        BUILTINS.iter().any(|(n, f)| *n == "Hurco WinMax (NC)" && f.0 == "hurco-winmax.cps"),
        "Hurco WinMax (NC) must be registered in BUILTINS"
    );
    assert!(
        MACHINES.iter().any(|(n, f)| *n == "Hurco VM10i" && f.0 == "hurco-vm10i.mch"),
        "Hurco VM10i must be registered in MACHINES"
    );
    let cps = builtin_post_text("Hurco WinMax (NC)").expect("hurco-winmax.cps loads");
    assert!(cps.contains("Hurco WinMax"), "unexpected post text");

    let mch_text = builtin_machine_text("Hurco VM10i").expect("hurco-vm10i.mch loads");
    let mch = parse_mch(mch_text).expect("hurco-vm10i.mch parses cleanly");
    assert_eq!(kind(Some(&mch)), Kind::Mill);
    assert!(mch.axes.iter().any(|a| a.name == "X"), "VM10i has X axis");
    assert!(mch.axes.iter().any(|a| a.name == "Y"), "VM10i has Y axis");
    assert!(mch.axes.iter().any(|a| a.name == "Z"), "VM10i has Z axis");

    assert!(post_serves("Hurco WinMax (NC)", Kind::Mill));
    assert!(post_serves("Hurco WinMax (NC)", Kind::MillTurn));
    assert!(!post_serves("Hurco WinMax (NC)", Kind::Lathe));
    assert!(!post_serves("Hurco WinMax (NC)", Kind::Plasma));
    assert!(!post_serves("Hurco WinMax (NC)", Kind::WireEdm));
    assert!(!post_serves("Hurco WinMax (NC)", Kind::SinkerEdm));
}

#[test]
fn posts_full_app_workflow_contour_pocket_drill_peck_tap_two_tools_two_offsets() {
    let world = new_world();
    let mut h = History::default();
    let stock = StockShape::Box { dx: 80.0, dy: 80.0, dz: 20.0 };
    h.execute(&world, SetStock::new(&world, stock, Location::default()).expect("stock"));
    h.execute(&world, AddTool::new(&world, "EM-0600-4FL", cat_tool("EM-0600-4FL")).expect("em"));
    h.execute(&world, AddTool::new(&world, "DR-0600", cat_tool("DR-0600")).expect("drill"));
    h.execute(&world, AddTool::new(&world, "TAP-M6", cat_tool("TAP-M6")).expect("tap"));

    h.execute(&world, CreateSetup::new(&world, None, None).expect("Setup1"));
    let vm10i = parse_mch(builtin_machine_text("Hurco VM10i").expect("vm10i")).expect("parse vm10i");
    h.execute(
        &world,
        SetSetupMachine::new(&world, "Setup1", Some(("hurco-vm10i.mch".into(), Some(vm10i))))
            .expect("mount vm10i"),
    );
    h.execute(
        &world,
        SetSetupPost::new(&world, "Setup1", Some(PostRef::Named("Hurco WinMax (NC)".into())))
            .expect("select hurco post"),
    );
    h.execute(&world, SetSetupWorkOffset::new(&world, "Setup1", Some(0)).expect("G54"));

    let contour = OpParams::Contour {
        stock_to_leave: 0.0,
        clearance_z: Height::fixed(25.0),
        retract_z: Height::fixed(22.0),
        top_z: Height::fixed(20.0),
        bottom_z: Height::fixed(16.0),
        stepdown: 4.0,
        feed: 900.0,
        plunge_feed: 300.0,
        climb: true,
        side: Side::Outside,
        lead: Lead::None,
        tolerance: 0.01,
        tab_count: 0,
        tab_width: 4.0,
        tab_height: 2.0,
        entry: Entry3D::Plunge,
        helix_radius: 2.0,
        blend_turns: false,
        links: Links::PLANAR,
        tilt: None,
        blend_radius: None,
        chain_overrides: Vec::new(),
        picks: Default::default(),
    };
    let pocket = OpParams::Pocket {
        stepover: 3.0,
        stock_to_leave: 0.0,
        clearance_z: Height::fixed(25.0),
        retract_z: Some(Height::fixed(22.0)),
        top_z: Height::fixed(20.0),
        bottom_z: Height::fixed(17.0),
        stepdown: 3.0,
        feed: 1000.0,
        plunge_feed: 300.0,
        climb: true,
        strategy: PocketStrategy::OffsetRings,
        entry: Entry3D::Plunge,
        helix_radius: 1.5,
        links: Links::PLANAR,
        optimal_load: 0.4,
        blend_turns: false,
        blend_radius: None,
        rest: None,
        tilt: None,
        corner_relief: Default::default(),
        pass_start: Default::default(),
        picks: Default::default(),
    };
    let drill = OpParams::Drill {
        clearance_z: Height::fixed(25.0),
        retract_z: Some(Height::fixed(22.0)),
        top_z: Height::fixed(20.0),
        bottom_z: Some(Height::fixed(12.0)),
        cycle: CycleChoice::Drill,
        peck: 0.0,
        dwell: 0.0,
        feed: 100.0,
        plunge_feed: 100.0,
        holes: vec![Hole {
            xy: Point2::new(15.0, 15.0),
            radius: 3.0,
            entry_z: 20.0,
            floor_z: 12.0,
            through: false,
            stepped: false,
        }],
        cross: Vec::new(),
        break_through: None,
        break_through_from: Default::default(),
        picks: Default::default(),
    };
    let peck = OpParams::Drill {
        clearance_z: Height::fixed(25.0),
        retract_z: Some(Height::fixed(22.0)),
        top_z: Height::fixed(20.0),
        bottom_z: Some(Height::fixed(6.0)),
        cycle: CycleChoice::Peck,
        peck: 3.0,
        dwell: 0.0,
        feed: 100.0,
        plunge_feed: 100.0,
        holes: vec![Hole {
            xy: Point2::new(30.0, 15.0),
            radius: 3.0,
            entry_z: 20.0,
            floor_z: 6.0,
            through: false,
            stepped: false,
        }],
        cross: Vec::new(),
        break_through: None,
        break_through_from: Default::default(),
        picks: Default::default(),
    };

    for (name, tool, params) in [
        ("Contour1", "EM-0600-4FL", contour),
        ("Pocket1", "EM-0600-4FL", pocket),
        ("Drill1", "DR-0600", drill),
        ("Peck1", "DR-0600", peck),
    ] {
        h.execute(&world, CreateOperation::new(&world, name, tool, SrcRef::Stock, params).expect(name));
        h.execute(&world, GenerateOperation::new(&world, name).expect("gen"));
    }

    // Second work offset in Setup2 (same machine + post mount).
    h.execute(&world, CreateSetup::new(&world, Some("Setup2"), Some("Setup1")).expect("Setup2"));
    h.execute(&world, SetSetupWorkOffset::new(&world, "Setup2", Some(1)).expect("G55"));
    let tap = OpParams::Drill {
        clearance_z: Height::fixed(25.0),
        retract_z: Some(Height::fixed(22.0)),
        top_z: Height::fixed(20.0),
        bottom_z: Some(Height::fixed(10.0)),
        cycle: CycleChoice::Tap,
        peck: 0.0,
        dwell: 0.0,
        feed: 500.0,
        plunge_feed: 500.0,
        holes: vec![Hole {
            xy: Point2::new(30.0, 15.0),
            radius: 3.0,
            entry_z: 20.0,
            floor_z: 10.0,
            through: false,
            stepped: false,
        }],
        cross: Vec::new(),
        break_through: None,
        break_through_from: Default::default(),
        picks: Default::default(),
    };
    h.execute(&world, CreateOperation::new(&world, "Tap1", "TAP-M6", SrcRef::Stock, tap).expect("Tap1"));
    h.execute(&world, GenerateOperation::new(&world, "Tap1").expect("gen tap"));

    let mut out_path = std::env::temp_dir();
    out_path.push(format!("camcraft-2573-vm10i-{}.nc", std::process::id()));
    let posted = posting::post_setup(&world, &out_path).expect("app posts whole Hurco VM10i program");
    let _ = std::fs::remove_file(&out_path);
    assert!(posted.ops >= 5, "expected all ops posted, got {}", posted.ops);

    let p = &posted.program;
    assert!(p.contains("T1 M6\n") && p.contains("T2 M6\n") && p.contains("T3 M6\n"), "tools:\n{p}");
    assert!(p.contains("G54") && p.contains("G55"), "two work offsets G54 & G55:\n{p}");
    assert!(p.contains("G81 ") && (p.contains("G83 ") || p.contains("G73 ")) && p.contains("G84.2 "), "drill/peck/tap:\n{p}");
    assert!(!p.contains("G91.1"), "no G91.1:\n{p}");

    // Also confirm extended auxiliary offset G54.1 P1 (slot index 6).
    h.execute(&world, SetSetupWorkOffset::new(&world, "Setup1", Some(6)).expect("G54.1 P1"));
    let ext = posting::post_setup_unwritten(&world).expect("posts with G54.1 P1");
    assert!(ext.program.contains("G54.1 P1"), "extended WCS G54.1 P1:\n{}", ext.program);
}
