//! #533 (part of #290): the generic ISO wire EDM post. A Wire Contour
//! posts end to end through the app's gate on a synthetic wire EDM whose
//! `edm.codes` are invented (clearly fictional `FX-...` words, #290
//! D10) - the golden locks the RS-274 motion and where each machine code
//! lands; every code the post needs, removed, refuses the post by its
//! name; a mill machine, and a cutting post on the wire section, are
//! refused by name; the registry offers the post for wire EDMs only.
//! #538: a rough pass and two skims post with three distinct offsets
//! and condition selects (their own golden); each skim refusal by name;
//! the rough-only golden above does not move.
//! Not exercised on a real EDM control (D10).
//!
//! Gate map (#533): covers app::posting (posting_wire, WIRE_POSTS,
//! post_serves, default_post_for, op_streams' wire branch),
//! app::posting_family (edm_at_post: wire passes, sinker refused), post::engine
//! (the wireEdm door), post::machine (edm codes mirror, edm_codes), and
//! the wire-iso.cps script; #538: operation::wire_op (skims), motion::
//! wire_edm::skims, posting_wire's skim announcements.
//! Provenance: our own tests over our own synthetic fixture.

#[path = "edm/sinker_fixture.rs"]
mod sinker_fixture;

use flecs_ecs::prelude::World;

use camcraft_app::commands::{CreateOperation, EditOperationParams, GenerateOperation, History, ImportModel, LoadMachine};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machine::{parse_mch, Kind};
use camcraft_app::operation::{Height, OpParams, SrcRef, WireContourOp, WireSide, WireSkim};
use camcraft_app::post::{self, cnc};
use camcraft_app::model;
use camcraft_app::posting::{self, BUILTINS, WIRE_POSTS};
use camcraft_app::project;
use camcraft_app::world::new_world;

const FIXTURE: &str = include_str!("edm/wire-iso-fixture.mch");
const WIRE: &str = "Generic ISO Wire EDM";
const GOLDEN: &str = "tests/golden/nc/wire-iso/wire_slug_punch.nc";
const SKIMS_GOLDEN: &str = "tests/golden/nc/wire-iso/wire_contour_punch_skims.nc";

fn cps(name: &str) -> &'static str {
    BUILTINS.iter().find(|(n, _)| *n == name).map(|(_, s)| *s).expect("registered")
}

/// A sharp 40 x 20 slug from z = 0 to 12, centred on the origin.
fn slug() -> Mesh {
    let pts = [(20.0, 10.0), (-20.0, 10.0), (-20.0, -10.0), (20.0, -10.0)];
    let p = |(x, y): (f64, f64), z: f64| Point3::new(x, y, z);
    let mut f = Vec::new();
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        f.push(Triangle::new(Point3::new(0.0, 0.0, 12.0), p(a, 12.0), p(b, 12.0)));
        f.push(Triangle::new(Point3::new(0.0, 0.0, 0.0), p(b, 0.0), p(a, 0.0)));
        f.push(Triangle::new(p(a, 0.0), p(b, 0.0), p(b, 12.0)));
        f.push(Triangle::new(p(a, 0.0), p(b, 12.0), p(a, 12.0)));
    }
    Mesh::from_facets(f)
}

/// The fixture machine mounted, a punch cut around the slug from a
/// start hole outside it, generated.
fn punch_doc(machine_text: &str) -> World {
    punch_doc_with(machine_text, Vec::new())
}

/// [`punch_doc`] with skim passes after the rough cut (#538).
fn punch_doc_with(machine_text: &str, skims: Vec<WireSkim>) -> World {
    let world = new_world();
    let mut h = History::default();
    let m = parse_mch(machine_text).expect("the fixture parses");
    h.execute(&world, LoadMachine::new(&world, "wire-iso.mch".into(), Some(m)).expect("mount"));
    h.execute(&world, ImportModel::from_mesh("slug", slug()));
    let op = WireContourOp {
        top_z: Height::fixed(12.0),
        bottom_z: Height::fixed(0.0),
        condition: "C110".into(),
        side: WireSide::Punch,
        start: Some([-30.0, 0.0]),
        skims,
    };
    let c = CreateOperation::new(&world, "Punch1", "", SrcRef::Entity("slug".into()), OpParams::WireContour(op)).expect("op");
    h.execute(&world, c);
    h.execute(&world, GenerateOperation::new(&world, "Punch1").expect("generates"));
    world
}

fn stream_text(world: &World) -> String {
    cnc::serialize(&posting::stream_records(world).expect("the wire section streams"))
}

/// The engine's counted warnings for `cps` over `stream` on `machine_text`.
fn warnings(cps: &str, machine_text: &str, stream: &str) -> Vec<String> {
    let m = parse_mch(machine_text).expect("parses");
    post::engine::run(cps, &post::machine::mirror(&m), stream).expect("the engine runs").warnings
}

/// Done means: the 2-axis contour posts a golden against the synthetic
/// fixture dialect - through the app's own gate, verification included.
#[test]
fn the_punch_contour_posts_its_golden_through_the_gate() {
    let world = punch_doc(FIXTURE);
    let out = std::env::temp_dir().join(format!("camcraft-wire-iso-{}.nc", std::process::id()));
    let posted = posting::post_script(&world, &out, cps(WIRE)).expect("the wire program posts");
    let _ = std::fs::remove_file(&out);
    let text = posted.program;
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        std::fs::write(GOLDEN, &text).expect("write golden");
    }
    let want = std::fs::read_to_string(GOLDEN).expect("golden present");
    assert_eq!(text, want, "the wire program drifted from its golden");
    // The shape the golden locks, said out loud: the wire goes to the
    // start hole cut, threads, selects the condition, cancels comp, cuts
    // in X Y only with no F, then cuts the wire and drops its tip.
    let lines: Vec<&str> = text.lines().collect();
    let at = |w: &str| lines.iter().position(|l| *l == w).unwrap_or_else(|| panic!("no {w} in\n{text}"));
    let (g0, th, cond, comp, cut, tip) =
        (at("G0 X-30 Y0"), at("FX-THREAD"), at("FX-COND C110"), at("FX-COMP-OFF"), at("FX-CUT"), at("FX-TIP-OUT"));
    assert!(g0 < th && th < cond && cond < comp && comp < cut && cut < tip, "{text}");
    let cuts: Vec<&&str> = lines[comp + 1..cut].iter().collect();
    assert!(!cuts.is_empty() && cuts.iter().all(|l| l.starts_with("G1 X") && !l.contains('Z') && !l.contains('F')), "{text}");
    // No control compensation, no spindle words.
    for l in &lines {
        let words: Vec<&str> = l.split_whitespace().collect();
        assert!(
            !words.iter().any(|w| ["G41", "G42", "M3", "M03", "M4", "M04"].contains(w) || (w.starts_with('S') && w[1..].parse::<f64>().is_ok())),
            "{l}"
        );
    }
}

/// Done means: removing any required code refuses by name.
#[test]
fn each_required_code_removed_refuses_the_post_by_name() {
    let world = punch_doc(FIXTURE);
    let stream = stream_text(&world);
    assert!(warnings(cps(WIRE), FIXTURE, &stream).is_empty(), "the full fixture posts clean");
    for code in ["condition", "comp_off", "thread", "cut", "tip_disposal"] {
        let mut v: serde_json::Value = serde_json::from_str(FIXTURE).expect("json");
        v["edm"]["codes"].as_object_mut().expect("codes").remove(code);
        let w = warnings(cps(WIRE), &v.to_string(), &stream);
        let want = format!("machine SYNTHETIC WIRE EDM (FICTIONAL CODES) states no edm.codes.{code}");
        assert!(w.iter().any(|m| m.contains(&want)), "{code}: {w:?}");
    }
    // Without AWT the operator threads and cuts at a stop: the AWT codes
    // are not needed, and the program says so with M00.
    let mut v: serde_json::Value = serde_json::from_str(FIXTURE).expect("json");
    v["edm"]["awt"] = serde_json::json!(false);
    for code in ["thread", "cut", "tip_disposal"] {
        v["edm"]["codes"].as_object_mut().expect("codes").remove(code);
    }
    let m = parse_mch(&v.to_string()).expect("parses");
    let out = post::engine::run(cps(WIRE), &post::machine::mirror(&m), &stream).expect("runs");
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert_eq!(out.program.lines().filter(|l| *l == "M00").count(), 2, "{}", out.program);
    // A condition code with no place for the id selects nothing: refused.
    let bad = FIXTURE.replace("FX-COND {id}", "FX-COND");
    let w = warnings(cps(WIRE), &bad, &stream);
    assert!(w.iter().any(|m| m.contains("edm.codes.condition states no {id}")), "{w:?}");
}

/// The post refuses a non-EDM machine by name, and no cutting post
/// voices the wire section (the engine's wireEdm door).
#[test]
fn a_mill_machine_and_a_cutting_post_are_refused_by_name() {
    let world = punch_doc(FIXTURE);
    let stream = stream_text(&world);
    let mill = std::fs::read_to_string("../../assets/machines/3axis-vmc.mch").expect("the vmc");
    let w = warnings(cps(WIRE), &mill, &stream);
    assert!(w.iter().any(|m| m.contains("is not declared kind wire_edm")), "{w:?}");
    for (n, s) in BUILTINS.iter().filter(|(n, _)| *n != WIRE) {
        let w = warnings(s, FIXTURE, &stream);
        assert!(w.iter().any(|m| m.contains("Punch1") && m.contains("does not declare wireEdm")), "{n}: {w:?}");
    }
    // And a milling section through the wire post is refused by name.
    let golden = std::fs::read_to_string("tests/golden/face_square_40x40.cl").expect("a mill golden");
    let w = warnings(cps(WIRE), FIXTURE, &golden);
    assert!(w.iter().any(|m| m.contains("not wire EDM work") || m.contains("outside a wire section")), "{w:?}");
}

#[test]
fn the_registry_offers_the_wire_post_for_wire_edms_only() {
    assert_eq!(WIRE_POSTS, &[WIRE]);
    for (n, _) in BUILTINS {
        assert_eq!(posting::post_serves(n, Kind::WireEdm), WIRE_POSTS.contains(n), "{n}");
    }
    for k in [Kind::Mill, Kind::Lathe, Kind::MillTurn, Kind::Plasma, Kind::SinkerEdm] {
        assert!(!posting::post_serves(WIRE, k), "{k:?}");
    }
    assert_eq!(posting::post_name(&posting::default_post_for(Kind::WireEdm)), Some(WIRE));
}

fn skim(id: &str, reverse: bool) -> WireSkim {
    WireSkim { condition: id.into(), reverse }
}

fn post_text(world: &World) -> String {
    let out = std::env::temp_dir().join(format!("camcraft-wire-skims-{}.nc", std::process::id()));
    let posted = posting::post_script(world, &out, cps(WIRE)).expect("the wire program posts");
    let _ = std::fs::remove_file(&out);
    posted.program
}

/// #538 Done means: a rough plus two skims post with three distinct
/// offsets and condition selects - one threading, one cut, the second
/// skim walked the other way round.
#[test]
fn a_rough_and_two_skims_post_three_offsets_and_condition_selects() {
    let world = punch_doc_with(FIXTURE, vec![skim("S21", false), skim("S31", true)]);
    let text = post_text(&world);
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        std::fs::write(SKIMS_GOLDEN, &text).expect("write golden");
    }
    let want = std::fs::read_to_string(SKIMS_GOLDEN).expect("golden present");
    assert_eq!(text, want, "the skimmed wire program drifted from its golden");
    let lines: Vec<&str> = text.lines().collect();
    let count = |w: &str| lines.iter().filter(|l| **l == w).count();
    assert_eq!((count("FX-THREAD"), count("FX-CUT"), count("FX-COMP-OFF")), (1, 1, 3), "{text}");
    let selects: Vec<usize> = ["FX-COND C110", "FX-COND S21", "FX-COND S31"]
        .iter()
        .map(|w| lines.iter().position(|l| l == w).unwrap_or_else(|| panic!("no {w} in\n{text}")))
        .collect();
    assert!(selects.windows(2).all(|w| w[0] < w[1]), "{text}");
    // Each pass's lead-in meets its own offset: wire radius 0.125 plus
    // the gap 0.04, 0.02, 0.01 - the punch's left side at X = -20 - k.
    for (i, x) in ["-20.165", "-20.145", "-20.135"].iter().enumerate() {
        let lead = lines[selects[i] + 2];
        assert_eq!(lead, format!("G1 X{x} Y0"), "pass {i}: {text}");
    }
    // The first skim keeps the rough's direction (down the left side),
    // the reversed second one goes up it.
    assert_eq!(lines[selects[1] + 3], "G1 X-20.145 Y-10.145", "{text}");
    assert_eq!(lines[selects[2] + 3], "G1 X-20.135 Y10.135", "{text}");
    assert_eq!(count("(SKIM 1)") + count("(SKIM 2)"), 2, "{text}");
}

/// #538: the rough-only program is byte-identical with skims absent,
/// and its project / MCP JSON carries no `skims` key.
#[test]
fn no_skims_is_the_rough_program_and_json_unchanged() {
    let world = punch_doc(FIXTURE);
    assert_eq!(post_text(&world), std::fs::read_to_string(GOLDEN).expect("golden present"));
    let op = model::find_by_name(&world, "Punch1").and_then(|e| model::operation_of(&world, &e)).expect("op");
    let json = project::params_to_json(&op.params);
    assert!(json["wire_contour"].get("skims").is_none(), "{json}");
}

/// #538: skims ride EditOperationParams (undoable) and the JSON door
/// the file and the MCP patch share.
#[test]
fn skims_edit_undo_and_round_trip() {
    let world = punch_doc(FIXTURE);
    let mut h = History::default();
    let params = |w: &World| model::find_by_name(w, "Punch1").and_then(|e| model::operation_of(w, &e)).map(|o| o.params);
    let Some(OpParams::WireContour(mut w)) = params(&world) else { panic!("a wire contour") };
    let p0 = OpParams::WireContour(w.clone());
    w.skims = vec![skim("S21", false), skim("S31", true)];
    let p1 = OpParams::WireContour(w);
    h.execute(&world, EditOperationParams::new(&world, "Punch1", p1.clone()).expect("edit"));
    assert_eq!(params(&world), Some(p1.clone()));
    assert!(h.undo(&world));
    assert_eq!(params(&world), Some(p0));
    assert!(h.redo(&world));
    let json = project::params_to_json(&p1);
    assert_eq!(json["wire_contour"]["skims"], serde_json::json!([{ "condition": "S21" }, { "condition": "S31", "reverse": true }]));
    assert_eq!(project::params_from_json(&json).expect("reads back"), p1);
}

/// #538: each skim refusal by name, at command creation.
#[test]
fn each_skim_refusal_is_named_at_the_door() {
    let world = punch_doc(FIXTURE);
    let refusal = |rough: &str, skims: Vec<WireSkim>| {
        let Some(OpParams::WireContour(mut w)) =
            model::find_by_name(&world, "Punch1").and_then(|e| model::operation_of(&world, &e)).map(|o| o.params)
        else {
            panic!("a wire contour")
        };
        w.condition = rough.into();
        w.skims = skims;
        EditOperationParams::new(&world, "Punch1", OpParams::WireContour(w)).map(|_| ()).expect_err("refused").to_string()
    };
    let e = refusal("C110", vec![skim("", false)]);
    assert!(e.contains("skim 1 of the Wire Contour names no EDM condition") && e.contains("C110, S21, S31, W20"), "{e}");
    let e = refusal("C110", vec![skim("S21", false), skim("S99", false)]);
    assert!(e.contains("skim 2 of the Wire Contour") && e.contains("states no EDM condition \"S99\""), "{e}");
    let e = refusal("C110", vec![skim("S31", false), skim("S21", false)]);
    assert!(e.contains("skim 1 (condition S31)") && e.contains("cut past the finished profile"), "{e}");
    let e = refusal("S31", vec![skim("S21", false)]);
    assert!(e.contains("the rough pass (S31)") && e.contains("cut past the finished profile"), "{e}");
    let e = refusal("C110", vec![skim("S31", false), skim("S21", false), skim("S31", false)]);
    assert!(e.contains("skim 2 (condition S21)") && e.contains("farther out than skim 1") && e.contains("cut only air"), "{e}");
    let e = refusal("C110", vec![skim("W20", false)]);
    assert!(e.contains("skim 1 (condition W20)") && e.contains("one threading cuts every pass with one wire"), "{e}");
}

/// Review of #1139: the EDM door at post (`posting_family::edm_at_post`)
/// lets a wire setup through to its post, so a wire machine missing a
/// code the post needs is refused by that code's name through the app's
/// own `post_script` - and no file is written.
#[test]
fn a_wire_setup_missing_a_code_is_refused_by_name_through_post_script() {
    for code in ["condition", "comp_off", "thread", "cut", "tip_disposal"] {
        let mut v: serde_json::Value = serde_json::from_str(FIXTURE).expect("json");
        v["edm"]["codes"].as_object_mut().expect("codes").remove(code);
        let world = punch_doc(&v.to_string());
        let out = std::env::temp_dir().join(format!("camcraft-wire-iso-missing-{code}-{}.nc", std::process::id()));
        let _ = std::fs::remove_file(&out);
        let err = posting::post_script(&world, &out, cps(WIRE)).expect_err("a wire machine missing a code never posts");
        let wrote = out.exists();
        let _ = std::fs::remove_file(&out);
        assert!(err.contains(&format!("edm.codes.{code}")), "{code}: {err}");
        assert!(!err.contains("#533"), "{code}: the EDM door no longer refuses a wire setup: {err}");
        assert!(!wrote, "{code}: the refused wire post wrote {}", out.display());
    }
}

/// Review of #1139: lifting the wire arm of the EDM door leaves the
/// sinker arm closed - a Sinker Burn is refused by name (#542) through
/// `post_script`, by the wire post and by a cutting post alike, and no
/// file is written.
#[test]
fn a_sinker_setup_is_still_refused_at_post_by_name() {
    let (world, mut h) = sinker_fixture::sinker_doc(6.0);
    let burn = sinker_fixture::burn(camcraft_app::operation::SinkerDatum::WorkTop, 3.0, 14.0);
    let op = CreateOperation::new(&world, "Burn0", sinker_fixture::ELECTRODE, SrcRef::Stock, burn).expect("a burn on its sinker");
    h.execute(&world, op);
    h.execute(&world, GenerateOperation::new(&world, "Burn0").expect("generates"));
    let out = std::env::temp_dir().join(format!("camcraft-wire-iso-sinker-{}.nc", std::process::id()));
    for (n, script) in BUILTINS {
        let _ = std::fs::remove_file(&out);
        let err = posting::post_script(&world, &out, script).expect_err("a sinker burn never posts");
        let wrote = out.exists();
        let _ = std::fs::remove_file(&out);
        assert!(err.contains("no post for a sinker EDM machine yet (#542)") && err.contains("Burn0"), "{n}: {err}");
        assert!(!wrote, "{n}: the refused sinker post wrote {}", out.display());
    }
}
