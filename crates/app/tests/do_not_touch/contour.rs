//! #3344: "Do not touch" protection for 2D Contour operations.
//! Provenance: our own integration pins extending #2055 (`do_not_touch.rs`) to 2D Contour;
//! no vendor material consulted.

use camcraft_app::verify::{AVOID_VERIFY_TOL, AvoidItem, AvoidSpec, Cutter};
use camcraft_app::world::new_world;
use camcraft_app::{model, operation, posting, project};
use super::do_not_touch_support::setup_plate_and_tower;
use serde_json::json;

fn nc_waypoints(nc: &str) -> Vec<[f64; 3]> {
    let (mut x, mut y, mut z) = (0.0, 0.0, 50.0);
    let mut armed = false;
    let mut pts = Vec::new();
    for line in nc.lines() {
        let trim = line.trim();
        if trim.contains("G43") || trim.contains("G54") {
            armed = true;
            continue;
        }
        if !armed || trim.contains("G28") || trim.contains("G53") || trim.contains("G91") || trim.starts_with('(') {
            continue;
        }
        let mut saw_coord = false;
        for tok in trim.split_whitespace() {
            if let Some(rest) = tok.strip_prefix('X')
                && let Ok(v) = rest.parse::<f64>()
            {
                x = v;
                saw_coord = true;
            } else if let Some(rest) = tok.strip_prefix('Y')
                && let Ok(v) = rest.parse::<f64>()
            {
                y = v;
                saw_coord = true;
            } else if let Some(rest) = tok.strip_prefix('Z')
                && let Ok(v) = rest.parse::<f64>()
            {
                z = v;
                saw_coord = true;
            }
        }
        if saw_coord {
            pts.push([x, y, z]);
        }
    }
    pts
}

/// Verify posted Fanuc NC waypoints keep SteppedMill flutes, shank, and holder clear of a protected box.
fn assert_contour_nc_clears_box(nc_path: &std::path::Path, req: f64, tower_bounds: ([f64; 3], [f64; 3])) {
    let txt = std::fs::read_to_string(nc_path).expect("read contour nc");
    let pts = nc_waypoints(&txt);
    assert!(!pts.is_empty(), "expected NC waypoints in {}", nc_path.display());
    let ([x0, y0, z0], [x1, y1, z1]) = tower_bounds;
    // SteppedMill after `reach_the_plate`: flute r=3.0 (0..25), shank r=6.0 (25..27), holder r=14.0 (27..47).
    // Each band is a cylinder over the tip; its gap to the box is the 3D distance
    // between the two (radial and vertical parts), as the clearance law measures it.
    let bands = [("flute", 3.0, 0.0, 25.0), ("shank", 6.0, 25.0, 27.0), ("holder", 14.0, 27.0, 47.0)];
    for p in pts {
        let dx = if p[0] < x0 { x0 - p[0] } else if p[0] > x1 { p[0] - x1 } else { 0.0 };
        let dy = if p[1] < y0 { y0 - p[1] } else if p[1] > y1 { p[1] - y1 } else { 0.0 };
        let xy = dx.hypot(dy);
        for (name, r, lo, hi) in bands {
            let dz = (z0 - (p[2] + hi)).max(p[2] + lo - z1).max(0.0);
            let gap = (xy - r).max(0.0).hypot(dz);
            assert!(gap >= req - 0.02, "{name} came within {gap:.3} of tower at {p:?}");
        }
    }
}

/// The finding kinds of a `verify` reply.
fn finding_kinds(ver: &serde_json::Value) -> std::collections::BTreeSet<String> {
    ver["findings"].as_array().into_iter().flatten().filter_map(|f| f["kind"].as_str().map(str::to_owned)).collect()
}

/// The shared fixture's stock stands 24 mm tall, and these operations cut the
/// plate at Z 10..5. A 15 mm flute leaves the shank in uncut stock, which the
/// flute law (#3525) blocks with or without protection (the tilted case cuts
/// about 21 mm down). A 25 mm flute and a 2 mm shank keep the holder base at
/// tip + 27 mm, as before.
fn reach_the_plate(c: &mut super::do_not_touch_support::Client) {
    c.ok("edit_tool", json!({ "name": "SteppedMill", "patch": { "flute_len": 25.0, "shank_len": 2.0 } }));
}

#[test]
fn contour_do_not_touch_keeps_full_tool_clear_of_protected_face_and_edge() {
    let (dir, mut c, _plate_top, tower_top_face, tower_edge, tower_vert) = setup_plate_and_tower("contour_clear");
    reach_the_plate(&mut c);

    // Place tower outside the plate at [53..65, -6..6, 0..24] so the straight outer wall at x = 43.5
    // is 9.5 mm clear (>= shank_r 6 + req 2 = 8.0) while 4 mm arc leads or links toward +X must keep clear.
    c.ok("move_model", json!({ "name": "tower", "position": [59.0, 0.0, 0.0] }));

    let added = c.ok("add_operation", json!({
        "kind": "2D Contour",
        "tool": "SteppedMill",
        "source": "plate",
        "params": {
            "stock_to_leave": 0.5,
            "stepdown": 5.0,
            "top_z": { "reference": "fixed", "offset": 10.0 },
            "bottom_z": { "reference": "fixed", "offset": 5.0 },
            "clearance_z": { "reference": "fixed", "offset": 35.0 },
            "retract_z": { "reference": "fixed", "offset": 26.0 },
            "lead": { "linear": null, "arc": { "radius": 4.0 } }
        }
    }));
    let op_name = added["operation"].as_str().expect("op name").to_owned();

    // Vertex picks and negative clearance are still refused on Contour.
    let vertex_err = c.refused("set_operation_do_not_touch", json!({
        "operation": op_name,
        "picks": [{ "model": "tower", "id": tower_vert }]
    }));
    assert!(vertex_err.contains("vertex"), "expected vertex refusal: {vertex_err}");

    c.ok("set_operation_do_not_touch", json!({
        "operation": op_name,
        "clearance": 1.5,
        "picks": [
            { "model": "tower", "id": tower_top_face },
            { "model": "tower", "id": tower_edge }
        ]
    }));

    let got = c.ok("get_operation_do_not_touch", json!({ "operation": op_name }));
    assert_eq!(got["clearance"], json!(1.5));
    assert_eq!(got["picks"].as_array().map(Vec::len), Some(2));

    let generated = c.ok("generate", json!({ "name": op_name }));
    assert_eq!(generated["generated"][0]["status"], json!("ok"), "{generated}");
    let ver = c.ok("verify", json!({}));
    assert_eq!(ver["ok"], json!(true), "{ver}");
    assert!(
        !ver["findings"].as_array().is_some_and(|fs| fs.iter().any(|f| f["kind"] == json!("avoid_hit"))),
        "Contour must have zero avoid_hit findings: {ver}"
    );
    let nc = dir.join("contour.nc");
    let posted = c.ok("post", json!({ "output": nc.display().to_string(), "post": "Fanuc" }));
    assert!(posted["lines"].as_u64().is_some_and(|n| n > 0));

    // Also test a protected tower overlapping one side of the contour ([38..50, -6..6, 0..24]):
    // Contour must trim/avoid the protected zone so verify and post both stay green.
    c.ok("move_model", json!({ "name": "tower", "position": [44.0, 0.0, 0.0] }));
    let generated_trim = c.ok("generate", json!({ "name": op_name }));
    assert_eq!(generated_trim["generated"][0]["status"], json!("ok"), "{generated_trim}");
    let ver_trim = c.ok("verify", json!({}));
    assert_eq!(ver_trim["ok"], json!(true), "{ver_trim}");
    let nc_trim = dir.join("contour_trim.nc");
    let posted_trim = c.ok("post", json!({ "output": nc_trim.display().to_string(), "post": "Fanuc" }));
    assert!(posted_trim["lines"].as_u64().is_some_and(|n| n > 0));
    assert_contour_nc_clears_box(&nc_trim, 2.0, ([38.0, -6.0, 0.0], [50.0, 6.0, 24.0]));

    // Holder clearance check: tower at z=[10..34] outside plate at [50..62, -6..6] only reaches the holder (r=14)
    // when cutting at z=5 (holder base = 5+27 = 32 <= 34 + 2). Contour must keep holder clear too.
    c.ok("move_model", json!({ "name": "tower", "position": [56.0, 0.0, 10.0] }));
    let gen_holder = c.ok("generate", json!({ "name": op_name }));
    assert_eq!(gen_holder["generated"][0]["status"], json!("ok"), "{gen_holder}");
    let ver_holder = c.ok("verify", json!({}));
    assert_eq!(ver_holder["ok"], json!(true), "{ver_holder}");
    let nc_holder = dir.join("contour_holder.nc");
    let post_holder = c.ok("post", json!({ "output": nc_holder.display().to_string(), "post": "Fanuc" }));
    assert!(post_holder["lines"].as_u64().is_some_and(|n| n > 0));
    assert_contour_nc_clears_box(&nc_holder, 2.0, ([50.0, -6.0, 10.0], [62.0, 6.0, 34.0]));

    // Tilted (3+2) Contour with Do not touch overlapping +X wall also generates & verifies clean.
    c.ok("move_model", json!({ "name": "tower", "position": [44.0, 0.0, 0.0] }));
    let tilted = c.ok("add_operation", json!({
        "kind": "2D Contour",
        "tool": "SteppedMill",
        "source": "plate",
        "params": {
            "stepdown": 5.0,
            "top_z": { "reference": "fixed", "offset": 8.0 },
            "bottom_z": { "reference": "fixed", "offset": 3.0 },
            "clearance_z": { "reference": "fixed", "offset": 45.0 },
            "retract_z": { "reference": "fixed", "offset": 35.0 },
            "tilt": { "a_deg": 5.0, "b_deg": 0.0, "origin": [0.0, 0.0, 0.0] }
        }
    }))["operation"].as_str().expect("tilted op").to_owned();
    // The plain tilted path already has gouge findings on main: the plate's
    // vertical walls stand 5 degrees off the tool axis. Its post is blocked
    // with or without protection; this case pins only the protection.
    c.ok("set_operation_do_not_touch", json!({
        "operation": tilted,
        "clearance": 1.0,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    let tgen = c.ok("generate", json!({ "name": tilted }));
    assert_eq!(tgen["generated"][0]["status"], json!("ok"), "{tgen}");
    let tver = c.ok("verify", json!({ "name": tilted }));
    assert!(
        !tver["findings"].as_array().is_some_and(|fs| fs.iter().any(|f| f["kind"] == json!("avoid_hit"))),
        "tilted Contour must have zero avoid_hit findings: {tver}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn contour_do_not_touch_refuses_unavoidable_cut_and_blocks_post() {
    let (dir, mut c, plate_top, tower_top_face, tower_edge, _vert) = setup_plate_and_tower("contour_refuse");
    reach_the_plate(&mut c);

    // Protecting the whole plate top with large clearance leaves no safe Contour cut room:
    // generate must refuse naming the protected item and post must be blocked.
    let op_block = c.ok("add_operation", json!({
        "kind": "2D Contour",
        "tool": "SteppedMill",
        "source": "plate",
        "params": {
            "stepdown": 5.0,
            "top_z": { "reference": "fixed", "offset": 10.0 },
            "bottom_z": { "reference": "fixed", "offset": 5.0 },
            "clearance_z": { "reference": "fixed", "offset": 35.0 },
            "retract_z": { "reference": "fixed", "offset": 26.0 }
        }
    }))["operation"].as_str().expect("op_block").to_owned();
    c.ok("set_operation_do_not_touch", json!({
        "operation": op_block,
        "clearance": 16.0,
        "picks": [{ "model": "plate", "id": plate_top }]
    }));
    let refused = c.refused("generate", json!({ "name": op_block }));
    assert!(refused.contains("plate") && refused.contains("Do not touch"), "expected item name in refusal: {refused}");
    let bad_nc = dir.join("blocked_unavoidable.nc");
    let post_refused = c.refused("post", json!({ "output": bad_nc.display().to_string(), "post": "Fanuc" }));
    assert!(!bad_nc.exists(), "blocked post must never create NC output: {post_refused}");

    // Fresh Contour where moving the protected tower onto the cut after generation blocks verify and post.
    c.ok("set_operation_do_not_touch", json!({
        "operation": op_block,
        "clearance": 1.5,
        "picks": [
            { "model": "tower", "id": tower_top_face },
            { "model": "tower", "id": tower_edge }
        ]
    }));
    c.ok("generate", json!({ "name": op_block }));
    let proj_path = dir.join("contour_protected.camproj");
    c.ok("save_project", json!({ "path": proj_path.display().to_string() }));

    c.ok("move_model", json!({ "name": "tower", "position": [44.0, 0.0, 0.0] }));
    let ver_bad = c.ok("verify", json!({}));
    assert_eq!(ver_bad["ok"], json!(false), "{ver_bad}");
    let post_bad = dir.join("blocked_moved.nc");
    let post_moved_err = c.refused("post", json!({ "output": post_bad.display().to_string(), "post": "Fanuc" }));
    assert!(!post_bad.exists(), "post must never create NC output after tower move: {post_moved_err}");

    // Saved project reloads clean and has no live avoid problems before move.
    let w = new_world();
    project::load(&w, &proj_path).expect("reload saved contour project");
    let live = operation::avoid::live_avoid_problems(&w);
    assert!(live.is_empty(), "clean saved contour has no live avoid problems: {live:?}");
    let op_ent = model::find_by_name(&w, &op_block).expect("op entity");
    let cache = model::toolpath_of(&w, &op_ent).expect("cached contour toolpath");
    assert!(!cache.path.moves.is_empty());
    let _ = posting::verification_report(&w);
    let _ = (AVOID_VERIFY_TOL, std::any::type_name::<AvoidItem>(), std::any::type_name::<AvoidSpec>(), std::any::type_name::<Cutter>());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn contour_without_do_not_touch_keeps_same_toolpath() {
    let (dir, mut c, plate_top, _tower_top_face, _tower_edge, _vert) = setup_plate_and_tower("contour_baseline");
    reach_the_plate(&mut c);
    let op_name = c.ok("add_operation", json!({
        "kind": "2D Contour",
        "tool": "SteppedMill",
        "params": {
            "stock_to_leave": 0.5,
            "stepdown": 5.0,
            "top_z": { "reference": "fixed", "offset": 10.0 },
            "bottom_z": { "reference": "fixed", "offset": 5.0 },
            "clearance_z": { "reference": "fixed", "offset": 35.0 },
            "retract_z": { "reference": "fixed", "offset": 26.0 }
        }
    }))["operation"].as_str().expect("op_name").to_owned();
    c.ok("set_operation_geometry", json!({
        "operation": op_name,
        "picks": [{ "model": "plate", "id": plate_top }]
    }));
    c.ok("generate", json!({ "name": op_name }));
    let nc = dir.join("baseline.nc");
    let posted = c.ok("post", json!({ "output": nc.display().to_string(), "post": "Fanuc" }));
    assert!(posted["lines"].as_u64().is_some_and(|n| n > 0));
    let before = std::fs::read_to_string(&nc).expect("read baseline nc");

    // No "picks" clears the protection (an empty list is refused).
    c.ok("set_operation_do_not_touch", json!({ "operation": op_name, "clearance": 0.0 }));
    c.ok("generate", json!({ "name": op_name }));
    let nc2 = dir.join("empty_dnt.nc");
    c.ok("post", json!({ "output": nc2.display().to_string(), "post": "Fanuc" }));
    let after = std::fs::read_to_string(&nc2).expect("read empty dnt nc");
    assert_eq!(before, after, "empty Do not touch must preserve Contour output");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn contour_modes_picked_open_and_dxf_sketch_stay_clear() {
    let (dir, mut c, plate_top, tower_top_face, _tower_edge, _vert) = setup_plate_and_tower("contour_modes");
    reach_the_plate(&mut c);
    let plate_chains = c.ok("list_features", json!({ "model": "plate", "kinds": ["chains"] }));
    let east_edge = plate_chains["chains"]
        .as_array()
        .expect("plate chains")
        .iter()
        .find(|ch| {
            ch["bounds"]["min"][0].as_f64().is_some_and(|x| (x - 40.0).abs() < 1e-3)
                && ch["bounds"]["max"][0].as_f64().is_some_and(|x| (x - 40.0).abs() < 1e-3)
                && ch["bounds"]["min"][2].as_f64().is_some_and(|z| (z - 10.0).abs() < 1e-3)
        })
        .expect("east top edge on plate")["id"]
        .as_str()
        .expect("chain id")
        .to_owned();

    c.ok("move_model", json!({ "name": "tower", "position": [44.0, 0.0, 0.0] }));

    // 1) Picked open edge run on part + Do not touch.
    let run_op = c.ok("add_operation", json!({
        "kind": "2D Contour",
        "tool": "SteppedMill",
        "params": {
            "stepdown": 5.0,
            "top_z": { "reference": "fixed", "offset": 10.0 },
            "bottom_z": { "reference": "fixed", "offset": 5.0 },
            "clearance_z": { "reference": "fixed", "offset": 35.0 },
            "retract_z": { "reference": "fixed", "offset": 26.0 }
        }
    }))["operation"].as_str().expect("run_op").to_owned();
    c.ok("set_operation_geometry", json!({
        "operation": run_op,
        "picks": [{ "model": "plate", "id": east_edge }]
    }));
    c.ok("set_operation_do_not_touch", json!({
        "operation": run_op,
        "clearance": 1.0,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    let rgen = c.ok("generate", json!({ "name": run_op }));
    assert_eq!(rgen["generated"][0]["status"], json!("ok"), "{rgen}");
    let rver = c.ok("verify", json!({ "name": run_op }));
    assert_eq!(rver["ok"], json!(true), "{rver}");

    // 2) Contour riding an open boundary + Do not touch.
    let open_op = c.ok("add_operation", json!({
        "kind": "2D Contour",
        "tool": "SteppedMill",
        "params": {
            "stepdown": 5.0,
            "top_z": { "reference": "fixed", "offset": 10.0 },
            "bottom_z": { "reference": "fixed", "offset": 5.0 },
            "clearance_z": { "reference": "fixed", "offset": 35.0 },
            "retract_z": { "reference": "fixed", "offset": 26.0 }
        }
    }))["operation"].as_str().expect("open_op").to_owned();
    c.ok("set_operation_geometry", json!({
        "operation": open_op,
        "picks": [{ "model": "plate", "id": plate_top }]
    }));
    c.ok("set_operation_area", json!({
        "operation": open_op,
        "area": { "open": { "picks": [{ "model": "plate", "id": east_edge }], "side": "outside" } }
    }));
    c.ok("set_operation_do_not_touch", json!({
        "operation": open_op,
        "clearance": 1.0,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    let ogen = c.ok("generate", json!({ "name": open_op }));
    assert_eq!(ogen["generated"][0]["status"], json!("ok"), "{ogen}");
    let over = c.ok("verify", json!({ "name": open_op }));
    assert_eq!(over["ok"], json!(true), "{over}");

    // 3) DXF sketch Contour + Do not touch (sketch sits on stock top at Z=24; protected tower at [44, 0, 0]).
    let dxf_path = dir.join("profile.dxf");
    let dxf = "0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n\
               0\nLWPOLYLINE\n8\ncut\n90\n4\n70\n1\n10\n-30\n20\n-30\n10\n30\n20\n-30\n10\n30\n20\n30\n10\n-30\n20\n30\n\
               0\nLINE\n8\nopen\n10\n-20\n20\n0\n11\n45\n21\n0\n\
               0\nENDSEC\n0\nEOF\n";
    std::fs::write(&dxf_path, dxf).expect("write dxf");
    c.ok("import_dxf", json!({ "path": dxf_path.display().to_string(), "name": "sk" }));
    let sk_op = c.ok("add_operation", json!({
        "kind": "2D Contour",
        "tool": "SteppedMill",
        "source": "sk",
        "params": {
            "stepdown": 2.0,
            "top_z": { "reference": "fixed", "offset": 24.0 },
            "bottom_z": { "reference": "fixed", "offset": 20.0 },
            "clearance_z": { "reference": "fixed", "offset": 35.0 },
            "retract_z": { "reference": "fixed", "offset": 28.0 }
        }
    }))["operation"].as_str().expect("sk_op").to_owned();
    // The plain sketch path already has crossing and gouge findings on main
    // (the open line crosses the closed square). Protection must add none.
    c.ok("generate", json!({ "name": sk_op }));
    let plain = finding_kinds(&c.ok("verify", json!({ "name": sk_op })));
    c.ok("set_operation_do_not_touch", json!({
        "operation": sk_op,
        "clearance": 1.0,
        "picks": [{ "model": "tower", "id": tower_top_face }]
    }));
    let sgen = c.ok("generate", json!({ "name": sk_op }));
    assert_eq!(sgen["generated"][0]["status"], json!("ok"), "{sgen}");
    let sver = c.ok("verify", json!({ "name": sk_op }));
    assert!(finding_kinds(&sver).is_subset(&plain), "protection added a finding: {sver}");
    assert!(!finding_kinds(&sver).contains("avoid_hit"), "sketch Contour must have zero avoid_hit findings: {sver}");

    let _ = std::fs::remove_dir_all(&dir);
}
