//! #3012: pattern folders at the post - by tool (each member cuts every
//! enabled copy in one section, one tool change) and by part (every
//! member at one copy, then the next), pinned as Fanuc goldens through
//! the gated post; a refusal by name; and the five MCP tools with
//! `get_job`, reorder, rename, undo, redo, save and open.
//! Provenance: our own post law over RS-274 rapids (see
//! `posting_pattern`).
//!
//! Gate map: covers app::posting (op_streams, posting_pattern,
//! verification_report), app::mcp::pattern_tools, app::mcp::order_tools,
//! app::mcp::job, assets/posts/fanuc.cps.

use super::pattern_folder::{grid, linear, names, three_faces};
use super::{tmp_dir, Client};
use camcraft_app::commands::{CreatePatternFolder, History, SetPatternInstanceEnabled};
use camcraft_app::posting;
use camcraft_app::schema::PatternOrder;
use flecs_ecs::prelude::World;
use serde_json::{json, Value};

fn has_word(line: &str, code: &str) -> bool {
    line.split_whitespace().any(|w| w.trim_end_matches('*') == code)
}

fn tool_changes(program: &str) -> Vec<String> {
    program
        .lines()
        .filter(|l| has_word(l, "M6") || has_word(l, "M06"))
        .map(|l| l.split_whitespace().find(|w| w.starts_with('T')).unwrap_or("T?").to_owned())
        .collect()
}

fn post_fanuc(world: &World, tag: &str) -> posting::Posted {
    let dir = tmp_dir(tag);
    let cps = posting::builtin_post_text("Fanuc").expect("a built-in");
    let posted = posting::post_script(world, &dir.join("out.nc"), cps).expect("the gate passes and the script runs");
    let _ = std::fs::remove_dir_all(&dir);
    posted
}

fn folder(world: &World, h: &mut History, kind: &camcraft_app::schema::PatternKind, mode: PatternOrder, members: &[&str]) {
    let cmd = CreatePatternFolder::new(world, kind, mode, &names(members)).expect("folder");
    h.execute(world, cmd);
}

#[test]
fn by_tool_posts_each_member_once_over_every_enabled_copy() {
    let (world, mut h) = three_faces();
    let plain = post_fanuc(&world, "pattern-plain");
    folder(&world, &mut h, &linear(3), PatternOrder::ByTool, &["Face2", "Face3"]);
    h.execute(&world, SetPatternInstanceEnabled::new(&world, "Pattern1", 2, false).expect("copy 2 off"));
    let posted = post_fanuc(&world, "pattern-by-tool");
    // Three sections, three tool changes - no change between copies.
    assert_eq!(posted.ops, 3, "one section per operation");
    assert_eq!(tool_changes(&posted.program), tool_changes(&plain.program), "{}", posted.program);
    // Copy 3 stands 80 mm along X; copy 2 (40 mm) is off.
    let xs: Vec<f64> = posted.line_pos.iter().flatten().map(|p| p[0]).collect();
    assert!(xs.iter().any(|x| *x > 70.0), "copy 3 is cut");
    assert!(!xs.iter().any(|x| (25.0..55.0).contains(x)), "copy 2 is not cut");
    assert_eq!(posted.line_pos.len(), posted.lines, "every program line maps");
    assert!(posted.not_judged.as_deref().is_some_and(|l| l.contains("pattern copies (Pattern1)")), "{:?}", posted.not_judged);
    crate::common::write_golden("nc/pattern_folder/fanuc_linear_by_tool.nc", &posted.program);
}

#[test]
fn by_part_posts_every_member_at_one_copy_then_the_next() {
    let (world, mut h) = three_faces();
    folder(&world, &mut h, &grid(), PatternOrder::ByPart, &["Face2", "Face3"]);
    h.execute(&world, SetPatternInstanceEnabled::new(&world, "Pattern1", 4, false).expect("copy 4 off"));
    let posted = post_fanuc(&world, "pattern-by-part");
    // Face1, then (Face2, Face3) at copies 1, 2 and 3.
    assert_eq!(posted.ops, 7, "one section per member and copy");
    assert_eq!(tool_changes(&posted.program).len(), 7, "{}", posted.program);
    crate::common::write_golden("nc/pattern_folder/fanuc_grid_by_part.nc", &posted.program);
    // One file per operation cannot hold the by-part order: refused by name.
    let dir = tmp_dir("pattern-per-op");
    let cps = posting::builtin_post_text("Fanuc").expect("a built-in");
    let err = posting::post_per_op_script(&world, &dir, "1001", cps).err().expect("per-op files refused");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(err.contains("Face2") && err.contains("by-part pattern folder"), "{err}");
}

/// Generate every op in program order until all are fresh (an order
/// or offset change stales the ops that read the program ahead).
fn regenerate(world: &World, h: &mut History) {
    for _ in 0..3 {
        for (op, e) in camcraft_app::setup::operation_order(world) {
            if camcraft_app::operation::status_of(world, &e) != camcraft_app::operation::OpStatus::Ok {
                h.execute(world, camcraft_app::commands::GenerateOperation::new(world, &op).expect("regenerate"));
            }
        }
    }
    let stale: Vec<String> = camcraft_app::setup::operation_order(world)
        .into_iter()
        .filter(|(_, e)| camcraft_app::operation::status_of(world, e) != camcraft_app::operation::OpStatus::Ok)
        .map(|(n, _)| n)
        .collect();
    assert!(stale.is_empty(), "all fresh: {stale:?}");
}

#[test]
fn a_folder_the_post_cannot_write_refuses_by_name() {
    let (world, mut h) = three_faces();
    folder(&world, &mut h, &linear(2), PatternOrder::ByTool, &["Face1", "Face2"]);
    assert!(posting::stream_records(&world).is_ok(), "the folder posts");
    // A hand-edited file puts Face3 between the members: refused, never
    // posted in an order the list does not show.
    let face3 = camcraft_app::model::find_by_name(&world, "Face3").expect("op");
    let mut meta = camcraft_app::model::operation_meta_of(&world, &face3);
    meta.order = Some(1);
    face3.entity_view(&world).set(meta);
    let face2 = camcraft_app::model::find_by_name(&world, "Face2").expect("op");
    let mut meta2 = camcraft_app::model::operation_meta_of(&world, &face2);
    meta2.order = Some(5);
    face2.entity_view(&world).set(meta2);
    regenerate(&world, &mut h);
    let err = posting::stream_records(&world).expect_err("out of its block");
    assert!(err.contains("Pattern1") && err.contains("directly after it"), "{err}");

    // A turned WCS: the copies would go along the wrong axes.
    let (world, mut h) = three_faces();
    folder(&world, &mut h, &linear(2), PatternOrder::ByTool, &["Face1"]);
    let yaw = camcraft_app::geometry::quat_from_euler(0.0, 0.0, 90f64.to_radians());
    let loc = camcraft_app::schema::Location { pos: camcraft_app::geometry::Point3::new(0.0, 0.0, 0.0), rot: yaw };
    h.execute(&world, camcraft_app::commands::SetWcsOrigin::new(&world, loc).expect("turn G54"));
    regenerate(&world, &mut h);
    let err = posting::stream_records(&world).expect_err("turned WCS");
    assert!(err.contains("Pattern1") && err.contains("WCS is turned"), "{err}");
}

fn job_folder(c: &mut Client) -> Value {
    let job = c.ok("get_job", json!({}));
    job["pattern_folders"][0].clone()
}

#[test]
fn mcp_tools_create_edit_reorder_undo_save_and_open_a_folder() {
    let mut c = Client::new();
    c.ok("set_stock", json!({ "box": { "dx": 30.0, "dy": 30.0, "dz": 20.0 } }));
    c.ok("add_tool", json!({ "catalog": "EM-1000-4FL" }));
    for name in ["F1", "F2", "F3"] {
        c.ok("add_operation", json!({ "kind": "Face", "tool": "EM-1000-4FL", "name": name }));
    }
    assert!(c.ok("get_job", json!({})).get("pattern_folders").is_none(), "no folder, no key");
    let made = c.ok(
        "create_pattern_folder",
        json!({ "pattern": { "type": "linear", "dir": [0, 1], "count": 3, "spacing": 45 }, "members": ["F2", "F3"] }),
    );
    assert_eq!(made["key"], json!("Pattern1"));
    assert_eq!(made["order"], json!("by_tool"));
    assert_eq!(made["copies"], json!(3));
    assert_eq!(made["members"], json!(["F2", "F3"]));
    assert_eq!(job_folder(&mut c), made, "get_job lists the folder");
    let msg = c.refused("create_pattern_folder", json!({ "pattern": { "type": "linear", "dir": [1, 0], "count": 2, "spacing": 5 }, "members": ["F3"] }));
    assert!(msg.contains("already in pattern folder"), "{msg}");
    let msg = c.refused("create_pattern_folder", json!({ "pattern": { "type": "ring" } }));
    assert!(msg.contains("linear"), "{msg}");

    let grid = c.ok(
        "edit_pattern_folder",
        json!({ "name": "Pattern1", "order": "by_part",
                "pattern": { "type": "grid", "dir1": [1, 0], "count1": 2, "spacing1": 40, "dir2": [0, 1], "count2": 2, "spacing2": 40 } }),
    );
    assert_eq!(grid["copies"], json!(4));
    assert_eq!(grid["order"], json!("by_part"));
    let off = c.ok("set_pattern_instance_enabled", json!({ "name": "Pattern1", "instance": 3, "enabled": false }));
    assert_eq!(off["disabled"], json!([3]));
    let renamed = c.ok("rename_operation", json!({ "name": "Pattern1", "new_name": "Vise row" }));
    assert_eq!(renamed["name"], json!("Vise row"));
    c.ok("reorder_operation", json!({ "name": "Vise row", "target": "F1", "place": "before" }));
    let members = c.ok("set_pattern_members", json!({ "name": "Vise row", "members": ["F3"] }));
    assert_eq!(members["members"], json!(["F3"]));
    let job = c.ok("get_job", json!({}));
    let order: Vec<&str> = job["operations"].as_array().expect("ops").iter().filter_map(|o| o["name"].as_str()).collect();
    assert_eq!(order, ["F3", "F2", "F1"], "the folder stands first, its member after it");

    // Undo the members, the reorder, the rename, the copy and the edit; redo them.
    let full = job_folder(&mut c);
    for _ in 0..5 {
        c.ok("undo", json!({}));
    }
    assert_eq!(job_folder(&mut c)["copies"], json!(3), "back to the linear pattern");
    assert_eq!(job_folder(&mut c)["members"], json!(["F2", "F3"]));
    for _ in 0..5 {
        c.ok("redo", json!({}));
    }
    assert_eq!(job_folder(&mut c), full);

    // Save and open: the same document.
    let dir = tmp_dir("pattern-mcp");
    let path = dir.join("p.camproj");
    c.ok("save_project", json!({ "path": path.display().to_string() }));
    c.ok("open_project", json!({ "path": path.display().to_string() }));
    assert_eq!(job_folder(&mut c), full, "the folder survives save and open");
    let _ = std::fs::remove_dir_all(&dir);

    let deleted = c.ok("delete_pattern_folder", json!({ "name": "Vise row" }));
    assert_eq!(deleted["ungrouped"], json!(["F3"]));
    assert!(c.ok("get_job", json!({})).get("pattern_folders").is_none());
    c.ok("undo", json!({}));
    assert_eq!(job_folder(&mut c), full);
}
