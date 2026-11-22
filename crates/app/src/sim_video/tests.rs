//! Internal unit tests for `sim_video` covering math/SLERP/ray intersections,
//! voxel stock carving/masking/invariants across all 5 cutter shapes and arbitrary
//! spindle axes, format/path encoding, HUD layout & occlusion, and camera azimuth logic.

use super::encode::{per_setup_path, write_png, VideoFormat};
use super::hud::{draw_hud, draw_line_2d, fill_rect, HudInfo, OpBar};
use super::math::{
    disp_to_part, intersect_aabb, intersect_cone_z, intersect_cylinder_z, intersect_sphere, norm_quat, part_to_disp,
    quat_slerp, Camera, Mat3, Vec3,
};
use super::plan::{VideoOp, VideoSeg, VideoSetup};
use super::voxel::{
    tag_color, SimTool, ToolShape, VoxelStock, TAG_BALL, TAG_CHAMFER, TAG_DRILL, TAG_FACED, TAG_RAW,
};
use super::{allocate_op_frames, setup_az_bias, validate_options, CameraView, VideoOptions};
use crate::geometry::{quat_rotate_point, Point3, Triangle};

use std::path::{Path, PathBuf};

fn mk_tool(shape: ToolShape, d: f32, corner_r: f32, tip_angle_deg: f32, cut_height: f32, tag: u8) -> SimTool {
    SimTool {
        shape,
        diameter: d,
        corner_r,
        tip_angle_deg,
        flute_len: cut_height,
        total_len: cut_height + 10.0,
        holder_d: d * 2.5,
        label: "T01".into(),
        tag,
        cut_height,
    }
}

fn close_v3(a: Vec3, b: Vec3, tol: f32) -> bool {
    (a.x - b.x).abs() <= tol && (a.y - b.y).abs() <= tol && (a.z - b.z).abs() <= tol
}

// ---- math.rs ------------------------------------------------------------------

#[test]
fn mat3_from_quat_matches_camcraft_geometry_and_is_orthogonal() {
    let quats: [[f64; 4]; 5] = [
        [0.0, 0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0, 0.0], // 180 deg about X
        [0.0, -std::f64::consts::FRAC_1_SQRT_2, 0.0, std::f64::consts::FRAC_1_SQRT_2], // -90 deg about Y
        [std::f64::consts::FRAC_1_SQRT_2, 0.0, 0.0, std::f64::consts::FRAC_1_SQRT_2],  // +90 deg about X
        [0.2, -0.4, 0.6, 0.663324958], // arbitrary rotation (normalized below)
    ];
    let sample = Point3::new(3.5, -7.25, 11.0);
    for q in quats {
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        let qn = [q[0] / len, q[1] / len, q[2] / len, q[3] / len];
        let qf = [qn[0] as f32, qn[1] as f32, qn[2] as f32, qn[3] as f32];
        let m = Mat3::from_quat(qf);
        let got = m.mul(Vec3::new(sample.x as f32, sample.y as f32, sample.z as f32));
        let want = quat_rotate_point(qn, sample);
        assert!(
            close_v3(got, Vec3::new(want.x as f32, want.y as f32, want.z as f32), 1e-4),
            "quat={qn:?}: got={got:?} want={want:?}"
        );
        // Orthogonal inverse: M^T * (M * v) == v
        let back = m.mul_t(got);
        assert!(close_v3(back, Vec3::new(3.5, -7.25, 11.0), 1e-4));
    }
    // Zero-quaternion degenerates safely to identity.
    assert_eq!(norm_quat([0.0, 0.0, 0.0, 0.0]), [0.0, 0.0, 0.0, 1.0]);
}

#[test]
fn quat_slerp_takes_shortest_arc_handles_double_cover_and_midpoint() {
    let id = [0.0, 0.0, 0.0, 1.0f32];
    let neg_id = [0.0, 0.0, 0.0, -1.0f32];
    let mid_same = quat_slerp(id, neg_id, 0.5);
    assert!(Mat3::from_quat(mid_same).mul(Vec3::new(1.0, 2.0, 3.0)).sub(Vec3::new(1.0, 2.0, 3.0)).len() < 1e-4);

    // 90° around Y -> midpoint must be 45° around Y, rotating +Z to (sqrt(2)/2, 0, sqrt(2)/2).
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let q90 = [0.0, s, 0.0, s];
    let q45 = quat_slerp(id, q90, 0.5);
    let len = (q45[0] * q45[0] + q45[1] * q45[1] + q45[2] * q45[2] + q45[3] * q45[3]).sqrt();
    assert!((len - 1.0).abs() < 1e-5);
    let r = Mat3::from_quat(q45).mul(Vec3::new(0.0, 0.0, 1.0));
    assert!(close_v3(r, Vec3::new(s, 0.0, s), 1e-4), "got {r:?}");
    // Negated double-cover target produces identical midpoint rotation.
    let q45_neg = quat_slerp(id, [-q90[0], -q90[1], -q90[2], -q90[3]], 0.5);
    let r2 = Mat3::from_quat(q45_neg).mul(Vec3::new(0.0, 0.0, 1.0));
    assert!(close_v3(r2, r, 1e-4));
}

#[test]
fn part_and_display_coordinate_round_trip_and_camera_projection() {
    let rot = Mat3::from_quat([0.0, -0.5, 0.0, 0.866_025_4]);
    let center = Vec3::new(12.0, -5.0, 34.0);
    let p = Vec3::new(-18.5, 9.25, 42.0);
    let d = part_to_disp(p, rot, center);
    assert!(close_v3(disp_to_part(d, rot, center), p, 1e-4));

    let cam = Camera::look_at(Vec3::new(0.0, -100.0, 0.0), Vec3::ZERO, 60.0, 800, 600);
    let (sx, sy, depth) = cam.project(Vec3::ZERO).expect("origin in front");
    assert!((sx - 400.0).abs() < 0.5 && (sy - 300.0).abs() < 0.5 && (depth - 100.0).abs() < 1e-3);
    assert!(cam.project(Vec3::new(0.0, -120.0, 0.0)).is_none(), "behind camera returns None");
}

#[test]
fn analytic_ray_intersections_hit_miss_and_endcaps() {
    // AABB slab hit and miss
    let hit = intersect_aabb(Vec3::new(-10.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(-2.0, -2.0, -2.0), Vec3::new(2.0, 2.0, 2.0));
    let (t0, t1) = hit.expect("aabb hit");
    assert!((t0 - 8.0).abs() < 1e-4 && (t1 - 12.0).abs() < 1e-4);
    assert!(intersect_aabb(Vec3::new(-10.0, 5.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(-2.0, -2.0, -2.0), Vec3::new(2.0, 2.0, 2.0)).is_none());

    // Cylinder side & top cap & bottom cap & miss
    let (ts, ns) = intersect_cylinder_z(Vec3::new(-10.0, 0.0, 5.0), Vec3::new(1.0, 0.0, 0.0), 3.0, 0.0, 10.0).expect("cyl side");
    assert!((ts - 7.0).abs() < 1e-4 && close_v3(ns, Vec3::new(-1.0, 0.0, 0.0), 1e-4));
    let (tt, nt) = intersect_cylinder_z(Vec3::new(1.0, 0.0, 20.0), Vec3::new(0.0, 0.0, -1.0), 3.0, 0.0, 10.0).expect("top cap");
    assert!((tt - 10.0).abs() < 1e-4 && close_v3(nt, Vec3::new(0.0, 0.0, 1.0), 1e-4));
    let (tb, nb) = intersect_cylinder_z(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0), 3.0, 0.0, 10.0).expect("bottom cap");
    assert!((tb - 5.0).abs() < 1e-4 && close_v3(nb, Vec3::new(0.0, 0.0, -1.0), 1e-4));
    assert!(intersect_cylinder_z(Vec3::new(-10.0, 4.0, 5.0), Vec3::new(1.0, 0.0, 0.0), 3.0, 0.0, 10.0).is_none());

    // Cone 45° widening from r0=0 at z=0 to r1=5 at z=5 (at z=3, r=3 -> hit x=-3, t=7)
    let (tc, _nc) = intersect_cone_z(Vec3::new(-10.0, 0.0, 3.0), Vec3::new(1.0, 0.0, 0.0), 0.0, 5.0, 0.0, 5.0).expect("cone hit");
    assert!((tc - 7.0).abs() < 1e-3, "tc={tc}");
    assert!(intersect_cone_z(Vec3::new(-10.0, 0.0, 3.0), Vec3::new(1.0, 0.0, 0.0), 0.0, 5.0, 2.0, 2.0).is_none(), "degenerate height");

    // Sphere
    let (th, nh) = intersect_sphere(Vec3::new(-10.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO, 4.0).expect("sphere hit");
    assert!((th - 6.0).abs() < 1e-4 && close_v3(nh, Vec3::new(-1.0, 0.0, 0.0), 1e-4));
    assert!(intersect_sphere(Vec3::new(-10.0, 5.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO, 4.0).is_none());
}

// ---- voxel.rs -----------------------------------------------------------------

#[test]
fn sim_tool_radius_at_covers_all_five_tool_shapes_accurately() {
    // Flat
    let flat = mk_tool(ToolShape::Flat, 10.0, 0.0, 0.0, 20.0, TAG_FACED);
    assert_eq!(flat.radius_at(-0.01), None);
    assert_eq!(flat.radius_at(0.0), Some(5.0));
    assert_eq!(flat.radius_at(20.0), Some(5.0));
    assert_eq!(flat.radius_at(20.1), None);

    // Ball-nose R=5: at dz=2, r(2) = sqrt(25 - 9) = 4.0
    let ball = mk_tool(ToolShape::Ball, 10.0, 0.0, 0.0, 20.0, TAG_BALL);
    assert!((ball.radius_at(2.0).unwrap() - 4.0).abs() < 1e-4);
    assert!((ball.radius_at(5.0).unwrap() - 5.0).abs() < 1e-4);
    assert!((ball.radius_at(15.0).unwrap() - 5.0).abs() < 1e-4);

    // Bull-nose D=12 (R=6), corner_r=2: at dz=1, r = 4 + sqrt(4 - 1)
    let bull = mk_tool(ToolShape::Bull, 12.0, 2.0, 0.0, 20.0, TAG_BALL);
    assert!((bull.radius_at(1.0).unwrap() - (4.0 + 3.0_f32.sqrt())).abs() < 1e-4);
    assert!((bull.radius_at(2.0).unwrap() - 6.0).abs() < 1e-4);

    // Drill D=10 (R=5), 90° tip -> cone height = 5, at dz=2.5 r=2.5, above dz=5 r=5
    let drill = mk_tool(ToolShape::Drill, 10.0, 0.0, 90.0, 25.0, TAG_DRILL);
    assert!((drill.radius_at(2.5).unwrap() - 2.5).abs() < 1e-4);
    assert!((drill.radius_at(10.0).unwrap() - 5.0).abs() < 1e-4);

    // Chamfer 90° D=10 (R=5): cone height = 5, clipped above cone_h + 0.25
    let chamfer = mk_tool(ToolShape::Chamfer, 10.0, 0.0, 90.0, 25.0, TAG_CHAMFER);
    assert!((chamfer.radius_at(3.0).unwrap() - 3.0).abs() < 1e-4);
    assert!(chamfer.radius_at(6.0).is_none(), "chamfer cuts only bevel head");
}

#[test]
fn voxel_cylinder_and_mesh_masks_match_analytic_volumes() {
    let mut cyl = VoxelStock::from_box(Vec3::new(-10.0, -10.0, 0.0), Vec3::new(10.0, 10.0, 10.0), 0.5);
    cyl.mask_cylinder_z(0.0, 0.0, 8.0);
    let expected_cyl_cm3 = std::f64::consts::PI * 64.0 * 10.0 / 1000.0;
    assert!(
        (cyl.volume_cm3() - expected_cyl_cm3).abs() < 0.06,
        "cyl got {:.3} vs {:.3} cm3",
        cyl.volume_cm3(),
        expected_cyl_cm3
    );

    // Closed 10x8x6 mm box inside 16x16x12 mm grid -> 0.48 cm^3 exact.
    let mut mesh_stock = VoxelStock::from_box(Vec3::new(-8.0, -8.0, -2.0), Vec3::new(8.0, 8.0, 10.0), 0.5);
    let p = |x, y, z| Point3::new(x, y, z);
    let v = [
        p(-5.0, -4.0, 0.0), p(5.0, -4.0, 0.0), p(5.0, 4.0, 0.0), p(-5.0, 4.0, 0.0),
        p(-5.0, -4.0, 6.0), p(5.0, -4.0, 6.0), p(5.0, 4.0, 6.0), p(-5.0, 4.0, 6.0),
    ];
    let tris: Vec<Triangle> = [
        (0, 2, 1), (0, 3, 2), (4, 5, 6), (4, 6, 7), (0, 1, 5), (0, 5, 4),
        (2, 3, 7), (2, 7, 6), (0, 4, 7), (0, 7, 3), (1, 2, 6), (1, 6, 5),
    ]
    .into_iter()
    .map(|(a, b, c)| Triangle::new(v[a], v[b], v[c]))
    .collect();
    mesh_stock.mask_mesh(&tris);
    assert!((mesh_stock.volume_cm3() - 0.48).abs() < 0.02, "got {:.4}", mesh_stock.volume_cm3());
}

#[test]
fn fast_z_carve_matches_oriented_carve_across_all_shapes_and_axes() {
    // Compare fast +Z / -Z column carving against general `carve_oriented` on identical pocket cuts.
    let shapes = [
        mk_tool(ToolShape::Flat, 6.0, 0.0, 0.0, 20.0, TAG_FACED),
        mk_tool(ToolShape::Ball, 6.0, 0.0, 0.0, 20.0, TAG_BALL),
        mk_tool(ToolShape::Bull, 6.0, 1.5, 0.0, 20.0, TAG_BALL),
        mk_tool(ToolShape::Drill, 6.0, 0.0, 118.0, 20.0, TAG_DRILL),
    ];
    for tool in &shapes {
        let mut fast_pos = VoxelStock::from_box(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0), 0.5);
        let mut fast_neg = VoxelStock::from_box(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0), 0.5);
        let mut side_x = VoxelStock::from_box(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0), 0.5);

        let a = Vec3::new(-4.0, 0.0, 2.0);
        let b = Vec3::new(4.0, 0.0, 2.0);
        let c_pos = fast_pos.sweep_segment(a, b, Vec3::new(0.0, 0.0, 1.0), tool);
        // Mirror along -Z at z = -2.0 -> exact same solid depth carved (8 mm into the cube).
        let c_neg = fast_neg.sweep_segment(Vec3::new(-4.0, 0.0, -2.0), Vec3::new(4.0, 0.0, -2.0), Vec3::new(0.0, 0.0, -1.0), tool);
        // Rotate 90° so spindle points along +X at x = 2.0 (dispatches through `carve_oriented`!).
        let c_side = side_x.sweep_segment(Vec3::new(2.0, -4.0, 0.0), Vec3::new(2.0, 4.0, 0.0), Vec3::new(1.0, 0.0, 0.0), tool);

        assert!(c_pos > 100, "{:?} should carve >100 voxels", tool.shape);
        let diff_neg = (c_pos as i64 - c_neg as i64).abs();
        let diff_side = (c_pos as i64 - c_side as i64).abs();
        assert!(diff_neg <= 4, "{:?}: pos={c_pos} neg={c_neg}", tool.shape);
        // Tiny grid-tie differences on cell centers can differ by <= 1% on curved cones/balls.
        let tol = (c_pos as i64 / 50).max(8);
        assert!(diff_side <= tol, "{:?}: pos={c_pos} side={c_side} (tol={tol})", tool.shape);
    }

    // Tilted 5-axis vector (non-axis-aligned) carves and stays consistent with `solid_count`.
    let mut tilted = VoxelStock::from_box(Vec3::new(-8.0, -8.0, 0.0), Vec3::new(8.0, 8.0, 12.0), 0.5);
    let t = mk_tool(ToolShape::Flat, 4.0, 0.0, 0.0, 14.0, TAG_FACED);
    let ax = Vec3::new(0.6, 0.0, 0.8).norm();
    let carved = tilted.sweep_segment(Vec3::new(-3.0, 0.0, 4.0), Vec3::new(3.0, 0.0, 4.0), ax, &t);
    assert!(carved > 50);
    let actual_solid = tilted.data.iter().filter(|&&v| v != 0).count() as u64;
    assert_eq!(tilted.solid_count, actual_solid);
}

#[test]
fn voxel_surface_tags_and_column_bounds_stay_consistent_after_repeated_cuts() {
    let mut s = VoxelStock::from_box(Vec3::new(0.0, 0.0, 0.0), Vec3::new(12.0, 12.0, 12.0), 1.0);
    let face = mk_tool(ToolShape::Flat, 4.0, 0.0, 0.0, 15.0, TAG_FACED);
    let drill = mk_tool(ToolShape::Flat, 4.0, 0.0, 0.0, 15.0, TAG_DRILL);
    s.sweep_segment(Vec3::new(2.0, 6.0, 9.0), Vec3::new(10.0, 6.0, 9.0), Vec3::new(0.0, 0.0, 1.0), &face);
    s.sweep_segment(Vec3::new(6.0, 6.0, 4.0), Vec3::new(6.0, 6.0, 4.0), Vec3::new(0.0, 0.0, -1.0), &drill);
    // Count solid and verify tags painted onto newly exposed neighbor shells.
    let actual = s.data.iter().filter(|&&v| v != 0).count() as u64;
    assert_eq!(s.solid_count, actual);
    assert!(s.data.contains(&TAG_FACED), "exposed face painted TAG_FACED");
    assert!(s.data.contains(&TAG_DRILL), "exposed hole painted TAG_DRILL");
    assert!(s.data.contains(&TAG_RAW), "untouched interior remains TAG_RAW");
    assert_eq!(s.removed_pct().round() as u32, ((1.0 - actual as f64 / s.initial_count as f64) * 100.0).round() as u32);
    // Outward normal on right wall (+X face) of cube points roughly +X.
    let n = s.smooth_normal(11, 2, 2, Vec3::ZERO);
    assert!(n.x > 0.5, "expected outward +X normal, got {n:?}");
    // Tag albedo entries stay inside normalized linear range.
    for tag in 1..=12 {
        let c = tag_color(tag);
        assert!(c.x >= 0.0 && c.x <= 1.0 && c.y >= 0.0 && c.y <= 1.0 && c.z >= 0.0 && c.z <= 1.0);
    }
    // Lowest display Z of upright cube [0..12]^3 centered at (6,6,6) is -6.
    assert!((s.lowest_display_z(Mat3::from_quat([0.0, 0.0, 0.0, 1.0]), s.center()) - -6.0).abs() < 1e-4);
}

// ---- encode.rs ----------------------------------------------------------------

#[test]
fn video_format_parse_and_resolve_cover_extensions_and_cases() {
    assert_eq!(VideoFormat::parse("AUTO").unwrap(), VideoFormat::Auto);
    assert_eq!(VideoFormat::parse("mp4").unwrap(), VideoFormat::Mp4);
    assert_eq!(VideoFormat::parse("WebM").unwrap(), VideoFormat::Webm);
    assert_eq!(VideoFormat::parse("gif").unwrap(), VideoFormat::Gif);
    assert_eq!(VideoFormat::parse("frames").unwrap(), VideoFormat::Frames);
    assert_eq!(VideoFormat::parse("PNG").unwrap(), VideoFormat::Frames);
    assert!(VideoFormat::parse("mkv").is_err());

    assert_eq!(VideoFormat::Auto.resolve(Path::new("out.mp4")), VideoFormat::Mp4);
    assert_eq!(VideoFormat::Auto.resolve(Path::new("out.MOV")), VideoFormat::Mp4);
    assert_eq!(VideoFormat::Auto.resolve(Path::new("out.m4v")), VideoFormat::Mp4);
    assert_eq!(VideoFormat::Auto.resolve(Path::new("out.webm")), VideoFormat::Webm);
    assert_eq!(VideoFormat::Auto.resolve(Path::new("out.gif")), VideoFormat::Gif);
    assert_eq!(VideoFormat::Auto.resolve(Path::new("frames_dir")), VideoFormat::Frames);
    assert_eq!(VideoFormat::Webm.resolve(Path::new("out.mp4")), VideoFormat::Webm);
}

#[test]
fn per_setup_path_sanitizes_names_for_videos_and_frame_dirs() {
    assert_eq!(
        per_setup_path(Path::new("/tmp/sim.mp4"), VideoFormat::Mp4, 0, "OP10 / Top: Face"),
        PathBuf::from("/tmp/sim_01_OP10___Top__Face.mp4")
    );
    assert_eq!(
        per_setup_path(Path::new("/tmp/sim.webm"), VideoFormat::Auto, 2, ""),
        PathBuf::from("/tmp/sim_03_setup.webm")
    );
    assert_eq!(
        per_setup_path(Path::new("/tmp/out_frames"), VideoFormat::Frames, 1, "OP20_Bottom"),
        PathBuf::from("/tmp/out_frames/02_OP20_Bottom")
    );
}

#[test]
fn write_png_creates_valid_rgb8_image() {
    let p = std::env::temp_dir().join(format!("camcraft-sv-png-{}.png", std::process::id()));
    let rgb = vec![128u8; 8 * 6 * 3];
    write_png(&p, 8, 6, &rgb).expect("write_png");
    let raw = std::fs::read(&p).expect("read png");
    let _ = std::fs::remove_file(&p);
    assert_eq!(&raw[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(u32::from_be_bytes(raw[16..20].try_into().unwrap()), 8);
    assert_eq!(u32::from_be_bytes(raw[20..24].try_into().unwrap()), 6);
}

// ---- hud.rs -------------------------------------------------------------------

#[test]
fn hud_is_safe_on_tiny_frames_and_narrow_footer_avoids_overlap() {
    // Tiny frame below HUD threshold (<320x180) safely skips without panic or change.
    let mut tiny = vec![42u8; 64 * 64 * 3];
    let bars = [OpBar { first_seg: 0, end_seg: 10, color: [80, 180, 255] }];
    let hud = HudInfo {
        setup_title: "SETUP 1/3: Very_Long_Setup_Name (G54)",
        op_title: "OP 1/1: Test",
        tool_badge: "T01 Flat EM D10.0",
        rpm: 8000.0,
        feed: 1200.0,
        is_rapid: false,
        show_tool: true,
        wcs_pos: [1.0, 2.0, 3.0],
        remaining_cm3: 99.5,
        removed_pct: 50.0,
        progress_frac: 0.5,
        footer_left: "CamCraft 3D Stock Simulation  |  blue:feed  amber:plunge  red:rapid",
        footer_right: "Stock 100x60x20mm  |  CAD 67.4cm3  |  Cell 0.50mm",
        banner: "COMPLETE",
        move_num: 10,
        bars: &bars,
        total_segs: 10,
    };
    draw_hud(&mut tiny, 64, 64, &hud);
    assert!(tiny.iter().all(|&b| b == 42), "HUD skipped under 320x180");

    // Out-of-bounds fill_rect never panics.
    fill_rect(&mut tiny, 64, 64, -50, -50, 200, 200, [10, 20, 30], 0.5);

    // Narrow 640x360 frame collapses footer_left to "CamCraft Sim" so left & right don't collide.
    let mut narrow = vec![0u8; 640 * 360 * 3];
    draw_hud(&mut narrow, 640, 360, &hud);
    // Column x in [120..280] at footer text row y = 360 - 35 + 3 = 328 sits strictly in the blank gap
    // between "CamCraft Sim" (ends at x=18+72=90) and footer_right (starts at x=640-18-300=322).
    let row_y = 328usize;
    let gap_bright = (120..280usize).any(|x| narrow[(row_y * 640 + x) * 3] > 60);
    assert!(!gap_bright, "unexpected glyph overlap inside 640px footer gap");
}

#[test]
fn draw_line_2d_dims_segments_behind_depth_buffer() {
    let (w, h) = (32u32, 32u32);
    let mut front_rgb = vec![0u8; (w * h * 3) as usize];
    let mut back_rgb = vec![0u8; (w * h * 3) as usize];
    let zbuf = vec![10.0f32; (w * h) as usize];
    draw_line_2d(&mut front_rgb, &zbuf, w, h, 2.0, 16.0, 5.0, 28.0, 16.0, 5.0, [200, 200, 200], 0.8);
    draw_line_2d(&mut back_rgb, &zbuf, w, h, 2.0, 16.0, 20.0, 28.0, 16.0, 20.0, [200, 200, 200], 0.8);
    let p_front = front_rgb[(16 * w as usize + 15) * 3];
    let p_back = back_rgb[(16 * w as usize + 15) * 3];
    assert!(p_front > 120 && p_back > 0 && p_front > p_back * 4, "front={p_front} back={p_back}");
}

// ---- mod.rs helpers -----------------------------------------------------------

#[test]
fn camera_views_validation_and_op_frame_budgeting() {
    assert_eq!(CameraView::parse("iso").unwrap(), CameraView::Iso);
    assert_eq!(CameraView::parse("ISOMETRIC").unwrap(), CameraView::Iso);
    assert_eq!(CameraView::parse("front").unwrap(), CameraView::FrontRight);
    assert_eq!(CameraView::parse("front-right").unwrap(), CameraView::FrontRight);
    assert_eq!(CameraView::parse("top").unwrap(), CameraView::Top);
    assert_eq!(CameraView::parse("side").unwrap(), CameraView::Side);
    assert!(CameraView::parse("bottom").is_err());

    let bad_empty = VideoOptions { output: PathBuf::new(), ..Default::default() };
    assert!(validate_options(&bad_empty).is_err());
    let bad_w = VideoOptions { width: 32, ..Default::default() };
    assert!(validate_options(&bad_w).is_err());
    let bad_h_odd_webm = VideoOptions { output: PathBuf::from("a.webm"), width: 320, height: 181, ..Default::default() };
    assert!(validate_options(&bad_h_odd_webm).is_err());
    let ok_odd_frames = VideoOptions { format: VideoFormat::Frames, width: 101, height: 81, ..Default::default() };
    assert!(validate_options(&ok_odd_frames).is_ok());
    let bad_fps = VideoOptions { fps: 0, ..Default::default() };
    assert!(validate_options(&bad_fps).is_err());
    let bad_sec = VideoOptions { seconds: Some(-1.0), ..Default::default() };
    assert!(validate_options(&bad_sec).is_err());
    let bad_spin = VideoOptions { spin_seconds: Some(f64::NAN), ..Default::default() };
    assert!(validate_options(&bad_spin).is_err());

    // Frame allocation gives short ops breathing room alongside huge 3D surfacing ops.
    let dummy_seg = VideoSeg {
        a_part: Vec3::ZERO,
        b_part: Vec3::new(1.0, 0.0, 0.0),
        axis_part: None,
        wcs_b: [0.0; 3],
        is_rapid: false,
        is_plunge: false,
        feed: 1000.0,
        move_no: 1,
    };
    let ops = vec![
        VideoOp { name: "short".into(), kind_label: "Drill".into(), tool: mk_tool(ToolShape::Drill, 5.0, 0.0, 118.0, 10.0, TAG_DRILL), rpm: 3000.0, segs: vec![dummy_seg.clone(); 4] },
        VideoOp { name: "huge".into(), kind_label: "Ball".into(), tool: mk_tool(ToolShape::Ball, 8.0, 0.0, 0.0, 10.0, TAG_BALL), rpm: 9000.0, segs: vec![dummy_seg; 40_000] },
    ];
    let counts = allocate_op_frames(&ops, 300);
    assert_eq!(counts.len(), 2);
    assert!(counts[0] >= 6, "short op kept readable: {:?}", counts);
    assert!(counts[1] > counts[0] * 10, "huge op gets lion's share: {:?}", counts);
}

#[test]
fn setup_az_bias_stays_zero_when_centered_and_turns_toward_rear_side_cuts() {
    let tool = mk_tool(ToolShape::Flat, 10.0, 0.0, 0.0, 20.0, TAG_FACED);
    let mk_setup = |pt: Vec3| VideoSetup {
        name: "S".into(),
        wcs_label: "G54".into(),
        rot_q: [0.0, 0.0, 0.0, 1.0],
        spindle_axis_part: Vec3::new(0.0, 0.0, 1.0),
        ops: vec![VideoOp {
            name: "op".into(),
            kind_label: "Contour".into(),
            tool: tool.clone(),
            rpm: 6000.0,
            segs: vec![VideoSeg {
                a_part: pt,
                b_part: pt,
                axis_part: None,
                wcs_b: [0.0; 3],
                is_rapid: false,
                is_plunge: false,
                feed: 1000.0,
                move_no: 1,
            }],
        }],
    };
    let span = Vec3::new(100.0, 70.0, 70.0);
    let rot_id = Mat3::from_quat([0.0, 0.0, 0.0, 1.0]);
    // Centered cut stays at default iso angle (0° bias).
    assert_eq!(setup_az_bias(CameraView::Iso, &mk_setup(Vec3::ZERO), rot_id, Vec3::ZERO, span), 0.0);
    // Cut on the far side (-X, +Y) relative to eye (+X, -Y) swings camera onto the matching side.
    let rear_bias = setup_az_bias(CameraView::Iso, &mk_setup(Vec3::new(-30.0, 0.0, 0.0)), rot_id, Vec3::ZERO, span);
    assert!(rear_bias.abs() > 45.0, "expected large azimuth shift toward -X ears, got {rear_bias}");
    // Top-down view never orbits azimuth.
    assert_eq!(setup_az_bias(CameraView::Top, &mk_setup(Vec3::new(-30.0, 0.0, 0.0)), rot_id, Vec3::ZERO, span), 0.0);
}

#[test]
fn render_frame_draws_all_five_tool_shapes_sparks_and_trail() {
    use super::render::{render_frame, FrameParams, TrailSeg};
    let stock = VoxelStock::from_box(Vec3::new(-20.0, -20.0, 0.0), Vec3::new(20.0, 20.0, 10.0), 2.0);
    let cam = Camera::look_at(Vec3::new(40.0, -45.0, 35.0), Vec3::ZERO, 40.0, 96, 64);
    let center = stock.center();
    let rot = Mat3::from_quat([0.0, 0.0, 0.0, 1.0]);
    let mut base_rgb = Vec::new();
    let mut zbuf = Vec::new();
    render_frame(
        &FrameParams {
            stock: &stock,
            cam,
            part_rot: rot,
            part_center: center,
            spindle_axis_part: Vec3::new(0.0, 0.0, 1.0),
            show_tool: false,
            tool_tip_part: Vec3::ZERO,
            tool: None,
            spin_phase: 0.0,
            is_cutting: false,
            show_toolpath: false,
            trail: &[],
            hud: None,
            table_z: -6.0,
        },
        &mut base_rgb,
        &mut zbuf,
    );
    assert_eq!(base_rgb.len(), 96 * 64 * 3);

    let trail = [
        TrailSeg { a_part: Vec3::new(-10.0, 0.0, 12.0), b_part: Vec3::new(0.0, 0.0, 12.0), is_rapid: true, is_plunge: false },
        TrailSeg { a_part: Vec3::new(0.0, 0.0, 12.0), b_part: Vec3::new(0.0, 0.0, 9.0), is_rapid: false, is_plunge: true },
        TrailSeg { a_part: Vec3::new(0.0, 0.0, 9.0), b_part: Vec3::new(10.0, 0.0, 9.0), is_rapid: false, is_plunge: false },
    ];
    let tools = [
        mk_tool(ToolShape::Flat, 8.0, 0.0, 0.0, 16.0, TAG_FACED),
        mk_tool(ToolShape::Ball, 8.0, 0.0, 0.0, 16.0, TAG_BALL),
        mk_tool(ToolShape::Bull, 8.0, 2.0, 0.0, 16.0, TAG_BALL),
        mk_tool(ToolShape::Drill, 8.0, 0.0, 118.0, 16.0, TAG_DRILL),
        mk_tool(ToolShape::Chamfer, 8.0, 0.0, 90.0, 16.0, TAG_CHAMFER),
    ];
    for tool in &tools {
        let mut rgb = Vec::new();
        render_frame(
            &FrameParams {
                stock: &stock,
                cam,
                part_rot: rot,
                part_center: center,
                spindle_axis_part: Vec3::new(0.0, 0.0, 1.0),
                show_tool: true,
                tool_tip_part: Vec3::new(0.0, 0.0, 9.5),
                tool: Some(tool),
                spin_phase: 0.7,
                is_cutting: true,
                show_toolpath: true,
                trail: &trail,
                hud: None,
                table_z: -6.0,
            },
            &mut rgb,
            &mut zbuf,
        );
        assert_ne!(rgb, base_rgb, "{:?} tool+trail+glow should differ from bare stock frame", tool.shape);
    }
}

