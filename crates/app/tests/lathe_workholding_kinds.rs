//! The workholding rpm limit fails closed for EVERY operation kind a
//! lathe or mill-turn setup can hold (#1174 review): the program gate
//! reads the same derivation the stream does (`post::section_spindle`),
//! so a section the lathe post opens on the work spindle (`G97 S M03`,
//! whatever its kind) is judged at its stated rpm, a C section by the
//! machine's C rate, and nothing posts with the work spindle above the
//! chuck's rating. Before the fix a mill-turn's on-centre Drill at 8000
//! rpm posted `G97 S8000 M03` under a 2000 rpm chuck: the gate only
//! judged what `spindle_law::turns` named.
//!
//! Gate map (M4.4): covers app::setup_workholding (the program gate),
//! post::post (section_spindle), app::posting (post_script).
//! Lane shape (M4.4): the default tests run in seconds each; the slow
//! kinds' rows are one heavy-lane test (`SLOW`).

use flecs_ecs::prelude::World;

use camcraft_app::commands::{
    AddTool, CreateOperation, CreateSetup, GenerateOperation, History, ImportModel, LoadMachine, SetSetupWorkholdingRpm,
    SetStock,
};
use camcraft_app::drilling::Hole;
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machine::{AxisLimits, Coolant, LiveToolCodes, Machine};
use camcraft_app::operation::{seed_params, KindInfo, OpParams, OpType, SrcRef};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::{catalog, Tool};
use camcraft_app::world::new_world;
use camcraft_app::posting;

const TAU: f64 = std::f64::consts::TAU;
const FANUC: &str = include_str!("../../../assets/posts/fanuc-lathe.cps");
const CHUCK: f64 = 2000.0;
const FAST: f64 = 8000.0;
/// A bar r6, z -16..16, with a blind r2.2 centre hole down to z 6.
const BORED: [(f64, f64); 6] = [(0.0, -16.0), (6.0, -16.0), (6.0, 16.0), (2.2, 16.0), (2.2, 6.0), (0.0, 6.0)];

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

fn machine(kind: &str) -> Machine {
    let lin = |n: &str| AxisLimits { name: n.into(), feed: Some(4000.0), ..Default::default() };
    let c = AxisLimits {
        name: "C".into(),
        rotary: true,
        axis: Some([0.0, 0.0, 1.0]),
        feed: Some(6000.0),
        rapid: Some(12_000.0),
        ..Default::default()
    };
    Machine {
        model: format!("TestBench {kind}"),
        kind: Some(kind.into()),
        spindle_min: Some(25.0),
        spindle_max: Some(12_000.0),
        coolants: vec![Coolant { name: "Flood".into(), m_code: "M8".into() }],
        axes: vec![lin("X"), lin("Y"), lin("Z"), c],
        live_tool: Some(LiveToolCodes {
            c_axis_on: "M1001".into(),
            c_axis_off: "M1002".into(),
            spindle_on: "M1003".into(),
            spindle_off: "M1004".into(),
        }),
        ..Default::default()
    }
}

/// The bored bar in its own stock on a `kind` machine, Setup1 stating
/// `chuck` rpm, every catalog tool at `rpm`.
fn doc(kind: &str, chuck: f64, rpm: f64) -> (World, History) {
    let world = new_world();
    let mut h = History::default();
    let at = Location { pos: Point3::new(0.0, 0.0, -16.0), ..Location::default() };
    let stock = SetStock::new(&world, StockShape::Cylinder { diameter: 12.0, height: 32.0 }, at);
    h.execute(&world, stock.expect("bar stock"));
    h.execute(&world, LoadMachine::new(&world, "bench.mch".into(), Some(machine(kind))).expect("machine"));
    for (name, t) in catalog() {
        let flute_len = t.flute_len.max(20.0);
        if let Ok(cmd) = AddTool::new(&world, name, Tool { rpm, flute_len, ..t }) {
            h.execute(&world, cmd);
        }
    }
    h.execute(&world, ImportModel::from_mesh("bar", revolve(&BORED, 24)));
    h.execute(&world, CreateSetup::new(&world, None, None).expect("Setup1"));
    h.execute(&world, SetSetupWorkholdingRpm::new(&world, "Setup1", Some(chuck)).expect("chuck limit"));
    (world, h)
}

/// The centre hole, for the kinds that take a hole list.
fn on_centre(p: &mut OpParams) {
    let hole = Hole { xy: Point2::new(0.0, 0.0), radius: 2.2, entry_z: 16.0, floor_z: 6.0, through: false, stepped: false };
    match p {
        OpParams::Drill { holes, .. } | OpParams::Bore { holes, .. } | OpParams::Thread { holes, .. } => *holes = vec![hole],
        _ => {}
    }
}

/// Create kind `i` as "K" with the first catalog tool its door takes,
/// and generate it. `false` when no tool fits or generation is refused.
fn add_kind(world: &World, h: &mut History, i: usize) -> bool {
    let mut p = seed_params(i);
    on_centre(&mut p);
    let made = catalog().into_iter().find_map(|(tool, _)| CreateOperation::new(world, "K", tool, SrcRef::Entity("bar".into()), p.clone()).ok());
    let Some(cmd) = made else { return false };
    h.execute(world, cmd);
    match GenerateOperation::new(world, "K") {
        Ok(g) => {
            h.execute(world, g);
            true
        }
        Err(_) => false,
    }
}

fn chuck_lines(world: &World) -> Vec<String> {
    posting::verification_gate(world).into_iter().filter(|p| p.contains("workholding")).collect()
}

/// Every work-spindle start in `program` held to `limit`: a G97's S
/// itself, a G96's G50 clamp in force. A live spindle's S is the tool's.
fn work_spindle_within(program: &str, limit: f64) -> Result<(), String> {
    let s_of = |l: &str| l.split(' ').find_map(|w| w.strip_prefix('S').and_then(|v| v.parse::<f64>().ok()));
    let mut clamp = f64::INFINITY;
    for l in program.lines() {
        if l.starts_with("G50") {
            clamp = s_of(l).unwrap_or(f64::INFINITY);
        }
        let starts = l.contains("M03") || l.contains("M04");
        if starts && l.contains("G97") && s_of(l).is_none_or(|s| s > limit) {
            return Err(format!("work spindle over the chuck: {l}"));
        }
        if starts && l.contains("G96") && clamp > limit {
            return Err(format!("G96 clamped at {clamp} over the chuck: {l}"));
        }
    }
    Ok(())
}

enum Outcome {
    Limit,
    Posted,
    Refused(String),
    NotBuilt,
}

fn judge(kind: &str, i: usize, chuck: f64, rpm: f64) -> Outcome {
    let (world, mut h) = doc(kind, chuck, rpm);
    if !add_kind(&world, &mut h, i) {
        return Outcome::NotBuilt;
    }
    if !chuck_lines(&world).is_empty() {
        return Outcome::Limit;
    }
    let out = std::env::temp_dir().join(format!("camcraft-1174-{}-{kind}-{i}.nc", std::process::id()));
    let posted = posting::post_script(&world, &out, FANUC);
    let program = std::fs::read_to_string(&out).unwrap_or_default();
    let _ = std::fs::remove_file(&out);
    match posted {
        Err(e) => Outcome::Refused(e),
        Ok(_) => {
            if let Err(why) = work_spindle_within(&program, chuck) {
                panic!("{kind} kind {} posted with the work spindle over the {chuck} rpm chuck - {why}\n{program}", KindInfo::ALL[i].label);
            }
            Outcome::Posted
        }
    }
}

/// Every kind `mk` admits whose registry index falls in `range`, judged
/// on a `kind` machine with the tools at 8000 rpm under a 2000 rpm chuck:
/// refused by the limit, posted with the work spindle within it (a C
/// section, judged by the C rate), refused for another named reason, or
/// not buildable on the fixture - never posted over the chuck (`judge`
/// panics on that). Split in slices so no one test runs past a minute.
fn table(kind: &str, mk: camcraft_app::machine::Kind, range: std::ops::Range<usize>, slow: bool) {
    let mut rows = Vec::new();
    let picked = |i: &usize, k: &KindInfo| range.contains(i) && k.admits(mk) && SLOW.contains(&k.ty) == slow;
    for (i, k) in KindInfo::ALL.iter().enumerate().filter(|(i, k)| picked(i, k)) {
        let t = std::time::Instant::now();
        let o = match judge(kind, i, CHUCK, FAST) {
            Outcome::Limit => "refused by the chuck limit".to_string(),
            Outcome::Posted => "posted, work spindle within the chuck".to_string(),
            Outcome::Refused(e) => format!("refused: {}", e.chars().take(110).collect::<String>()),
            Outcome::NotBuilt => "not built (no tool / generation refused)".to_string(),
        };
        rows.push(format!("{kind:9} {:24} {:5.1}s {o}", k.label, t.elapsed().as_secs_f64()));
    }
    println!("{}", rows.join("
"));
}

const N: usize = KindInfo::ALL.len();

/// The kinds whose generation on the bar takes 15-65 s in debug: their
/// rows run in the heavy lane (`every_slow_kind_...`). The C-rate rule
/// they reach is pinned in the default lane by `lathe_workholding_live`.
const SLOW: [OpType; 5] = [OpType::Contour5D, OpType::Parallel5D, OpType::Flow5D, OpType::RotaryContour, OpType::TrochoidalSlot];

#[test]
fn no_lathe_kind_posts_the_work_spindle_over_the_chuck() {
    table("lathe", camcraft_app::machine::Kind::Lathe, 0..N, false);
}

#[test]
fn no_mill_turn_kind_posts_the_work_spindle_over_the_chuck_first_third() {
    table("mill-turn", camcraft_app::machine::Kind::MillTurn, 0..N / 3, false);
}

#[test]
fn no_mill_turn_kind_posts_the_work_spindle_over_the_chuck_second_third() {
    table("mill-turn", camcraft_app::machine::Kind::MillTurn, N / 3..2 * N / 3, false);
}

#[test]
fn no_mill_turn_kind_posts_the_work_spindle_over_the_chuck_last_third() {
    table("mill-turn", camcraft_app::machine::Kind::MillTurn, 2 * N / 3..N, false);
}

#[test]
#[ignore = "heavy lane (#1174 per-kind workholding table): the 5-axis, rotary-contour and trochoidal kinds on a mill-turn, ~2 min debug - run at the cut with -- --ignored"]
fn every_slow_kind_on_a_mill_turn_keeps_the_work_spindle_within_the_chuck() {
    table("mill-turn", camcraft_app::machine::Kind::MillTurn, 0..N, true);
}

#[test]
fn a_mill_turn_on_centre_drill_bore_is_judged_like_the_lathes() {
    use camcraft_app::machine::Kind;
    for (kind, mk) in [("lathe", Kind::Lathe), ("mill-turn", Kind::MillTurn)] {
        for ty in [OpType::Drill, OpType::Bore] {
            let i = KindInfo::ALL.iter().position(|k| k.ty == ty).expect("kind");
            if !KindInfo::ALL[i].admits(mk) {
                continue;
            }
            assert!(matches!(judge(kind, i, CHUCK, FAST), Outcome::Limit), "{kind} {ty:?} at 8000 under 2000 is refused by the limit");
            assert!(!matches!(judge(kind, i, CHUCK, CHUCK), Outcome::Limit), "{kind} {ty:?} exactly at the limit is allowed");
        }
    }
}
