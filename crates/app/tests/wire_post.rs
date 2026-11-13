//! #533 (part of #290): the generic ISO wire EDM post. A Wire Contour
//! posts end to end through the app's gate on a synthetic wire EDM whose
//! `edm.codes` are invented (clearly fictional `FX-...` words, #290
//! D10) - the golden locks the RS-274 motion and where each machine code
//! lands; every code the post needs, removed, refuses the post by its
//! name; a mill machine, and a cutting post on the wire section, are
//! refused by name; the registry offers the post for wire EDMs only.
//! The wire laws (#534) gate it: a clean cut posts its golden, a
//! gouging one is refused by name with no NC text written.
//! Not exercised on a real EDM control (D10).
//!
//! Gate map (#533): covers app::posting (posting_wire, WIRE_POSTS,
//! post_serves, default_post_for, op_streams' wire branch),
//! app::posting_family (edm_at_post: wire passes, sinker refused), post::engine
//! (the wireEdm door), post::machine (edm codes mirror, edm_codes), and
//! the wire-iso.cps script, and the wire clause's place in the post
//! gate (posting_wire's wire_clause before NC text, #534).
//! Provenance: our own tests over our own synthetic fixture.

#[path = "edm/sinker_fixture.rs"]
mod sinker_fixture;

use flecs_ecs::prelude::World;

use camcraft_app::commands::{CreateOperation, GenerateOperation, History, ImportModel, LoadMachine, SetStock};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::machine::{parse_mch, Kind};
use camcraft_app::operation::{Height, OpParams, SrcRef, WireContourOp, WireSide};
use camcraft_app::post::{self, cnc};
use camcraft_app::posting::{self, BUILTINS, WIRE_POSTS};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::world::new_world;

const FIXTURE: &str = include_str!("edm/wire-iso-fixture.mch");
const WIRE: &str = "Generic ISO Wire EDM";
const GOLDEN: &str = "tests/golden/nc/wire-iso/wire_slug_punch.nc";

fn cps(name: &str) -> &'static str {
    BUILTINS.iter().find(|(n, _)| *n == name).map(|(_, s)| *s).expect("registered")
}

/// A sharp 40 x 20 slug from z = 0 to 12, centred on the origin.
fn slug() -> Mesh {
    prism(&[(20.0, 10.0), (-20.0, 10.0), (-20.0, -10.0), (20.0, -10.0)])
}

/// A 40 x 20 die opening with 2 mm corners (3-degree facets) from z = 0
/// to 12 - a die cannot cut a sharp inside corner.
fn opening() -> Mesh {
    let (r, hx, hy) = (2.0, 18.0, 8.0);
    let mut pts = Vec::new();
    for (cx, cy, a0) in [(hx, hy, 0.0_f64), (-hx, hy, 90.0), (-hx, -hy, 180.0), (hx, -hy, 270.0)] {
        for s in 0..=30 {
            let t = (a0 + 3.0 * f64::from(s)).to_radians();
            pts.push((cx + r * t.cos(), cy + r * t.sin()));
        }
    }
    prism(&pts)
}

/// The CCW outline `pts` extruded from z = 0 to 12.
fn prism(pts: &[(f64, f64)]) -> Mesh {
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
        burnout: None,
    };
    let c = CreateOperation::new(&world, "Punch1", "", SrcRef::Entity("slug".into()), OpParams::WireContour(op)).expect("op");
    h.execute(&world, c);
    h.execute(&world, GenerateOperation::new(&world, "Punch1").expect("generates"));
    world
}

/// A die cut through the 12 mm deep opening in a 60 x 60 x 20 plate - the
/// wire's kerf enters the plate above the opening. Generated: the generator does not judge the plate; the laws do.
fn gouging_die() -> World {
    let world = new_world();
    let mut h = History::default();
    let m = parse_mch(FIXTURE).expect("the fixture parses");
    h.execute(&world, LoadMachine::new(&world, "wire-iso.mch".into(), Some(m)).expect("mount"));
    h.execute(&world, ImportModel::from_mesh("opening", opening()));
    let stock = SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Location::default()).expect("stock");
    h.execute(&world, stock);
    let op = WireContourOp {
        top_z: Height::fixed(12.0),
        bottom_z: Height::fixed(0.0),
        condition: "C110".into(),
        side: WireSide::Die,
        start: Some([0.0, 0.0]),
        burnout: None,
    };
    let c = CreateOperation::new(&world, "Die1", "", SrcRef::Entity("opening".into()), OpParams::WireContour(op)).expect("op");
    h.execute(&world, c);
    h.execute(&world, GenerateOperation::new(&world, "Die1").expect("generates"));
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

/// Done means (#534 over #533): the wire laws stand between a checked
/// cut and its NC text. The clean punch is judged by them and posts its
/// golden; a die whose kerf burns the plate above its opening streams
/// like any wire cut but is refused by name at the gate, and no NC file
/// is written.
#[test]
fn the_wire_laws_stand_between_the_cut_and_its_nc_text() {
    let world = punch_doc(FIXTURE);
    let clause = posting::wire_clause(&world);
    assert!(clause.problems.is_empty(), "{:?}", clause.problems);
    let line = clause.line.expect("the clean punch is judged, not skipped");
    assert!(line.contains("Punch1 frees") && line.ends_with("(the part)"), "{line}");
    let out = std::env::temp_dir().join(format!("camcraft-wire-laws-ok-{}.nc", std::process::id()));
    let posted = posting::post_script(&world, &out, cps(WIRE)).expect("the judged punch posts");
    let written = std::fs::read_to_string(&out);
    let _ = std::fs::remove_file(&out);
    assert_eq!(written.ok().as_deref(), Some(posted.program.as_str()), "the program is written");
    assert_eq!(posted.program, std::fs::read_to_string(GOLDEN).expect("golden present"), "the golden holds");
    assert!(posted.not_judged.is_some_and(|n| n.contains(posting::WIRE_VERDICT)), "said, not silent");

    // A die through a 12 mm deep opening in a 20 mm plate: above
    // the opening the vertical wire burns the plate itself (#534's own
    // gouge), now in front of a real wire post.
    let world = gouging_die();
    posting::op_streams(&world).expect("the gouging cut still streams - the laws, not the stream, refuse it");
    let out = std::env::temp_dir().join(format!("camcraft-wire-laws-gouge-{}.nc", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let err = posting::post_script(&world, &out, cps(WIRE)).expect_err("a gouging wire cut never posts");
    let wrote = out.exists();
    let _ = std::fs::remove_file(&out);
    assert!(err.starts_with("post blocked") && err.contains("Die1: wire gouge: the kerf of move"), "{err}");
    assert!(!wrote, "no NC text is written for a refused wire cut");
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
