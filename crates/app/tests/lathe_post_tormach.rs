//! The Tormach PathPilot (LinuxCNC) LATHE post (#612):
//! `assets/posts/tormach-lathe.cps` posts the turning documents' own
//! streams - profile rough and finish, threading (G33 passes or the
//! one-block LinuxCNC G76), the axial drill as plain moves - to
//! byte-identical G-code in inch and mm, and refuses by name what the
//! dialect cannot say (a tap, a back-bore, a hole off the axis, a G76
//! record it cannot state exactly, a tool number outside T01-T99,
//! control comp, a C-axis section). Every program also passes a
//! LinuxCNC syntax sanity check: G7 stated, no G32 / G80 / G81 / G83 /
//! M6 / G43, no O word or % fence, no nested comment, every word from
//! the sets the LinuxCNC reference and Tormach's lathe tables list.
//!
//! Regenerate after an intentional change:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-app --test lathe_post_tormach`
//!
//! Gate map (M4.4): covers app::posting (the registry, stream_records,
//! post_setup on the Tormach lathe post), app::post (engine on the
//! PathPilot lathe dialect), app::commands (SelectPost / SetPostOptions).
//! Lane shape (M4.4): all tests here run in seconds.

mod common;

use flecs_ecs::prelude::World;
use serde_json::json;

use camcraft_app::cl::{CycleKind, MoveKind};
use camcraft_app::commands::{
    AddTool, CreateOperation, GenerateOperation, History, ImportModel, LoadMachine, SelectPost, SetPostOptions,
    SetStock,
};
use camcraft_app::drilling::{CycleChoice, Hole};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machine::{Coolant, Kind, Machine};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::post::{cnc, cnc_thread, engine};
use camcraft_app::posting;
use camcraft_app::schema::{Location, PostOptions, PostRef, ThreadingOutput};
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

const TAU: f64 = std::f64::consts::TAU;
const CPS: &str = include_str!("../../../assets/posts/tormach-lathe.cps");
const NAME: &str = "Tormach PathPilot Lathe";

// ------------------------------------------------------------ fixtures

fn revolve(meridian: &[(f64, f64)], segs: usize) -> Mesh {
    let v = |j: usize, r: f64, z: f64| {
        let th = TAU * j as f64 / segs as f64;
        Point3::new(r * th.cos(), r * th.sin(), z)
    };
    let mut facets = Vec::new();
    for seg in meridian.windows(2) {
        let ((ra, za), (rb, zb)) = (seg[0], seg[1]);
        for j in 0..segs {
            let (a1, a2, b1, b2) = (v(j, ra, za), v(j + 1, ra, za), v(j, rb, zb), v(j + 1, rb, zb));
            if ra <= 1e-12 {
                facets.push(Triangle::new(a1, b2, b1));
            } else if rb <= 1e-12 {
                facets.push(Triangle::new(a1, a2, b2));
            } else {
                facets.push(Triangle::new(a1, a2, b2));
                facets.push(Triangle::new(a1, b2, b1));
            }
        }
    }
    Mesh::from_facets(facets)
}

/// A PathPilot-style lathe: a stated ceiling (G96's D cap), flood M8,
/// in the unit asked.
fn lathe(inch: bool) -> Machine {
    Machine {
        model: "TestBench PathPilot lathe".into(),
        vendor: "Camcraft Testbench".into(),
        kind: Some("lathe".into()),
        units: inch.then(|| "inch".into()),
        spindle_min: Some(25.0),
        spindle_max: Some(12000.0),
        coolants: vec![Coolant { name: "Flood".into(), m_code: "M8".into() }],
        ..Default::default()
    }
}

fn world_with(bar: Mesh, stock: camcraft_app::stock::StockShape, z0: f64, tools: &[&str], inch: bool) -> (World, History) {
    let world = new_world();
    let mut history = History::default();
    let at = Location { pos: Point3::new(0.0, 0.0, z0), ..Location::default() };
    history.execute(&world, SetStock::new(&world, stock, at).expect("stock"));
    history.execute(&world, LoadMachine::new(&world, "lathe.mch".into(), Some(lathe(inch))).expect("lathe"));
    for t in tools {
        let tool = catalog().into_iter().find(|(n, _)| n == t).map(|(_, t)| t).expect("catalog entry");
        history.execute(&world, AddTool::new(&world, t, tool).expect("tool"));
    }
    history.execute(&world, ImportModel::from_mesh("bar", bar));
    history.execute(&world, SelectPost::new(&world, PostRef::Named(NAME.into())).expect("a registry name"));
    (world, history)
}

fn create(world: &World, history: &mut History, name: &str, tool: &str, params: OpParams) {
    let cmd = CreateOperation::new(world, name, tool, SrcRef::Entity("bar".into()), params);
    history.execute(world, cmd.expect("create"));
    history.execute(world, GenerateOperation::new(world, name).expect("generate"));
    let e = camcraft_app::model::find_by_name(world, name).expect("op");
    assert_eq!(
        camcraft_app::operation::status_of(world, &e),
        camcraft_app::operation::OpStatus::Ok,
        "{name} generates"
    );
}

/// The flanged bar's rough and finish (the Fanuc lathe suite's stage B).
fn turning_world(inch: bool) -> (World, History) {
    let flanged = [(0.0, -16.0), (6.0, -16.0), (6.0, 2.0), (3.5, 2.0), (3.5, 16.0), (0.0, 16.0)];
    let stock = camcraft_app::stock::StockShape::Cylinder { diameter: 12.0, height: 32.0 };
    let (world, mut history) = world_with(revolve(&flanged, 16), stock, -16.0, &["TN-1600-0R8"], inch);
    let rough = OpParams::ProfileRough {
        tolerance: 0.05,
        clearance_z: Height::fixed(25.0),
        top_z: Height::fixed(16.0),
        bottom_z: Height::fixed(-16.0),
        stepdown: 0.8,
        stock_to_leave: 0.25,
        feed: 150.0,
        plunge_feed: 80.0,
    };
    let finish = OpParams::ProfileFinish {
        tolerance: 0.05,
        clearance_z: Height::fixed(25.0),
        top_z: Height::fixed(16.0),
        bottom_z: Height::fixed(2.0),
        feed: 120.0,
        plunge_feed: 60.0,
    };
    create(&world, &mut history, "R1", "TN-1600-0R8", rough);
    create(&world, &mut history, "S1", "TN-1600-0R8", finish);
    (world, history)
}

const MAJOR: f64 = 6.0;
const PITCH: f64 = 2.0;
const DEPTH: f64 = 1.2;

/// The threading suite's bar: wall r 6, a 60-degree V every 2.0 from
/// z 8 to z 14, 1.2 deep.
fn threaded() -> Mesh {
    let half = DEPTH * (30.0_f64).to_radians().tan();
    let root = MAJOR - DEPTH;
    let mut m = vec![(0.0, -16.0), (MAJOR, -16.0), (MAJOR, 8.0 - half)];
    let mut c = 8.0;
    while c <= 14.0 + 1e-9 {
        m.push((root, c));
        m.push((MAJOR, c + half));
        if c + PITCH <= 14.0 + 1e-9 {
            m.push((MAJOR, c + PITCH - half));
        }
        c += PITCH;
    }
    m.push((MAJOR, 14.0 + half));
    m.push((MAJOR, 40.0));
    m.push((0.0, 40.0));
    revolve(&m, 16)
}

fn thread_world(inch: bool) -> (World, History) {
    let stock = camcraft_app::stock::StockShape::Cylinder { diameter: 12.0, height: 56.0 };
    let (world, mut history) = world_with(threaded(), stock, -16.0, &["TD-1732-60"], inch);
    let params = OpParams::Threading {
        tolerance: 0.05,
        clearance_z: Height::fixed(24.0),
        top_z: Height::fixed(14.3),
        bottom_z: Height::fixed(7.7),
        pitch: PITCH,
        depth: DEPTH,
        infeed: 0.4,
        feed: 120.0,
        plunge_feed: 40.0,
    };
    create(&world, &mut history, "TH1", "TD-1732-60", params);
    (world, history)
}

/// The axial drill fixture of the Fanuc lathe suite: a Ø4.1 hole 10 deep
/// on centre.
fn drill_world(inch: bool) -> (World, History) {
    let bar = revolve(&[(0.0, -20.0), (10.0, -20.0), (10.0, 20.0), (2.05, 20.0), (2.05, 10.0), (0.0, 10.0)], 64);
    let stock = camcraft_app::stock::StockShape::Box { dx: 24.0, dy: 24.0, dz: 40.0 };
    let (world, mut history) = world_with(bar, stock, -20.0, &["DR-0400"], inch);
    let hole = Hole { xy: Point2::new(0.0, 0.0), radius: 2.05, entry_z: 20.0, floor_z: 10.0, through: false, stepped: false };
    let params = OpParams::Drill {
        clearance_z: Height::fixed(30.0),
        retract_z: None,
        top_z: Height::fixed(20.0),
        bottom_z: None,
        cycle: CycleChoice::Auto,
        peck: 0.0,
        dwell: 0.0,
        feed: 120.0,
        plunge_feed: 80.0,
        holes: vec![hole],
        cross: vec![],
    };
    create(&world, &mut history, "AX1", "DR-0400", params);
    (world, history)
}

fn g76() -> PostOptions {
    PostOptions { threading: ThreadingOutput::G76Cycle, ..PostOptions::default() }
}

/// The document's stream through the script with these options; the
/// script must add no warning beyond the stream's own WARN records.
fn program_with(world: &World, options: PostOptions) -> String {
    let records = posting::stream_records(world).expect("every op generated");
    let out = engine::run_with(CPS, &posting::machine_value(world), &cnc::serialize(&records), &posting::options_value(options))
        .expect("the script runs");
    let warns: Vec<String> = records
        .iter()
        .filter_map(|r| match r {
            cnc::Record::Warn { text } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(out.warnings, warns, "the PathPilot lathe post refused a turning stream by name");
    linuxcnc_sane(&out.program);
    out.program
}

/// The document's program through the in-app gate (the post chosen by
/// name, the gate run, the file written) - and it must equal the
/// stream's own translation.
fn gated(world: &World, tag: &str) -> String {
    let path = std::env::temp_dir().join(format!("camcraft-tormach-lathe-{tag}-{}.nc", std::process::id()));
    let _ = std::fs::remove_file(&path);
    posting::post_setup(world, &path).expect("a clean lathe program posts through the gate");
    let text = std::fs::read_to_string(&path).expect("written");
    let _ = std::fs::remove_file(&path);
    text
}

// ------------------------------------------------------------ syntax

const GCODES: &[&str] = &[
    "G0", "G1", "G2", "G3", "G4", "G7", "G18", "G20", "G21", "G30", "G33", "G40", "G54", "G55", "G56", "G57",
    "G58", "G59", "G64", "G76", "G90", "G91.1", "G95", "G96", "G97",
];
const MCODES: &[&str] = &["M0", "M1", "M3", "M5", "M7", "M8", "M9", "M30"];

/// A LinuxCNC / PathPilot lathe syntax check over a posted program:
/// G7 stated before any motion, every comment one un-nested
/// parenthesis pair, every word a letter the lathe tables list with a
/// number, every G and M from the documented sets above (G32, G80-G83,
/// M6, G43 and friends are not among them), no Y, no O, no %.
fn linuxcnc_sane(text: &str) {
    let first_motion = text.lines().position(|l| l.starts_with("G0 ") || l.starts_with("G1 "));
    let g7 = text.lines().position(|l| l.split(' ').any(|w| w == "G7"));
    assert!(g7.is_some() && g7 < first_motion.or(Some(usize::MAX)), "G7 before any motion: {text}");
    assert!(!text.contains('%'), "no % fence: {text}");
    for line in text.lines() {
        if line.starts_with('(') {
            assert!(line.ends_with(')'), "a comment is one line: {line}");
            let inner = &line[1..line.len() - 1];
            assert!(!inner.contains('(') && !inner.contains(')'), "no nested comment: {line}");
            continue;
        }
        assert!(!line.contains('(') && !line.contains(';'), "no trailing comment on a motion line: {line}");
        for w in line.split(' ') {
            let mut ch = w.chars();
            let letter = ch.next().unwrap_or(' ');
            let num = ch.as_str();
            assert!("GMTSFXZIJKPRQHD".contains(letter), "a word LinuxCNC's lathe lists: {w} in {line}");
            assert!(num.parse::<f64>().is_ok(), "a word with a number: {w} in {line}");
            if letter == 'G' {
                assert!(GCODES.contains(&w), "G code outside the PathPilot lathe set: {w} in {line}");
            }
            if letter == 'M' {
                assert!(MCODES.contains(&w), "M code outside the PathPilot lathe set: {w} in {line}");
            }
            if letter == 'T' {
                assert!(num.len() == 4, "T word is tool + offset, T0101: {w}");
            }
        }
    }
}

// ------------------------------------------------------------ goldens

#[test]
fn turning_posts_to_the_pathpilot_lathe_goldens_in_mm_and_inch() {
    for (inch, file) in [(false, "turning_mm"), (true, "turning_inch")] {
        let (world, _history) = turning_world(inch);
        let text = program_with(&world, PostOptions::default());
        assert!(text.starts_with("(TestBench PathPilot lathe)\n"), "{text}");
        let unit = if inch { "G20" } else { "G21" };
        assert!(text.contains(&format!("G7 G18 {unit} G90 G91.1 G40 G95\n")), "{text}");
        assert_eq!(text.matches("G97 S3000 M3").count(), 2, "one spindle start per section: {text}");
        assert!(text.contains("G30\nT0101\n"), "G30 then the T word, no M6: {text}");
        if inch {
            // 150 mm/min at 3000 rpm: 0.05 mm/rev = 0.00197 in/rev.
            assert!(text.contains("F0.00197"), "{text}");
        } else {
            assert!(text.contains("F0.05") && text.contains("F0.04"), "{text}");
        }
        assert!(text.ends_with("M5\nG30\nM30\n"), "{text}");
        assert_eq!(gated(&world, file), text, "the gated program is the stream's own translation");
        common::write_golden(&format!("nc/tormach-lathe/{file}.nc"), &text);
    }
}

#[test]
fn a_thread_posts_as_g33_passes_by_default_in_mm_and_inch() {
    for (inch, file) in [(false, "thread_g33_mm"), (true, "thread_g33_inch")] {
        let (world, _history) = thread_world(inch);
        let text = program_with(&world, PostOptions::default());
        assert!(!text.contains("G76"), "{text}");
        let k = if inch { "K0.07874" } else { "K2\n" };
        assert!(text.contains(k), "K = the lead, distance per revolution: {text}");
        // One synchronized block per pass: three rungs of 0.4 to 1.2.
        assert_eq!(text.matches("G33 ").count(), 3, "one G33 per pass, half-turns merged: {text}");
        common::write_golden(&format!("nc/tormach-lathe/{file}.nc"), &text);
    }
}

#[test]
fn a_thread_posts_as_one_g76_block_with_the_option_in_mm_and_inch() {
    for (inch, file) in [(false, "thread_g76_mm"), (true, "thread_g76_inch")] {
        let (world, mut history) = thread_world(inch);
        history.execute(&world, SetPostOptions::new(&world, g76()).expect("G76 chosen"));
        let text = program_with(&world, g76());
        assert_eq!(text.matches("G76").count(), 1, "one block, LinuxCNC's form: {text}");
        assert!(!text.contains("G33"), "the passes are the control's now: {text}");
        if !inch {
            // Drive line at the start radius; peak Ø12, so I = 12 - drive Ø;
            // J 0.8 = the first 0.4 rung on the diameter; K 2.4 = 1.2 deep.
            let line = text.lines().find(|l| l.starts_with("G76")).expect("G76");
            assert!(line.starts_with("G76 P2 Z14 I-"), "{line}");
            assert!(line.ends_with(" J0.8 R1 K2.4 Q0 H0"), "{line}");
        } else {
            let line = text.lines().find(|l| l.starts_with("G76")).expect("G76");
            assert!(line.starts_with("G76 P0.07874 Z0.5512 I-"), "{line}");
            assert!(line.ends_with(" J0.0315 R1 K0.0945 Q0 H0"), "{line}");
        }
        assert_eq!(gated(&world, file), text, "the in-app post follows the document's option");
        common::write_golden(&format!("nc/tormach-lathe/{file}.nc"), &text);
    }
}

#[test]
fn an_axial_drill_posts_as_plain_moves_in_mm_and_inch() {
    for (inch, file) in [(false, "axial_drill_mm"), (true, "axial_drill_inch")] {
        let (world, _history) = drill_world(inch);
        let text = program_with(&world, PostOptions::default());
        for no in ["G81", "G83", "G80", "G98"] {
            assert!(!text.contains(no), "no canned cycle word on the PathPilot lathe ({no}): {text}");
        }
        let floor = if inch { "G1 X0 Z0.3937 " } else { "G1 X0 Z10 " };
        assert!(text.contains(floor), "the chain feeds to the floor on centre: {text}");
        assert!(text.contains("G97 S10000 M3"), "{text}");
        assert_eq!(gated(&world, file), text);
        common::write_golden(&format!("nc/tormach-lathe/{file}.nc"), &text);
    }
}

// ------------------------------------------------------- synthetic

fn lathe_mch(inch: bool) -> serde_json::Value {
    camcraft_app::post::machine::mirror(&lathe(inch))
}

fn mv(kind: MoveKind, x: f64, y: f64, z: f64, feed: Option<f64>) -> cnc::Record {
    cnc::Record::Move(cnc::MoveRec { kind, x, y, z, feed, center: None, ccw: false, plane: Default::default(), abc: None })
}

fn preface() -> Vec<cnc::Record> {
    vec![
        cnc::Record::Param { name: "SpindleSpeed".into(), value: json!(1000.0) },
        cnc::Record::Param { name: "WorkOffset".into(), value: json!("G54") },
        cnc::Record::Param { name: "OperationName".into(), value: json!("OP1") },
        cnc::Record::Tool { number: 1, name: "TN-1600-0R8".into() },
        cnc::Record::Section { begin: true },
    ]
}

fn run(records: Vec<cnc::Record>) -> engine::PostOutput {
    engine::run(CPS, &lathe_mch(false), &cnc::serialize(&records)).expect("the script runs")
}

fn cycle(kind: CycleKind, y: f64, dwell: f64) -> cnc::Record {
    cnc::Record::Cycle(cnc::CycleRec {
        kind, x: 0.0, y, r_z: 22.0, bottom_z: 10.0, entry_z: 20.0, peck: 0.0, dwell,
        first_move: 0, feed: Some(100.0),
    })
}

#[test]
fn a_dwelled_hole_dwells_in_seconds_at_its_floor() {
    let mut r = preface();
    r.push(cycle(CycleKind::Drill, 0.0, 0.5));
    r.push(mv(MoveKind::Rapid, 0.0, 0.0, 22.0, None));
    r.push(mv(MoveKind::Feed, 0.0, 0.0, 10.0, Some(100.0)));
    r.push(mv(MoveKind::Rapid, 0.0, 0.0, 22.0, None));
    r.push(cnc::Record::Section { begin: false });
    let out = run(r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(out.program.contains("G1 X0 Z10 F0.1\nG4 P0.5\nG0 X0 Z22\n"), "{}", out.program);
    linuxcnc_sane(&out.program);
}

#[test]
fn css_arcs_part_load_and_a_trip_home_speak_linuxcnc() {
    // G96 with the machine's ceiling as D (m/min under G21, ft/min
    // under G20); a ZX arc with I a RADIUS increment (G7 keeps I a
    // radius); the part-load advance in the program's unit; Go home as
    // G30, written once at the end (#504).
    for inch in [false, true] {
        let mut r = vec![cnc::Record::Param { name: "SurfaceSpeed".into(), value: json!(200.0) }];
        r.extend(preface());
        r.insert(0, cnc::Record::Param { name: "AfterOperation".into(), value: json!("home") });
        r.push(cnc::Record::Part { loading: true, advance: 25.4 });
        r.push(mv(MoveKind::Rapid, 0.0, 5.0, 2.0, None));
        r.push(cnc::Record::Move(cnc::MoveRec {
            kind: MoveKind::Feed, x: 0.0, y: 7.0, z: 0.0, feed: Some(100.0), center: Some((0.0, 0.0)),
            ccw: false, plane: camcraft_app::cl::ArcPlane::Yz, abc: None,
        }));
        r.push(cnc::Record::Section { begin: false });
        let out = engine::run(CPS, &lathe_mch(inch), &cnc::serialize(&r)).expect("runs");
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        let p = &out.program;
        if inch {
            assert!(p.contains("G96 D12000 S656 M3\n"), "200 m/min is 656 ft/min: {p}");
            assert!(p.contains("(PART LOAD ADVANCE 1 IN)"), "{p}");
        } else {
            assert!(p.contains("G96 D12000 S200 M3\n"), "{p}");
            assert!(p.contains("(PART LOAD ADVANCE 25.4 MM)"), "{p}");
            // r 5 -> r 7 about (r 0, z 0) seen on the +Y meridian: CW
            // there reads CCW in ZX; X diameter 14, I K from the start.
            assert!(p.contains("G3 X14 Z0 I-5 K-2 F0.1\n"), "{p}");
        }
        assert!(p.ends_with("M9\nM5\nG30\nM30\n"), "home once, not twice: {p}");
        linuxcnc_sane(p);
    }
}

#[test]
fn what_the_dialect_cannot_say_is_refused_by_name() {
    // A tap (no G33.1 yet), a back-bore and a hole off the axis.
    for (kind, y, why) in [
        (CycleKind::Tap, 0.0, "tapping"),
        (CycleKind::BackBore, 0.0, "back-boring"),
        (CycleKind::Drill, 5.0, "off the spindle axis"),
    ] {
        let mut r = preface();
        r.push(cycle(kind, y, 0.0));
        r.push(mv(MoveKind::Rapid, 0.0, y, 22.0, None));
        r.push(mv(MoveKind::Feed, 0.0, y, 10.0, Some(100.0)));
        r.push(cnc::Record::Section { begin: false });
        let out = run(r);
        assert!(out.warnings.iter().any(|w| w.contains(why)), "{why}: {:?}", out.warnings);
        assert!(!out.program.contains("Z10"), "{why}: the refused hole wrote its chain: {}", out.program);
    }
    // A move off every meridian: milling work.
    let mut r = preface();
    r.push(mv(MoveKind::Feed, 40.0, 30.0, -1.0, Some(800.0)));
    r.push(cnc::Record::Section { begin: false });
    assert!(run(r).warnings.iter().any(|w| w.contains("milling work")));
    // A tool the T word cannot name.
    let r = vec![
        cnc::Record::Param { name: "SpindleSpeed".into(), value: json!(1000.0) },
        cnc::Record::Tool { number: 100, name: "X".into() },
        cnc::Record::Section { begin: true },
        cnc::Record::Section { begin: false },
    ];
    assert!(run(r).warnings.iter().any(|w| w.contains("T01-T99")));
    // Control compensation.
    let mut r = preface();
    r.push(cnc::Record::Comp { side: Some(camcraft_app::cl::CompSide::Left), radius_mm: 0.8, wear: false });
    r.push(cnc::Record::Section { begin: false });
    assert!(run(r).warnings.iter().any(|w| w.contains("nose-radius compensation")));
    // A C-axis live-tool section: the script does not declare liveTooling.
    let mut r = preface();
    r.insert(0, cnc::Record::Param { name: "LiveTool".into(), value: json!(true) });
    r.push(cnc::Record::Section { begin: false });
    assert!(run(r).warnings.iter().any(|w| w.contains("liveTooling")));
}

#[test]
fn a_g76_record_the_cycle_cannot_state_exactly_is_refused_by_name() {
    let (world, _history) = thread_world(false);
    let records = posting::stream_records(&world).expect("generated");
    let bend = |f: fn(&mut cnc_thread::ThreadRec)| -> Vec<cnc::Record> {
        records
            .iter()
            .cloned()
            .map(|r| match r {
                cnc::Record::Thread(Some(mut t)) => {
                    f(&mut t);
                    cnc::Record::Thread(Some(t))
                }
                other => other,
            })
            .collect()
    };
    let bare: Vec<cnc::Record> = records.iter().filter(|r| !matches!(r, cnc::Record::Thread(_))).cloned().collect();
    type Bend = fn(&mut cnc_thread::ThreadRec);
    let cases: [(&str, Vec<cnc::Record>); 6] = [
        ("no thread cycle record", bare),
        ("infeed angle", bend((|t| t.infeed_angle = 29.5) as Bend)),
        ("finish allowance", bend((|t| t.finish_allowance = 0.05) as Bend)),
        ("pull-out", bend((|t| t.pull_out = 1.0) as Bend)),
        ("not equal rungs", bend((|t| t.min_depth = 0.1) as Bend)),
        ("not an external thread", bend((|t| t.minor_diameter = 13.0) as Bend)),
    ];
    for (why, recs) in cases {
        let out = engine::run_with(CPS, &posting::machine_value(&world), &cnc::serialize(&recs), &posting::options_value(g76()))
            .expect("runs");
        assert!(out.warnings.iter().any(|w| w.contains("G76") && w.contains(why)), "{why}: {:?}", out.warnings);
        assert!(!out.program.contains("G76 P"), "{why}: a refused cycle was written: {}", out.program);
    }
}

// ------------------------------------------------------------ registry

#[test]
fn the_registry_lists_the_pathpilot_lathe_post_for_lathes_only() {
    assert!(posting::BUILTINS.iter().any(|(n, _)| *n == NAME));
    assert!(posting::post_serves(NAME, Kind::Lathe));
    assert!(posting::post_serves(NAME, Kind::MillTurn));
    assert!(!posting::post_serves(NAME, Kind::Mill));
    // The lathe default is unchanged: Fanuc Lathe stays first.
    assert_eq!(posting::default_post_for(Kind::Lathe), PostRef::Named("Fanuc Lathe".into()));
    let world = new_world();
    assert!(SelectPost::new(&world, PostRef::Named(NAME.into())).is_ok());
}
