//! #4045: the shipped Sodick AQ600L wire EDM machine file and the Sodick
//! LN2W post on it. Pins the machine file's figures and its default post,
//! the EDM test parts (square, 2-axis) posted with the LN2W dialect as
//! goldens under `tests/golden/nc/sodick-ln2w/`, and the taper test part
//! refused by name (#4075).
//! Provenance: our own tests over our own machine file and our own test
//! parts (`testdata/models/edm/`, made for this project); the figures and
//! their sources are in docs/decisions/4045-sodick-ln2w-dialect.md.
//! Gate map (#4045): covers app::posting (MACHINES, WIRE_POSTS,
//! post_script), app::machine (parse_mch, library), sodick-ln2w.cps.

use std::collections::BTreeMap;

use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, ImportModel, LoadMachine, SetPostScriptOptions, SetStock};
use camcraft_app::post::options::OptionValue;
use camcraft_app::schema::PostRef;
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::{feature, Point3};
use camcraft_app::machine::{library, parse_mch, Kind};
use camcraft_app::model;
use camcraft_app::operation::{self, FeatureSel, GeoSel, Height, OpParams, SrcRef, WireContourOp, WireSide, WireSkim};
use camcraft_app::path_picks::{PathPick, PathPicks};
use camcraft_app::posting;
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;

const NAME: &str = "Sodick AQ600L";
const LN2W: &str = "Sodick Wire EDM (LN2W)";
const MARK21: &str = "Sodick Wire EDM (Mark 21)";
const GOLDENS: &str = "tests/golden/nc/sodick-ln2w";

fn text() -> &'static str {
    posting::builtin_machine_text(NAME).expect("the AQ600L is shipped")
}

fn wire(side: WireSide, start: (f64, f64), top: f64, taper_deg: f64, skims: &[&str], sharp_corners: bool) -> WireContourOp {
    WireContourOp {
        top_z: Height::fixed(top),
        bottom_z: Height::fixed(0.0),
        condition: "C001".into(),
        side,
        picks: PathPicks { start: vec![PathPick::xy(Point2::new(start.0, start.1))], exit: Vec::new() },
        tabs: None,
        capture: true,
        both_pieces_restrained: false,
        stock_centered_in_xy_travel: false,
        taper_deg,
        four_axis: None,
        burnout: None,
        clearance: operation::DEFAULT_CLEARANCE,
        skims: skims.iter().map(|c| WireSkim { condition: (*c).into(), reverse: false }).collect(),
        sharp_corners,
        lead_arc_radius: None,
        compensation: Default::default(),
        overlap: 0.0,
    }
}

/// The edge chains of the part at height `z` whose every point is `on`
/// the wanted profile: one closed profile, as the user picks it.
fn profile(world: &World, name: &str, z: f64, on: impl Fn(f64, f64) -> bool) -> Vec<GeoSel> {
    let e = model::find_by_name(world, name).expect("part");
    let f = model::features_of(world, &e).expect("features");
    let picked: Vec<GeoSel> = feature::chains(&f)
        .into_iter()
        .filter(|c| {
            let p = feature::chain_points(&f, c);
            p.len() >= 2 && p.iter().all(|q| (q[2] - z).abs() < 1e-6 && on(q[0], q[1]))
        })
        .map(|c| GeoSel { entity: name.into(), feature: FeatureSel::Chain(c.id) })
        .collect();
    assert!(!picked.is_empty(), "the profile is on the part");
    picked
}

fn post(world: &World, script: &str, tag: &str) -> Result<String, String> {
    let cps = posting::builtin_post_text(script).expect("registered");
    let out = std::env::temp_dir().join(format!("camcraft-aq600l-{tag}-{}.nc", std::process::id()));
    let posted = posting::post_script(world, &out, cps);
    let _ = std::fs::remove_file(&out);
    posted.map(|p| p.program).map_err(|e| format!("{e:?}"))
}

/// The AQ600L mounted, the test part `step` imported with a stock box
/// round it, and one picked profile cut by `op` and generated.
fn cut(step: &str, stock: (f64, f64), top: f64, on: impl Fn(f64, f64) -> bool, op: WireContourOp) -> World {
    let world = new_world();
    let mut h = History::default();
    let m = parse_mch(text()).expect("parses");
    h.execute(&world, LoadMachine::new(&world, "sodick-aq600l.mch".into(), Some(m)).expect("mount"));
    h.execute(&world, AddTool::new(&world, "EDM-W025", camcraft_app::wire_tools::wire_tool(0.25)).expect("wire"));
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models/edm").join(step);
    let imp = ImportModel::new(&path).expect("the test part imports");
    let name = imp.name.clone();
    h.execute(&world, imp);
    let (dx, dy) = stock;
    let loc = Location { pos: Point3::new(0.0, 0.0, 0.0), ..Default::default() };
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx, dy, dz: top }, loc).expect("stock"));
    let src = SrcRef::Selection(profile(&world, &name, top, on));
    h.execute(&world, CreateOperation::new(&world, "Cut1", "EDM-W025", src, OpParams::WireContour(op)).expect("the op is made"));
    h.execute(&world, GenerateOperation::new(&world, "Cut1").expect("the cut generates"));
    world
}

fn within(x: (f64, f64), y: (f64, f64)) -> impl Fn(f64, f64) -> bool {
    move |px, py| px > x.0 - 0.01 && px < x.1 + 0.01 && py > y.0 - 0.01 && py < y.1 + 0.01
}

/// EDM-T1: the 10 mm square opening, a rough cut and two skims.
fn square() -> World {
    let op = wire(WireSide::Die, (0.0, 0.0), 10.0, 0.0, &["C002", "C903"], true);
    cut("edm-test-square.step", (40.0, 40.0), 10.0, within((-5.0, 5.0), (-5.0, 5.0)), op)
}

/// EDM-T3: the outer walls, 44 x 24 at the top and 50 x 30 at the bottom
/// over 20 mm: 3 mm a side, atan(3 / 20) degrees.
fn taper() -> World {
    let a = (3.0f64 / 20.0).atan().to_degrees();
    let wall = |x: f64, y: f64| (x.abs() - 22.0).abs() < 1e-6 || (y.abs() - 12.0).abs() < 1e-6;
    cut("edm-test-taper.step", (60.0, 40.0), 20.0, wall, wire(WireSide::Punch, (-35.0, 0.0), 20.0, a, &[], false))
}

fn golden(name: &str, text: &str) {
    let file = format!("{GOLDENS}/{name}");
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        std::fs::create_dir_all(GOLDENS).expect("golden dir");
        std::fs::write(&file, text).expect("write golden");
    }
    assert_eq!(text, std::fs::read_to_string(&file).expect("golden present"), "{file} drifted");
}

/// The program's executable lines (no comment lines).
fn words(text: &str) -> Vec<&str> {
    text.lines().filter(|l| !l.starts_with('(')).collect()
}

fn find_axis<'a>(parts: &'a serde_json::Value, id: &str) -> Option<&'a serde_json::Value> {
    for p in parts.as_array()? {
        if p["name"] == id {
            return Some(&p["travel"]);
        }
        if let Some(t) = find_axis(&p["parts"], id) {
            return Some(t);
        }
    }
    None
}

#[test]
fn the_aq600l_file_states_its_figures_and_names_the_ln2w_post() {
    let m = parse_mch(text()).expect("the file parses");
    assert_eq!(m.post.as_deref(), Some(LN2W), "the default post is the LN2W dialect");
    let raw: serde_json::Value = serde_json::from_str(text()).expect("json");
    // Review of PR #4065: U/V 80 x 80 mm, Sodick's AQ400L / AQ600L brochure.
    for (axis, span) in [("X", 600.0), ("Y", 400.0), ("U", 80.0), ("V", 80.0)] {
        let hit = find_axis(&raw["kinematics"]["default"]["parts"], axis).unwrap_or_else(|| panic!("{axis}"));
        let (lo, hi) = (hit["min"].as_f64().expect("min"), hit["max"].as_f64().expect("max"));
        assert!((hi - lo - span).abs() < 1e-9, "{axis}: {lo}..{hi}");
    }
    let edm = m.edm.as_ref().expect("edm");
    assert_eq!(edm.max_taper_deg, Some(20.0));
    // A 20 degree lean over the example guide span stays inside the U/V half travel.
    let span = edm.upper_guide_z.expect("upper") - edm.lower_guide_z.expect("lower");
    assert!(span * 20.0f64.to_radians().tan() <= 40.0, "guide span {span} mm leans past the 40 mm U/V half travel at 20 deg");
    // No unconfirmed corner or side rule is stated (#4075).
    assert!(edm.taper_angle.is_none(), "no edm.taper_angle until the LN2W corner motion and lean side are confirmed");
    assert_eq!(edm.awt, Some(true));
    // What is an example says so, as the ALC400G file does (#3214).
    let about = raw["general"]["description"].as_str().expect("description");
    assert!(about.contains("EXAMPLE") && about.contains("replace") && about.contains("CHECK"), "{about}");
    for c in raw["edm"]["conditions"].as_array().expect("conditions") {
        assert!(c["notes"].as_str().unwrap_or_default().starts_with("EXAMPLE"), "{c}");
    }
    for key in ["wire_diameters", "codes"] {
        assert!(raw["edm"].get(key).is_none(), "edm.{key} has no public source");
    }
}

#[test]
fn the_library_lists_the_aq600l_and_offers_both_sodick_posts() {
    let e = library::builtins().into_iter().find(|e| e.name == NAME).expect("listed");
    assert!(e.is_valid(), "{:?}", e.refusals);
    assert_eq!(e.kind, Some(Kind::WireEdm));
    assert!(e.posts.contains(&LN2W) && e.posts.contains(&MARK21), "{:?}", e.posts);
}

/// Done means: the square posts in the owner's start order (U0. V0. on the
/// G00 to the start point, his round-2 correction on PR #4065), with no
/// unit word, and each skim takes T85 and its own C word.
#[test]
fn the_ln2w_square_posts_its_golden_in_the_owners_start_order() {
    let text = post(&square(), LN2W, "square").expect("the square posts");
    golden("edm_test_square.nc", &text);
    let w = words(&text);
    assert_eq!(&w[..9], ["G90", "G54", "G00 X0. Y0. U0. V0.", "Z0.", "T91", "T84", "T94", "C001", "G01 X4.835 Y0."], "{text}");
    assert_eq!(w.iter().filter(|l| l.contains("U0.")).count(), 1, "U0. V0. rides the one G00 only: {text}");
    assert!(!w.iter().any(|l| l.split(' ').any(|x| matches!(x, "G20" | "G21" | "G40" | "G92"))), "{text}");
    assert!(text.contains("NOT VERIFIED ON A CONTROL"), "{text}");
    let skims: Vec<usize> = w.iter().enumerate().filter(|(_, l)| **l == "T85").map(|(i, _)| i).collect();
    assert_eq!(skims.len(), 2, "{text}");
    assert_eq!((w[skims[0] + 1], w[skims[1] + 1]), ("C002", "C903"), "{text}");
    assert_eq!(&w[w.len() - 2..], ["T90", "M02"], "{text}");
}

/// Done means: a chain of holes of the 2-axis test block posts its golden,
/// its circles as G02 / G03 arcs, with no taper word.
#[test]
fn the_ln2w_two_axis_block_posts_its_golden() {
    let op = wire(WireSide::Die, (-13.0, -3.0), 20.0, 0.0, &[], true);
    let world = cut("edm-test-2axis.step", (60.0, 40.0), 20.0, within((-24.0, -4.0), (-6.0, 0.0)), op);
    let text = post(&world, LN2W, "2axis").expect("the 2-axis block posts");
    golden("edm_test_2axis.nc", &text);
    let w = words(&text);
    assert_eq!(&w[2..8], ["G00 X-13. Y-3. U0. V0.", "Z0.", "T91", "T84", "T94", "C001"], "{text}");
    assert!(w[8].starts_with("G01 "), "the first cut follows the C word: {text}");
    assert!(w.iter().any(|l| l.starts_with("G03 ")) && w.iter().any(|l| l.starts_with("G02 ")), "{text}");
    assert!(!w.iter().any(|l| l.starts_with("TP") || l.starts_with("TN") || matches!(*l, "G50" | "G51" | "G52")), "{text}");
}

/// #4075: TP is the part's bottom above the worktable, and CamCraft knows
/// no fixture. Without the post option the taper test part is refused by
/// name, with no program. No taper word is written from a guess.
#[test]
fn the_ln2w_taper_part_is_refused_by_name_without_the_table_height() {
    let err = post(&taper(), LN2W, "taper-unstated").expect_err("no taper is posted");
    assert!(err.contains("Cut1") && err.contains("is tapered") && err.contains("partBottomAboveTable") && err.contains("#4075"), "{err}");
}

/// #4075 Done means: with the table height stated (the post option, set by
/// the undoable command), the taper test part posts in the owner's 4-axis
/// form: the start order of #4065, TP TN G74 after the C word, U V on
/// every cut, G75 before T90, no G51 / G52. The walls lean 3 mm over the
/// 20 mm part: each leaning move's U or V is 3 mm (the punch is 50 x 30 at
/// the bottom, 44 x 24 at the top, with the wire's kerf offset).
#[test]
fn the_ln2w_taper_part_posts_as_a_four_axis_program_with_the_table_height() {
    let world = taper();
    let post_ref = PostRef::Named(LN2W.into());
    let options = BTreeMap::from([("partBottomAboveTable".to_owned(), OptionValue::Float(5.0))]);
    History::default().execute(&world, SetPostScriptOptions::for_post(&world, &post_ref, options).expect("the option is set"));
    // Posted as the named post, so the document's stored option applies.
    let cps = posting::builtin_post_text(LN2W).expect("registered");
    let out = std::env::temp_dir().join(format!("camcraft-aq600l-taper-{}.nc", std::process::id()));
    let posted = posting::post_script_for_post_with_options(&world, &out, &post_ref, cps, None);
    let _ = std::fs::remove_file(&out);
    let text = posted.map(|p| p.program).expect("the taper posts");
    golden("edm_test_taper.nc", &text);
    // The section note names the work Z that TP stands for (PR #4171).
    assert!(text.contains("TP 5. IS WORK Z") && text.contains("ABOVE THE WORKTABLE - THE POST OPTION STATES WORK Z"), "{text}");
    let w = words(&text);
    assert_eq!(&w[2..11], ["G00 X-35. Y0. U0. V0.", "Z0.", "T91", "T84", "T94", "C001", "TP5.", "TN25.", "G74"], "{text}");
    let cuts: Vec<&str> = w.iter().copied().filter(|l| l.starts_with("G01 ")).collect();
    assert!(!cuts.is_empty() && cuts.iter().all(|l| l.contains(" U") && l.contains(" V")), "U V on every cut: {text}");
    assert!(cuts.iter().any(|l| l.contains("U3.") || l.contains("U-3.") || l.contains("V3.") || l.contains("V-3.")), "a 3 mm lean over 20 mm: {text}");
    assert_eq!(&w[w.len() - 3..], ["G75", "T90", "M02"], "{text}");
    assert!(!w.iter().any(|l| l.split(' ').any(|x| matches!(x, "G50" | "G51" | "G52" | "G02" | "G03"))), "{text}");
}

/// The Mark 21 dialect is not changed by this work: on the same machine
/// it still writes its own words, and it still refuses the taper.
#[test]
fn the_mark_21_post_keeps_its_own_words_on_the_aq600l() {
    let text = post(&square(), MARK21, "square-mark21").expect("the square posts on Mark 21");
    let w = words(&text);
    assert_eq!(&w[..7], ["G21", "G90", "G54", "G40 G00 X0. Y0.", "T91", "T84", "T94"], "{text}");
    assert!(!w.contains(&"Z0.") && !w.iter().any(|l| l.contains("U0.")), "{text}");
    let err = post(&taper(), MARK21, "taper-mark21").expect_err("Mark 21 writes a vertical cut only");
    assert!(err.contains("angle-word move") || err.contains("is tapered"), "{err}");
}

#[path = "sodick_aq600l_offset.rs"]
mod offset;
