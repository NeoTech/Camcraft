//! #2218: an external NC program (parsed by camcraft-gcode) simulated
//! against the setup's stock, the target model, fixtures and a `.mch`
//! machine, with each finding on its 1-based NC line; unmapped tools and
//! a missing stock refused by name; a playback seed that plays.
//!
//! Fixture: the suite's 60x60x20 box stock at the origin (XY -30..30,
//! Z 0..20), a model top at Z 18, a 4 mm end mill as T1, and the WCS on
//! the world origin unless a test moves it.

use flecs_ecs::prelude::World;

use camcraft_app::cl::{ArcPlane, Move, MoveKind};
use camcraft_app::commands::{AddTool, History, ImportModel, SetStock, SetWcsOrigin};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{IDENTITY_Q, Point3, cube};
use camcraft_app::machine::{AxisDirection, AxisLimits, Machine};
use camcraft_app::nc_sim::{
    frame_for, play_seed, simulate_nc, simulate_nc_text, unmapped_tools, NcFindingKind, NcSimError, NcSimOptions,
    NcToolSource,
};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::stock_play::FieldPlay;
use camcraft_app::tool_library::LibraryTool;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use camcraft_gcode::{parse_with, ParseOptions};
use camcraft_sim::fixture::FixtureBox;

const TOOL: &str = "EM-0400-2FL";

fn tool(name: &str) -> camcraft_app::tools::Tool {
    catalog().into_iter().find(|(n, _)| *n == name).map(|(_, t)| t).expect("catalog entry")
}

fn world_with(stock: bool, wcs: Point3) -> World {
    let world = new_world();
    let mut h = History::default();
    if stock {
        let loc = Location { pos: Point3::new(0.0, 0.0, 0.0), rot: IDENTITY_Q };
        h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, loc).expect("stock"));
    }
    h.execute(&world, AddTool::new(&world, TOOL, tool(TOOL)).expect("tool"));
    assert_eq!(camcraft_app::tool_numbers::number_of(&world, TOOL), Some(1), "the first tool is T1");
    let part = cube(Point3::new(-30.0, -30.0, 0.0), Point3::new(30.0, 30.0, 18.0));
    h.execute(&world, ImportModel::from_mesh("part", part));
    h.execute(&world, SetWcsOrigin::new(&world, Location { pos: wcs, rot: IDENTITY_Q }).expect("wcs"));
    world
}

fn world() -> World {
    world_with(true, Point3::new(0.0, 0.0, 0.0))
}

/// A face program at `z` in rows 3 mm apart; `gouge` puts one feed row 1 mm
/// lower. Returns the text and the 1-based line of the lowered row.
fn face(tool: u32, z: f64, shift: [f64; 3], gouge: bool) -> (String, usize) {
    let mut l = vec![
        "%".to_string(),
        "O0100 (FACE)".into(),
        "G21 G17 G90 G94 G54".into(),
        format!("T{tool} M6"),
        "S8000 M3".into(),
        format!("G0 X{:.3} Y{:.3} Z{:.3}", -34.0 + shift[0], -30.0 + shift[1], 25.0 + shift[2]),
    ];
    let mut low = 0;
    for (k, y) in (0..=20).map(|k| (k, -30.0 + 3.0 * f64::from(k))) {
        let x = if k % 2 == 0 { 34.0 } else { -34.0 };
        let zz = if gouge && k == 10 { z - 1.0 } else { z };
        l.push(format!("G1 Y{:.3} Z{:.3} F600", y + shift[1], zz + shift[2]));
        l.push(format!("G1 X{:.3}", x + shift[0]));
        if gouge && k == 10 {
            low = l.len();
        }
    }
    l.push(format!("G0 Z{:.3}", 25.0 + shift[2]));
    l.extend(["M5".into(), "M30".into(), "%".into()]);
    (l.join("\n"), low)
}

fn run(world: &World, text: &str, opts: &NcSimOptions) -> Result<camcraft_app::nc_sim::NcSimReport, NcSimError> {
    simulate_nc_text(world, text, &ParseOptions::default(), opts)
}

#[test]
fn nc_face_pass_cuts_the_stock_to_the_model_without_findings() {
    let world = world();
    let (text, _) = face(1, 18.0, [0.0; 3], false);
    let r = run(&world, &text, &NcSimOptions::default()).expect("simulates");
    assert_eq!(r.sections.len(), 1);
    assert_eq!((r.sections[0].tool_number, r.sections[0].tool.as_str()), (1, TOOL));
    let dev = r.deviation.as_ref().expect("the part is the target");
    assert!(dev.max_gouge <= r.tol, "no gouge: {}", dev.max_gouge);
    assert!(dev.max_rest <= r.tol + 1e-9 || dev.rest_cells > 0, "the deviation is measured");
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert!(r.notes.iter().any(|n| n.contains("no machine")), "the unjudged machine is said: {:?}", r.notes);
}

#[test]
fn nc_gouge_is_found_on_its_nc_line() {
    let world = world();
    let (text, low) = face(1, 18.0, [0.0; 3], true);
    let r = run(&world, &text, &NcSimOptions::default()).expect("simulates");
    assert!(r.deviation.as_ref().is_some_and(|d| d.max_gouge > 0.9), "the cut stock shows the 1 mm gouge");
    let gouges: Vec<usize> = r.findings.iter().filter(|f| f.kind == NcFindingKind::Gouge).map(|f| f.line).collect();
    assert!(gouges.contains(&low), "the gouge names line {low}: {:?}", r.findings);
    assert!(!r.is_clean());
}

#[test]
fn nc_program_reads_the_setup_wcs_origin() {
    // The WCS at the stock's top corner: program X0 Y0 Z0 is world (-30, -30, 20).
    let world = world_with(true, Point3::new(-30.0, -30.0, 20.0));
    let (text, _) = face(1, 18.0, [30.0, 30.0, -20.0], false);
    let r = run(&world, &text, &NcSimOptions::default()).expect("simulates");
    assert!(r.deviation.as_ref().is_some_and(|d| d.max_gouge <= r.tol), "same cut as the world-frame program");
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn nc_unmapped_tool_is_refused_by_number() {
    let world = world();
    let (text, _) = face(7, 18.0, [0.0; 3], false);
    let err = run(&world, &text, &NcSimOptions::default()).expect_err("T7 has no geometry");
    match &err {
        NcSimError::UnmappedTools { numbers, reasons } => {
            assert_eq!(numbers, &vec![7]);
            assert!(reasons.iter().any(|r| r.starts_with("T7:")), "each unmapped number says why: {reasons:?}");
        }
        other => panic!("wrong refusal: {other}"),
    }
    assert!(err.to_string().contains("T7"), "{err}");
    let parsed = parse_with(&text, &ParseOptions::default()).expect("parses");
    let map = unmapped_tools(&world, &parsed, &NcSimOptions::default()).expect("maps");
    assert_eq!(map.unmapped, vec![7]);
}

#[test]
fn nc_tools_come_from_the_map_and_the_library() {
    let world = world();
    let mut row = LibraryTool::bare("LIB-4", tool(TOOL));
    row.number = Some(9);
    let opts = NcSimOptions {
        tool_map: vec![(7, "LIB-4".into())],
        library: vec![row],
        ..Default::default()
    };
    let (text, _) = face(7, 18.0, [0.0; 3], false);
    let parsed = parse_with(&text, &ParseOptions::default()).expect("parses");
    let map = unmapped_tools(&world, &parsed, &opts).expect("maps");
    assert_eq!(map.get(7).map(|t| t.source), Some(NcToolSource::Explicit));
    assert!(simulate_nc(&world, &parsed, &opts).is_ok_and(|r| r.sections[0].tool == "LIB-4"));
    let (text, _) = face(9, 18.0, [0.0; 3], false);
    let parsed = parse_with(&text, &ParseOptions::default()).expect("parses");
    let map = unmapped_tools(&world, &parsed, &opts).expect("maps");
    assert_eq!(map.get(9).map(|t| t.source), Some(NcToolSource::Library));
    let probe = LibraryTool::bare("PROBE", tool("PRB-0400"));
    let opts = NcSimOptions { tool_map: vec![(9, "PROBE".into())], library: vec![probe], ..Default::default() };
    let err = simulate_nc(&world, &parsed, &opts).expect_err("a probe is no cutter");
    assert!(matches!(err, NcSimError::UnmappedTools { .. }), "{err}");
}

#[test]
fn nc_missing_stock_is_refused_by_name() {
    let world = world_with(false, Point3::new(0.0, 0.0, 0.0));
    let (text, _) = face(1, 18.0, [0.0; 3], false);
    let err = run(&world, &text, &NcSimOptions::default()).expect_err("no stock");
    assert!(err.to_string().contains("no stock"), "{err}");
}

#[test]
fn nc_fixture_strike_is_on_its_nc_line() {
    let world = world();
    let text = "G21 G90 G54\nT1 M6\nS8000 M3\nG0 X45 Y0 Z40\nG0 Z10\nG0 Z40\nM30";
    let clamp = FixtureBox { min: Point3::new(40.0, -5.0, 0.0), max: Point3::new(50.0, 5.0, 25.0) };
    let opts = NcSimOptions { fixtures: Some(vec![clamp]), ..Default::default() };
    let r = run(&world, text, &opts).expect("simulates");
    let lines: Vec<usize> = r.findings.iter().filter(|f| f.kind == NcFindingKind::Fixture).map(|f| f.line).collect();
    assert!(lines.contains(&5), "the rapid down to Z10 on line 5 strikes the clamp: {:?}", r.findings);
    assert!(!lines.contains(&4), "line 4 stays above the clamp: {:?}", r.findings);
}

fn small_mill() -> Machine {
    let axis = |name: &str, travel| AxisLimits {
        name: name.into(),
        feed: Some(500.0),
        rapid: Some(10000.0),
        travel: Some(travel),
        ..Default::default()
    };
    Machine {
        model: "NC test mill".into(),
        spindle_min: Some(100.0),
        spindle_max: Some(6000.0),
        axes: vec![axis("X", (-20.0, 20.0)), axis("Y", (-100.0, 100.0)), axis("Z", (-100.0, 100.0))],
        ..Default::default()
    }
}

#[test]
fn nc_machine_travel_feed_and_spindle_findings_have_lines() {
    let world = world();
    let (text, _) = face(1, 18.0, [0.0; 3], false);
    let opts = NcSimOptions { machine: Some(small_mill()), ..Default::default() };
    let r = run(&world, &text, &opts).expect("simulates");
    let machine: Vec<_> = r.findings.iter().filter(|f| f.kind == NcFindingKind::Machine).collect();
    // Line 4 is `T1 M6`; the section's first move is line 6.
    assert!(machine.iter().any(|f| f.line == 6 && f.message.contains("8000 rpm")), "{machine:?}");
    assert!(machine.iter().any(|f| f.line == 6 && f.message.contains("stroke")), "X-34 is past -20: {machine:?}");
    assert!(machine.iter().any(|f| f.line == 7 && f.message.contains("600 mm/min")), "{machine:?}");
    assert!(machine.iter().all(|f| f.message.starts_with(&format!("line {}:", f.line))), "{machine:?}");
}

#[test]
fn nc_work_offsets_map_by_origin_and_sign() {
    let world = world();
    let text = "G21 G90 G55\nT1 M6\nG0 X0 Y0 Z30\nM30";
    let err = run(&world, text, &NcSimOptions::default()).expect_err("G55 has no origin");
    assert!(err.to_string().contains("G55"), "{err}");

    let mut opts = NcSimOptions::default();
    opts.wcs_origins[1] = Some([10.0, 0.0, 0.0]);
    let mut m = small_mill();
    m.axes[0].direction = AxisDirection::Inverted;
    let f = frame_for(&world, &opts, Some(&m), 1).expect("G55 frame");
    assert_eq!(f.sign, [-1.0, 1.0, 1.0]);
    let arc = Move::new(MoveKind::Feed, Point3::new(5.0, 0.0, 1.0), Point3::new(0.0, 5.0, 1.0), Some(300.0))
        .with_arc_in(Point2::new(0.0, 0.0), true, ArcPlane::Xy);
    let w = f.move_to_world(&arc).expect("maps");
    assert_eq!((w.from.x, w.to.x, w.to.y), (5.0, 10.0, 5.0));
    let a = w.arc.expect("still an arc");
    assert_eq!((a.center.x, a.center.y, a.ccw), (10.0, 0.0, false), "one negated axis reverses G3 to G2");
}

#[test]
fn nc_play_seed_builds_a_playback_that_plays_to_the_end() {
    let world = world();
    let (text, _) = face(1, 18.0, [0.0; 3], false);
    let parsed = parse_with(&text, &ParseOptions::default()).expect("parses");
    let seed = play_seed(&world, &parsed, &NcSimOptions::default(), 20_000.0).expect("seed");
    let total = seed.index().total();
    assert_eq!(total, parsed.sections.iter().map(|s| s.toolpath.moves.len()).sum::<usize>());
    let mut play = FieldPlay::from_seed(seed).expect("plays");
    assert!(play.seek(total).is_ok(), "the playback seeks to its last move");
}
