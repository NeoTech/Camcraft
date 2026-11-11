//! #828 acceptance: a machine file names the spindle mount and the
//! holder a NEW tool starts with (`tooling.spindle_mount`,
//! `tooling.default_holder`). A new tool on the 770MX lands with the
//! BT30 ER20, on the 770M with the TTS ER20, in the add's own undo
//! step; a stated holder wins; a machine without the key (or no
//! machine) adds the tool bare, as before; an existing tool and an
//! existing project are never changed; an unknown or mismatched holder
//! in a machine file is refused by name with the file and the key. And
//! the default is judged: the new tool's BT30 flange strikes a wall
//! the bare holder cylinder misses.
//!
//! Fixture law (clean-room): our own values in the `.mch` layout of
//! `assets/machines/AGENTS.md`; the Tormach spindle types from
//! https://www.tormach.com/tormach_mills/.
//! Provenance: our own tests, specced from the issue's "Done means".
//!
//! Gate map (M4.4): covers app::commands::tool, app::commands::tool_body,
//! app::machine (tooling keys), post::machine::tooling.

use camcraft_app::commands::{
    AddTool, CommandError, CreateOperation, GenerateOperation, History, HolderPick, LoadMachine, SetStock, TagFixture,
};
use camcraft_app::geometry::{Point3, Triangle};
use camcraft_app::machine::validate::validate_text;
use camcraft_app::model;
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tool_body;
use camcraft_app::tools::{catalog, Tool};
use camcraft_app::verify::{FixturePart, Warning};
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;

/// A 4 mm end mill on a 6 mm shank: flutes 12, shank 40.
const TOOL: &str = "EM-0400-2FL";
const CUT_Z: f64 = 19.5;

fn catalog_tool(name: &str) -> Tool {
    catalog().into_iter().find(|(n, _)| *n == name).map(|(_, t)| t).expect("catalog entry")
}

/// A shipped machine file by its stem.
fn asset(stem: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/machines").join(format!("{stem}.mch"))
}

fn mounted(machine: Option<&str>) -> (World, History) {
    let world = new_world();
    let mut history = History::default();
    if let Some(m) = machine {
        history.execute(&world, LoadMachine::from_file(&world, &asset(m)).expect("shipped machine"));
    }
    (world, history)
}

/// The library name of the holder `name` carries, read back off its
/// drawn body (the profile the library draws for it).
fn body(world: &World, name: &str) -> Option<tool_body::BodyFrame> {
    let e = model::find_by_name(world, name)?;
    tool_body::body_of(world, &e).map(|b| b.frame)
}

fn holder_d(world: &World, name: &str) -> Option<f64> {
    let e = model::find_by_name(world, name)?;
    let (judged, b) = tool_body::judged(world, &e, catalog_tool(TOOL)).ok()?;
    b.map(|_| judged.holder_d)
}

fn add(world: &World, history: &mut History, pick: HolderPick) -> Option<String> {
    let cmd = AddTool::with_holder(world, TOOL, catalog_tool(TOOL), pick).expect("added");
    let held = cmd.holder().map(str::to_owned);
    history.execute(world, cmd);
    held
}

#[test]
fn a_new_tool_on_the_770mx_starts_with_the_bt30_er20() {
    let (world, mut history) = mounted(Some("tormach-770mx"));
    assert_eq!(add(&world, &mut history, HolderPick::MachineDefault).as_deref(), Some("BT30 ER20"));
    assert!(body(&world, TOOL).is_some(), "the holder landed with the tool");
    assert_eq!(holder_d(&world, TOOL).map(|d| (d * 1e6).round() / 1e6), Some(46.0), "the BT30 flange is its widest");
    // One undo takes the tool and its holder; redo brings both back.
    assert!(history.undo(&world));
    assert!(model::find_by_name(&world, TOOL).is_none(), "undo removes the tool");
    assert!(history.redo(&world));
    assert!(body(&world, TOOL).is_some(), "redo lands the holder again");
}

#[test]
fn a_new_tool_on_the_770m_starts_with_the_tts_er20() {
    let (world, mut history) = mounted(Some("tormach-770m"));
    assert_eq!(add(&world, &mut history, HolderPick::MachineDefault).as_deref(), Some("TTS ER20"));
    assert_eq!(holder_d(&world, TOOL).map(|d| (d * 1e6).round() / 1e6), Some(38.1), "the TTS collar");
}

#[test]
fn without_the_key_or_a_machine_the_tool_is_added_bare() {
    for m in [None, Some("vmc")] {
        let (world, mut history) = mounted(m);
        assert_eq!(add(&world, &mut history, HolderPick::MachineDefault), None, "{m:?}");
        assert!(body(&world, TOOL).is_none(), "{m:?}: no holder body, as before #828");
    }
}

#[test]
fn a_stated_holder_wins_over_the_default() {
    let (world, mut history) = mounted(Some("tormach-770mx"));
    assert_eq!(add(&world, &mut history, HolderPick::Named("bt40 er20".into())).as_deref(), Some("BT40 ER20"));
    let (world, mut history) = mounted(Some("tormach-770mx"));
    assert_eq!(add(&world, &mut history, HolderPick::Bare), None);
    assert!(body(&world, TOOL).is_none(), "\"none\" adds it bare");
    // A stated holder that cannot be is refused, never swapped.
    let (world, _) = mounted(Some("tormach-770mx"));
    let refused = AddTool::with_holder(&world, TOOL, catalog_tool(TOOL), HolderPick::Named("BT30 ER99".into()));
    assert!(matches!(refused, Err(CommandError::Invalid(ref m)) if m.contains("BT30 ER99")));
}

/// A shank the default collet cannot take: the tool lands bare and the
/// note names the holder and why - never another holder in its place.
#[test]
fn a_default_that_cannot_take_the_shank_is_left_off_by_name() {
    let (world, mut history) = mounted(Some("tormach-770m"));
    let mut tool = catalog_tool(TOOL);
    tool.shank_d = 16.0;
    let cmd = AddTool::with_holder(&world, TOOL, tool, HolderPick::MachineDefault).expect("still added");
    assert_eq!(cmd.holder(), None);
    let note = cmd.note().expect("a note");
    assert!(note.contains("TTS ER20") && note.contains("not applied"), "{note}");
    history.execute(&world, cmd);
    assert!(body(&world, TOOL).is_none());
}

#[test]
fn an_existing_tool_is_never_changed_by_a_machine() {
    let (world, mut history) = mounted(None);
    add(&world, &mut history, HolderPick::MachineDefault);
    history.execute(&world, LoadMachine::from_file(&world, &asset("tormach-770mx")).expect("mount"));
    assert!(body(&world, TOOL).is_none(), "mounting a machine leaves the tool's holder alone");
}

/// A project saved before #828 (its machine states no default) loads
/// and saves back byte-identical, and a new tool in it starts bare.
#[test]
fn an_existing_project_loads_unchanged() {
    let (world, mut history) = mounted(Some("tormach-770mx"));
    add(&world, &mut history, HolderPick::Bare);
    let dir = std::env::temp_dir().join(format!("camcraft_828_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dir");
    let path = dir.join("old.ccam");
    camcraft_app::project::save(&world, &path).expect("save");
    // What an older build wrote: the machine without the two keys.
    let saved = std::fs::read_to_string(&path).expect("read");
    assert!(saved.contains("\"default_holder\": \"BT30 ER20\""), "the new build saves the key");
    // Drop the two lines (and the comma they leave behind) from the text.
    let mut kept: Vec<String> = Vec::new();
    for line in saved.lines() {
        let t = line.trim_start();
        if t.starts_with("\"spindle_mount\"") || t.starts_with("\"default_holder\"") {
            if !t.ends_with(',')
                && let Some(prev) = kept.last_mut()
                && prev.ends_with(',')
            {
                prev.pop();
            }
            continue;
        }
        kept.push(line.to_owned());
    }
    let old = kept.join("\n");
    assert!(serde_json::from_str::<serde_json::Value>(&old).is_ok(), "still JSON");
    std::fs::write(&path, &old).expect("write");
    let back = new_world();
    camcraft_app::project::load(&back, &path).expect("an older file loads");
    let again = dir.join("again.ccam");
    camcraft_app::project::save(&back, &again).expect("save again");
    assert_eq!(std::fs::read_to_string(&again).expect("read"), old, "byte-identical");
    assert!(body(&back, TOOL).is_none());
    let mut h = History::default();
    let cmd = AddTool::with_holder(&back, "T2", catalog_tool(TOOL), HolderPick::MachineDefault).expect("add");
    assert_eq!(cmd.holder(), None, "the stored machine states no default");
    h.execute(&back, cmd);
    let _ = std::fs::remove_dir_all(&dir);
}

fn bad_file(tooling: &str) -> String {
    let text = include_str!("../../../assets/machines/tormach-770mx.mch");
    let at = text.find("\"tooling\"").expect("tooling block");
    format!("{}\"tooling\": {tooling}\n}}\n", &text[..at])
}

#[test]
fn an_unknown_or_mismatched_holder_is_refused_by_file_and_key() {
    let dir = std::env::temp_dir().join(format!("camcraft_828_bad_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dir");
    for (tooling, word) in [
        (r#"{ "default_holder": "BT30 ER99" }"#, "BT30 ER99"),
        (r#"{ "spindle_mount": "TTS", "default_holder": "BT30 ER20" }"#, "TTS"),
        (r#"{ "spindle_mount": "R8" }"#, "R8"),
    ] {
        let text = bad_file(tooling);
        let path = dir.join("bad.mch");
        std::fs::write(&path, &text).expect("write");
        let world = new_world();
        let Err(CommandError::Invalid(msg)) = LoadMachine::from_file(&world, &path) else {
            panic!("{tooling}: must be refused");
        };
        assert!(msg.contains("bad.mch") && msg.contains("tooling.") && msg.contains(word), "{msg}");
        // The Machines pane names the same key.
        let r = validate_text(&text);
        assert!(r.iter().any(|r| r.key.starts_with("tooling.") && r.rule.contains(word)), "{r:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_shipped_tormach_mills_state_their_mount() {
    for (m, mount, holder) in [
        ("tormach-770mx", "BT30", "BT30 ER20"),
        ("tormach-770mx-ml", "BT30", "BT30 ER20"),
        ("tormach-770mx-microarc4", "BT30", "BT30 ER20"),
        ("tormach-1100mx", "BT30", "BT30 ER20"),
        ("tormach-1500mx", "BT30", "BT30 ER20"),
        ("tormach-770m", "TTS", "TTS ER20"),
        ("tormach-770m-plus", "TTS", "TTS ER20"),
        ("tormach-1100m", "TTS", "TTS ER20"),
        ("tormach-1100m-plus", "TTS", "TTS ER20"),
        ("tormach-pcnc440", "TTS", "TTS ER20"),
        ("tormach-pcnc770", "TTS", "TTS ER20"),
        ("tormach-pcnc770-s3", "TTS", "TTS ER20"),
        ("tormach-pcnc1100-s1", "TTS", "TTS ER20"),
        ("tormach-pcnc1100-s2", "TTS", "TTS ER20"),
        ("tormach-pcnc1100-s3", "TTS", "TTS ER20"),
    ] {
        let (world, _) = mounted(Some(m));
        let machine = camcraft_app::machine::of(&world).1.expect("mounted");
        assert_eq!((machine.spindle_mount.as_deref(), machine.default_holder.as_deref()), (Some(mount), Some(holder)), "{m}");
    }
}

// ---- the default is judged: the reach pin ----------------------------

fn face_on_770mx(pick: HolderPick) -> (World, History) {
    let (world, mut history) = mounted(Some("tormach-770mx"));
    history.execute(
        &world,
        SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Location::default()).expect("stock"),
    );
    add(&world, &mut history, pick);
    let params = OpParams::Face {
        holes: Default::default(),
        tilt: None,
        blend_turns: false,
        stepover: 3.0,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(CUT_Z),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    };
    history.execute(&world, CreateOperation::new(&world, "Face1", TOOL, SrcRef::Stock, params).expect("face op"));
    history.execute(&world, GenerateOperation::new(&world, "Face1").expect("generate"));
    // A wall 22.5 mm off the outermost pass, 95-105 mm above the cut:
    // beside the BT30 flange (90-110 above the tip at its default
    // gauge), above the catalog's holder cylinder.
    let op = model::find_by_name(&world, "Face1").expect("Face1");
    let cache = model::toolpath_of(&world, &op).expect("generated");
    let xmax = cache.path.moves.iter().flat_map(|m| [m.from.x, m.to.x]).fold(f64::NEG_INFINITY, f64::max);
    let (y0, y1) =
        cache.path.moves.iter().flat_map(|m| [m.from.y, m.to.y]).fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), y| (a.min(y), b.max(y)));
    let (x, zb, h) = (xmax + 22.5, CUT_Z + 95.0, 10.0);
    let (a, b, c, d) = (Point3::new(x, y0, zb), Point3::new(x, y1, zb), Point3::new(x, y1, zb + h), Point3::new(x, y0, zb + h));
    let e = model::spawn(&world, "Wall1");
    model::set_facets(&world, &e, vec![Triangle::new(a, b, c), Triangle::new(a, c, d)].into());
    model::set_rev(&world, &e, 1);
    history.execute(&world, TagFixture::new(&world, "Wall1", None).expect("tagged"));
    history.execute(&world, GenerateOperation::new(&world, "Face1").expect("regenerate"));
    (world, history)
}

fn holder_hits(world: &World) -> usize {
    let op = model::find_by_name(world, "Face1").expect("Face1");
    let cache = model::toolpath_of(world, &op).expect("generated");
    cache.warnings.iter().filter(|w| matches!(w, Warning::FixtureHit { fixture_part: FixturePart::Holder, .. })).count()
}

/// The default makes the check stricter, never looser: the new tool's
/// BT30 flange strikes the wall the bare holder cylinder misses.
#[test]
fn the_default_holder_is_judged_by_the_reach_laws() {
    let (bare, _) = face_on_770mx(HolderPick::Bare);
    assert_eq!(holder_hits(&bare), 0, "the bare holder cylinder misses the wall");
    let (held, _) = face_on_770mx(HolderPick::MachineDefault);
    assert!(holder_hits(&held) > 0, "the default BT30 ER20 flange strikes it");
}
