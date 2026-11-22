//! Tests for the built-in 3D multi-setup stock-simulation movie renderer
//! (`sim_video`), `render_simulation_video` MCP tool, live approval gate,
//! JS script call, and `camcraft-cli render-video` subcommand.
//!
//! All tests use `format: "frames"` (pure-Rust PNG encoding) so the suite is
//! fully hermetic with zero external `ffmpeg` dependency.
//!
//! Gate map: covers app::sim_video, app::cli_video, app::mcp::video.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

use camcraft_app::commands::{
    ActivateSetup, AddTool, AlignPartToSetup, CreateOperation, CreateSetup, GenerateOperation, History, ImportModel,
    SetStock,
};
use camcraft_app::geometry::{Mesh, Point3, Triangle};
use camcraft_app::schema::Location;
use camcraft_app::mcp::{requires_approval, GateKind, Server};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::project;
use camcraft_app::script;
use camcraft_app::sim_video::{render_simulation_video, CameraView, VideoFormat, VideoOptions};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;

fn scratch_dir(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("camcraft-sim-video-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    p
}

fn read_png_magic_and_dims(path: &Path) -> (u32, u32) {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert!(bytes.len() > 24, "tiny png {}", path.display());
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "valid PNG header");
    let w = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    (w, h)
}

fn facing_world() -> flecs_ecs::prelude::World {
    let world = new_world();
    let mut history = History::default();
    let stock = SetStock::new(&world, StockShape::Box { dx: 40.0, dy: 40.0, dz: 16.0 }, Default::default())
        .expect("box stock");
    history.execute(&world, stock);
    let tool = catalog()
        .into_iter()
        .find(|(n, _)| *n == "EM-0400-2FL")
        .map(|(_, t)| t)
        .expect("catalog tool");
    history.execute(&world, AddTool::new(&world, "EM-0400-2FL", tool).expect("add tool"));
    let params = OpParams::Face {
        holes: Default::default(),
        blend_turns: false,
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::fixed(22.0),
        cut_z: Height::fixed(14.0),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    };
    let op = CreateOperation::new(&world, "Face1", "EM-0400-2FL", SrcRef::Stock, params).expect("create op");
    history.execute(&world, op);
    world
}

fn rpc_call(srv: &mut Server, id: i64, name: &str, args: Value) -> Value {
    let reply = srv
        .handle(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": { "name": name, "arguments": args }
        }))
        .expect("rpc reply");
    reply["result"].clone()
}

#[test]
fn single_setup_frames_render_valid_pngs_and_removes_stock() {
    let out = scratch_dir("single");
    let proj = out.with_extension("camproj");
    let world = facing_world();
    project::save(&world, &proj).expect("save");
    let loaded = script::run(&format!(
        "cam.load({}); cam.generate(); cam.call(\"render_simulation_video\", {});",
        json!(proj.display().to_string()),
        json!({
            "output": out.display().to_string(),
            "format": "frames",
            "width": 160,
            "height": 90,
            "fps": 10,
            "cell_mm": 2.0,
            "seconds": 0.4,
            "spin_seconds": 0.0,
            "transition_seconds": 0.0,
            "view": "iso"
        })
    ));
    let _ = std::fs::remove_file(&proj);
    assert!(loaded.ok, "script errors: {:?}", loaded.errors);
    let r = &loaded.result;
    assert_eq!(r["format"], "frames");
    assert_eq!(r["width"], 160);
    assert_eq!(r["height"], 90);
    let total = r["total_frames"].as_u64().expect("total_frames");
    assert!(total >= 1, "wrote frames: {r}");
    let init = r["initial_cm3"].as_f64().unwrap();
    let rem = r["remaining_cm3"].as_f64().unwrap();
    assert!(rem < init, "facing cut material: init={init}, rem={rem}");
    let frame1 = out.join("frame_00001.png");
    assert!(frame1.exists(), "frame_00001.png missing");
    assert_eq!(read_png_magic_and_dims(&frame1), (160, 90));
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn multi_setup_side_on_carries_stock_and_supports_per_setup_and_filter() {
    let dir = scratch_dir("multi");
    std::fs::create_dir_all(&dir).expect("mkdir");
    let proj = dir.join("multi.camproj");
    let all_dir = dir.join("all");
    let s3_dir = dir.join("s3_only");
    let kf_dir = dir.join("kf");

    // Build a 3-setup box job centered around origin:
    //   billet 40×40×20 mm (z in -10..10, 32 cm^3), part 32×32×14 mm (14.34 cm^3).
    //   OP10_Top faces Z_doc = +7 (removes top 3 mm -> 27.2 cm^3)
    //   OP20_Flip (180° X) faces Z_setup = +7 (= Z_doc -7, removes bottom 3 mm -> 22.4 cm^3)
    //   OP30_Side (-90° X) faces Z_setup = +16 (= Y_doc -16, removes side 4 mm -> 19.8 cm^3)
    let world = new_world();
    let mut history = History::default();
    let p = |x, y, z| Point3::new(x, y, z);
    let (x0, x1, y0, y1, z0, z1) = (-16.0, 16.0, -16.0, 16.0, -7.0, 7.0);
    let v = [
        p(x0, y0, z0), p(x1, y0, z0), p(x1, y1, z0), p(x0, y1, z0),
        p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1),
    ];
    let tris: Vec<Triangle> = [
        (0, 2, 1), (0, 3, 2), (4, 5, 6), (4, 6, 7), (0, 1, 5), (0, 5, 4),
        (2, 3, 7), (2, 7, 6), (0, 4, 7), (0, 7, 3), (1, 2, 6), (1, 6, 5),
    ]
    .into_iter()
    .map(|(a, b, c)| Triangle::new(v[a], v[b], v[c]))
    .collect();
    history.execute(&world, ImportModel::from_mesh("part", Mesh::from_facets(tris)));
    let tool = catalog()
        .into_iter()
        .find(|(n, _)| *n == "EM-0400-2FL")
        .map(|(_, t)| t)
        .expect("catalog tool");
    history.execute(&world, AddTool::new(&world, "EM-0400-2FL", tool).expect("add tool"));

    let face = |cut_z: f64, clear_z: f64| OpParams::Face {
        holes: Default::default(),
        blend_turns: false,
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::fixed(clear_z),
        cut_z: Height::fixed(cut_z),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    };

    let at_flat = Location { pos: Point3::new(0.0, 0.0, -10.0), ..Default::default() };
    history.execute(&world, CreateSetup::new(&world, Some("OP10_Top"), None).expect("op10"));
    history.execute(&world, ActivateSetup::new(&world, Some("OP10_Top")).expect("act1"));
    history.execute(
        &world,
        SetStock::new(&world, StockShape::Box { dx: 40.0, dy: 40.0, dz: 20.0 }, at_flat).expect("s1"),
    );
    history.execute(
        &world,
        CreateOperation::new(&world, "FaceTop", "EM-0400-2FL", SrcRef::Stock, face(7.0, 16.0)).expect("f1"),
    );

    history.execute(&world, CreateSetup::new(&world, Some("OP20_Flip"), None).expect("op20"));
    history.execute(&world, ActivateSetup::new(&world, Some("OP20_Flip")).expect("act2"));
    history.execute(&world, AlignPartToSetup::angles(&world, "OP20_Flip", [180.0, 0.0, 0.0]).expect("flip"));
    history.execute(
        &world,
        SetStock::new(&world, StockShape::Box { dx: 40.0, dy: 40.0, dz: 20.0 }, at_flat).expect("s2"),
    );
    history.execute(
        &world,
        CreateOperation::new(&world, "FaceBot", "EM-0400-2FL", SrcRef::Stock, face(7.0, 16.0)).expect("f2"),
    );

    let at_side = Location { pos: Point3::new(0.0, 0.0, -20.0), ..Default::default() };
    history.execute(&world, CreateSetup::new(&world, Some("OP30_Side"), None).expect("op30"));
    history.execute(&world, ActivateSetup::new(&world, Some("OP30_Side")).expect("act3"));
    history.execute(
        &world,
        AlignPartToSetup::angles(&world, "OP30_Side", [-90.0, 0.0, 0.0]).expect("side"),
    );
    history.execute(
        &world,
        SetStock::new(&world, StockShape::Box { dx: 40.0, dy: 20.0, dz: 40.0 }, at_side).expect("s3"),
    );
    history.execute(
        &world,
        CreateOperation::new(&world, "FaceSide", "EM-0400-2FL", SrcRef::Stock, face(16.0, 26.0)).expect("f3"),
    );

    for name in ["FaceTop", "FaceBot", "FaceSide"] {
        let g = GenerateOperation::new(&world, name).unwrap_or_else(|e| panic!("gen {name}: {e}"));
        history.execute(&world, g);
    }
    project::save(&world, &proj).expect("save multi");

    let mut srv = Server::new();
    let list = srv
        .handle(&json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {} }))
        .expect("list");
    assert!(
        list["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "render_simulation_video"),
        "render_simulation_video listed in MCP tools/list"
    );
    let opened = rpc_call(&mut srv, 2, "open_project", json!({ "path": proj.display().to_string() }));
    assert_eq!(opened["isError"], false, "open_project: {opened}");

    let full = rpc_call(
        &mut srv,
        3,
        "render_simulation_video",
        json!({
            "output": all_dir.display().to_string(),
            "format": "frames",
            "per_setup": true,
            "width": 120,
            "height": 80,
            "fps": 10,
            "cell_mm": 2.0,
            "seconds": 0.6,
            "transition_seconds": 0.2,
            "spin_seconds": 0.2,
            "keyframes_dir": kf_dir.display().to_string()
        }),
    );
    assert_eq!(full["isError"], false, "full render failed: {full}");
    let sc = &full["structuredContent"];
    let setups = sc["setups"].as_array().unwrap();
    assert_eq!(setups.len(), 3);
    assert_eq!(sc["per_setup_files"].as_array().unwrap().len(), 3);
    for (si, s) in setups.iter().enumerate() {
        let dir_path = PathBuf::from(s["file"].as_str().expect("per-setup dir"));
        let count = std::fs::read_dir(&dir_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", dir_path.display()))
            .filter(|e| e.as_ref().is_ok_and(|x| x.path().extension().is_some_and(|ex| ex == "png")))
            .count();
        let expected = s["frames"].as_u64().unwrap() as usize;
        assert_eq!(count, expected, "setup {si} png frame count matches report");
    }
    let init = sc["initial_cm3"].as_f64().unwrap();
    let r1 = setups[0]["remaining_cm3"].as_f64().unwrap();
    let r2 = setups[1]["remaining_cm3"].as_f64().unwrap();
    let r3 = setups[2]["remaining_cm3"].as_f64().unwrap();
    assert!(r1 < init, "setup 1 cut: init={init}, r1={r1}");
    assert!(r2 < r1, "setup 2 flipped cut stock: {r1} -> {r2}");
    assert!(r3 < r2, "setup 3 side-on cut stock: {r2} -> {r3}");
    let pv = sc["part_volume_cm3"].as_f64().unwrap();
    assert!((pv - 14.34).abs() < 0.05, "part_volume_cm3 across aligned setups: {pv}");
    assert!(!sc["keyframes"].as_array().unwrap().is_empty(), "keyframes written");

    // Re-rendering only setup 3 must silently simulate setups 1 & 2 to reach the same final volume.
    let single = rpc_call(
        &mut srv,
        4,
        "render_simulation_video",
        json!({
            "output": s3_dir.display().to_string(),
            "format": "frames",
            "setup": "OP30_Side",
            "width": 120,
            "height": 80,
            "fps": 10,
            "cell_mm": 2.0,
            "seconds": 0.2,
            "transition_seconds": 0.0,
            "spin_seconds": 0.0
        }),
    );
    assert_eq!(single["isError"], false, "single setup render failed: {single}");
    let ss = &single["structuredContent"];
    assert_eq!(ss["setups"][0]["rendered"], false);
    assert_eq!(ss["setups"][1]["rendered"], false);
    assert_eq!(ss["setups"][2]["rendered"], true);
    assert_eq!(ss["remaining_cm3"], sc["remaining_cm3"]);

    // Unknown setup name refuses cleanly via MCP error.
    let bad = rpc_call(
        &mut srv,
        5,
        "render_simulation_video",
        json!({ "output": s3_dir.display().to_string(), "format": "frames", "setup": "NO_SUCH" }),
    );
    assert_eq!(bad["isError"], true);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn refusals_and_live_gate_are_honest() {
    let world = facing_world();
    // Ungenerated op refuses clearly.
    let err = render_simulation_video(
        &world,
        &VideoOptions {
            output: PathBuf::from("/tmp/should_not_exist"),
            format: VideoFormat::Frames,
            width: 120,
            height: 80,
            fps: 10,
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(err.contains("generate"), "mentions generate: {err}");

    // Odd frame width rejected for MP4/WebM (yuv420p compatibility).
    let err = render_simulation_video(
        &world,
        &VideoOptions {
            output: PathBuf::from("/tmp/odd.mp4"),
            format: VideoFormat::Mp4,
            width: 321,
            height: 240,
            fps: 10,
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(err.contains("even"), "mentions even: {err}");

    // Unknown format / view parse cleanly to Err.
    assert!(VideoFormat::parse("avi").is_err());
    assert!(CameraView::parse("bottom").is_err());

    // Live approval gate treats render_simulation_video as a file write.
    assert_eq!(
        requires_approval("render_simulation_video", &json!({ "output": "sim.mp4" })),
        Some(GateKind::FileWrite)
    );
}

#[test]
fn cli_render_video_help_usage_and_generate_work() {
    let bin = env!("CARGO_BIN_EXE_camcraft-cli");
    let help = Command::new(bin).args(["render-video", "--help"]).output().expect("run");
    assert_eq!(help.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&help.stdout).contains("--per-setup"));

    // Usage refusals exit 2.
    let no_out = Command::new(bin).args(["render-video", "job.camproj"]).output().expect("run");
    assert_eq!(no_out.status.code(), Some(2));
    let bad_ext = Command::new(bin)
        .args(["render-video", "job.js", "-o", "out.mp4"])
        .output()
        .expect("run");
    assert_eq!(bad_ext.status.code(), Some(2));

    // Real headless end-to-end render with --generate to a pure-Rust frame folder.
    let dir = scratch_dir("cli");
    std::fs::create_dir_all(&dir).expect("mkdir");
    let proj = dir.join("tiny.camproj");
    let frames = dir.join("frames");
    project::save(&facing_world(), &proj).expect("save");
    let run = Command::new(bin)
        .args([
            "render-video",
            proj.to_str().unwrap(),
            "-o",
            frames.to_str().unwrap(),
            "--format",
            "frames",
            "--generate",
            "--width",
            "120",
            "--height",
            "80",
            "--fps",
            "10",
            "--cell",
            "2.0",
            "--seconds",
            "0.3",
            "--spin",
            "0.0",
            "--no-hud",
        ])
        .output()
        .expect("run");
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(run.status.code(), Some(0), "stderr={stderr} stdout={stdout}");
    assert!(frames.join("frame_00001.png").exists(), "wrote PNG frame");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cylinder_and_model_stock_shapes_match_initial_volumes() {
    // 1) Cylinder stock: diameter=40 (r=20), height=12 -> pi*400*12 / 1000 = 15.08 cm^3.
    let dir = scratch_dir("stock_shapes");
    let world = new_world();
    let mut history = History::default();
    let cyl = StockShape::Cylinder { diameter: 40.0, height: 12.0 };
    history.execute(&world, SetStock::new(&world, cyl, Default::default()).expect("cyl stock"));
    let tool = catalog()
        .into_iter()
        .find(|(n, _)| *n == "EM-0400-2FL")
        .map(|(_, t)| t)
        .expect("catalog tool");
    history.execute(&world, AddTool::new(&world, "EM-0400-2FL", tool.clone()).expect("tool"));
    let params = OpParams::Face {
        holes: Default::default(),
        blend_turns: false,
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::fixed(18.0),
        cut_z: Height::fixed(10.0),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    };
    history.execute(
        &world,
        CreateOperation::new(&world, "FaceCyl", "EM-0400-2FL", SrcRef::Stock, params.clone()).expect("op"),
    );
    history.execute(&world, GenerateOperation::new(&world, "FaceCyl").expect("gen"));
    let rep_c = render_simulation_video(
        &world,
        &VideoOptions {
            output: dir.join("cyl"),
            format: VideoFormat::Frames,
            width: 100,
            height: 80,
            fps: 10,
            cell_mm: Some(1.0),
            seconds: Some(0.2),
            spin_seconds: Some(0.0),
            transition_seconds: Some(0.0),
            ..Default::default()
        },
    )
    .expect("cyl render");
    let expected_cyl = std::f64::consts::PI * 20.0 * 20.0 * 12.0 / 1000.0;
    assert!(
        (rep_c.initial_cm3 - expected_cyl).abs() < 0.8,
        "cylinder initial_cm3={} expected~{expected_cyl:.2}",
        rep_c.initial_cm3
    );
    assert!(rep_c.remaining_cm3 < rep_c.initial_cm3);

    // 2) FromModel stock: closed 30x20x10 box -> 6.0 cm^3.
    let w2 = new_world();
    let mut h2 = History::default();
    let p = |x, y, z| Point3::new(x, y, z);
    let v = [
        p(-15.0, -10.0, 0.0), p(15.0, -10.0, 0.0), p(15.0, 10.0, 0.0), p(-15.0, 10.0, 0.0),
        p(-15.0, -10.0, 10.0), p(15.0, -10.0, 10.0), p(15.0, 10.0, 10.0), p(-15.0, 10.0, 10.0),
    ];
    let tris: Vec<Triangle> = [
        (0, 2, 1), (0, 3, 2), (4, 5, 6), (4, 6, 7), (0, 1, 5), (0, 5, 4),
        (2, 3, 7), (2, 7, 6), (0, 4, 7), (0, 7, 3), (1, 2, 6), (1, 6, 5),
    ]
    .into_iter()
    .map(|(a, b, c)| Triangle::new(v[a], v[b], v[c]))
    .collect();
    h2.execute(&w2, ImportModel::from_mesh("cast_stock", Mesh::from_facets(tris)));
    h2.execute(
        &w2,
        SetStock::new(&w2, StockShape::FromModel { source: "cast_stock".into() }, Default::default()).expect("from model"),
    );
    h2.execute(&w2, AddTool::new(&w2, "EM-0400-2FL", tool).expect("tool"));
    let p2 = OpParams::Face {
        holes: Default::default(),
        blend_turns: false,
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::fixed(18.0),
        cut_z: Height::fixed(8.0),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    };
    h2.execute(&w2, CreateOperation::new(&w2, "FaceCast", "EM-0400-2FL", SrcRef::Stock, p2).expect("op"));
    h2.execute(&w2, GenerateOperation::new(&w2, "FaceCast").expect("gen"));
    let rep_m = render_simulation_video(
        &w2,
        &VideoOptions {
            output: dir.join("mesh"),
            format: VideoFormat::Frames,
            width: 100,
            height: 80,
            fps: 10,
            cell_mm: Some(1.0),
            seconds: Some(0.2),
            spin_seconds: Some(0.0),
            transition_seconds: Some(0.0),
            ..Default::default()
        },
    )
    .expect("model stock render");
    assert!((rep_m.initial_cm3 - 6.0).abs() < 0.4, "from_model got {}", rep_m.initial_cm3);
    assert!(rep_m.remaining_cm3 < rep_m.initial_cm3);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn camera_views_and_hud_and_toolpath_toggles_change_rendered_pixels() {
    let dir = scratch_dir("views");
    let world = facing_world();
    let mut history = History::default();
    history.execute(&world, GenerateOperation::new(&world, "Face1").expect("gen"));

    let frame_bytes = |sub: &str, view: CameraView, hud: bool, toolpath: bool| -> Vec<u8> {
        let out = dir.join(sub);
        let rep = render_simulation_video(
            &world,
            &VideoOptions {
                output: out.clone(),
                format: VideoFormat::Frames,
                width: 360,
                height: 200,
                fps: 10,
                cell_mm: Some(2.0),
                view,
                hud,
                toolpath,
                seconds: Some(0.2),
                spin_seconds: Some(0.0),
                transition_seconds: Some(0.0),
                ..Default::default()
            },
        )
        .expect("render");
        assert!(rep.total_frames >= 1);
        std::fs::read(out.join("frame_00001.png")).expect("read frame 1")
    };

    let iso_full = frame_bytes("iso_full", CameraView::Iso, true, true);
    let iso_no_hud = frame_bytes("iso_no_hud", CameraView::Iso, false, true);
    let iso_no_tp = frame_bytes("iso_no_tp", CameraView::Iso, true, false);
    let top = frame_bytes("top", CameraView::Top, false, false);
    let side = frame_bytes("side", CameraView::Side, false, false);
    let front = frame_bytes("front", CameraView::FrontRight, false, false);

    assert_ne!(iso_full, iso_no_hud, "hud toggle must alter rendered pixels");
    assert_ne!(iso_full, iso_no_tp, "toolpath toggle must alter rendered pixels");
    assert_ne!(top, side, "top vs side view must differ");
    assert_ne!(top, front, "top vs front-right view must differ");
    assert_ne!(side, front, "side vs front-right view must differ");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn helical_bore_arc_and_drill_toolpath_render_and_carve_material() {
    let dir = scratch_dir("bore_drill");
    let mut srv = Server::new();
    let r = rpc_call(&mut srv, 1, "set_stock", json!({ "box": { "dx": 40.0, "dy": 40.0, "dz": 16.0 } }));
    assert_eq!(r["isError"], false, "{r}");
    let r = rpc_call(&mut srv, 2, "add_tool", json!({ "name": "EM", "catalog": "EM-0400-2FL" }));
    assert_eq!(r["isError"], false, "{r}");
    let drill_name = catalog()
        .into_iter()
        .find(|(_, t)| t.kind == camcraft_app::tools::ToolKind::Drill)
        .map(|(n, _)| n)
        .expect("drill in catalog");
    let drill_r = catalog()
        .into_iter()
        .find(|(n, _)| *n == drill_name)
        .unwrap()
        .1
        .diameter
        * 0.5;
    let r = rpc_call(&mut srv, 3, "add_tool", json!({ "name": "DR", "catalog": drill_name }));
    assert_eq!(r["isError"], false, "{r}");
    let r = rpc_call(
        &mut srv,
        4,
        "add_operation",
        json!({
            "kind": "Bore",
            "name": "Bore1",
            "tool": "EM",
            "source": "stock",
            "params": {
                "stepdown": 3.0,
                "clearance_z": { "reference": "top_of_stock", "offset": 6.0 },
                "top_z": { "reference": "top_of_stock", "offset": 0.0 },
                "bottom_z": { "reference": "top_of_stock", "offset": -12.0 },
                "holes": [{
                    "xy": [-8.0, 0.0], "radius": 4.5,
                    "entry_z": 16.0, "floor_z": 4.0,
                    "through": false, "stepped": false
                }]
            }
        }),
    );
    assert_eq!(r["isError"], false, "add bore: {r}");
    let r = rpc_call(
        &mut srv,
        5,
        "add_operation",
        json!({
            "kind": "Drill",
            "name": "Drill1",
            "tool": "DR",
            "source": "stock",
            "params": {
                "clearance_z": { "reference": "top_of_stock", "offset": 6.0 },
                "top_z": { "reference": "top_of_stock", "offset": 0.0 },
                "bottom_z": { "reference": "top_of_stock", "offset": -4.0 },
                "holes": [{
                    "xy": [8.0, 0.0], "radius": drill_r,
                    "entry_z": 16.0, "floor_z": 12.0,
                    "through": false, "stepped": false
                }]
            }
        }),
    );
    assert_eq!(r["isError"], false, "add drill: {r}");
    let r = rpc_call(&mut srv, 6, "generate", json!({}));
    assert_eq!(r["isError"], false, "gen: {r}");

    let out = rpc_call(
        &mut srv,
        7,
        "render_simulation_video",
        json!({
            "output": dir.display().to_string(),
            "format": "frames",
            "width": 160,
            "height": 100,
            "fps": 10,
            "cell_mm": 1.0,
            "seconds": 0.4,
            "spin_seconds": 0.0,
            "transition_seconds": 0.0
        }),
    );
    assert_eq!(out["isError"], false, "render bore+drill: {out}");
    let sc = &out["structuredContent"];
    // Helical Bore generates circular arc moves expanded into chords (`segments3`), so segments >> 20.
    let segs = sc["setups"][0]["segments"].as_u64().unwrap();
    assert!(segs > 20, "arcs expanded into chords: {segs}");
    let init = sc["initial_cm3"].as_f64().unwrap();
    let rem = sc["remaining_cm3"].as_f64().unwrap();
    assert!(rem < init - 0.5, "carved bore+drill: init={init}, rem={rem}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mcp_and_cli_refuse_invalid_options_voxel_overflow_and_unknown_args() {
    let dir = scratch_dir("bad_opts");
    std::fs::create_dir_all(&dir).expect("mkdir");
    let proj = dir.join("ok.camproj");
    let world = facing_world();
    let mut history = History::default();
    history.execute(&world, GenerateOperation::new(&world, "Face1").expect("gen"));
    project::save(&world, &proj).expect("save");

    let mut srv = Server::new();
    let opened = rpc_call(&mut srv, 1, "open_project", json!({ "path": proj.display().to_string() }));
    assert_eq!(opened["isError"], false);

    // Unknown argument rejected by tool schema validator.
    let bad_arg = rpc_call(&mut srv, 2, "render_simulation_video", json!({ "output": "a", "bogus_key": 1 }));
    assert_eq!(bad_arg["isError"], true);
    // Negative / zero cell_mm refused.
    let zero_cell = rpc_call(&mut srv, 3, "render_simulation_video", json!({ "output": "a", "format": "frames", "cell_mm": 0.0 }));
    assert_eq!(zero_cell["isError"], true);
    // Tiny cell exceeding 120M voxel cap refused cleanly instead of OOMing.
    let huge = rpc_call(&mut srv, 4, "render_simulation_video", json!({ "output": "a", "format": "frames", "cell_mm": 0.01 }));
    assert_eq!(huge["isError"], true);
    assert!(huge["content"][0]["text"].as_str().unwrap().contains("120M"));
    // Odd height for WebM refused.
    let odd_webm = rpc_call(&mut srv, 5, "render_simulation_video", json!({ "output": "a.webm", "width": 120, "height": 81 }));
    assert_eq!(odd_webm["isError"], true);
    // Out-of-range fps refused.
    let bad_fps = rpc_call(&mut srv, 6, "render_simulation_video", json!({ "output": "a", "fps": 500 }));
    assert_eq!(bad_fps["isError"], true);

    // CLI argv refusals exit 2.
    let bin = env!("CARGO_BIN_EXE_camcraft-cli");
    for bad in [
        &["render-video", "ok.camproj", "-o", "a", "--unknown-flag"][..],
        &["render-video", "ok.camproj", "-o"][..],
        &["render-video", "ok.camproj", "-o", "a", "--width", "abc"][..],
        &["render-video", "ok.camproj", "-o", "a", "--cell", "nan"][..],
        &["render-video", "a.camproj", "b.camproj", "-o", "out"][..],
    ] {
        let o = Command::new(bin).args(bad).output().expect("cli");
        assert_eq!(o.status.code(), Some(2), "expected exit 2 for {bad:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
