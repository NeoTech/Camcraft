//! Built-in headless 3D multi-setup stock-simulation movie renderer (`sim_video`).
//!
//! Renders material removal, spinning cutting tools/holders, depth-tested 3D toolpath trails,
//! smooth setup re-orientation transitions (quaternion SLERP), optional per-setup split videos,
//! turntable end inspection spins, and PNG keyframes directly from a generated CamCraft `World`.
//!
//! Outputs pure-Rust PNG frame sequences (`format = "frames"`, zero external tools) or streams
//! raw RGB24 frames to `ffmpeg` for `.mp4`, `.webm`, and `.gif`.
//!
//! Provenance: clean-room 3D voxel billet & DDA ray-marcher built over CamCraft's own ECS world
//! and `camcraft-cl` chord tessellation (`Move::segments3`).

mod encode;
mod hud;
mod math;
mod plan;
mod render;
mod voxel;

#[cfg(test)]
mod tests;

pub use encode::VideoFormat;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use flecs_ecs::prelude::World;
use serde::Serialize;

use encode::{per_setup_path, write_png, FrameSink};
use hud::{HudInfo, OpBar};
use math::{norm_quat, quat_slerp, Camera, Mat3, Vec3};
use plan::{gather_plan, VideoOp, VideoSetup};
use render::{render_frame, FrameParams, TrailSeg};
use voxel::{bar_color, VoxelStock};

pub const MIN_EDGE: u32 = 64;
pub const MAX_EDGE: u32 = 2048;
pub const DEFAULT_WIDTH: u32 = 960;
pub const DEFAULT_HEIGHT: u32 = 540;
pub const DEFAULT_FPS: u32 = 30;
const MAX_TRAIL_SEGS: usize = 340;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CameraView {
    #[default]
    Iso,
    FrontRight,
    Top,
    Side,
}

impl CameraView {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "iso" | "isometric" => Ok(Self::Iso),
            "front" | "front-right" | "front_right" => Ok(Self::FrontRight),
            "top" => Ok(Self::Top),
            "side" => Ok(Self::Side),
            other => Err(format!("unsupported view \"{other}\" (expected iso, front-right, top, or side)")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Iso => "iso",
            Self::FrontRight => "front-right",
            Self::Top => "top",
            Self::Side => "side",
        }
    }

    fn angles_deg(self) -> (f32, f32) {
        match self {
            Self::Iso => (-48.0, 31.0),
            Self::FrontRight => (-28.0, 22.0),
            Self::Top => (-90.0, 82.0),
            Self::Side => (-12.0, 14.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VideoOptions {
    pub output: PathBuf,
    pub format: VideoFormat,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub cell_mm: Option<f64>,
    pub setup: Option<String>,
    pub per_setup: bool,
    pub toolpath: bool,
    pub hud: bool,
    pub view: CameraView,
    /// Approximate total video cutting time in seconds (`None` = auto based on setup/op count).
    pub seconds: Option<f64>,
    /// Transition flip/turn animation duration between setups in seconds (`None` = default 1.2s).
    pub transition_seconds: Option<f64>,
    /// End-of-program turntable showcase spin duration in seconds (`None` = default 3.0s).
    pub spin_seconds: Option<f64>,
    /// Optional directory where representative PNG keyframes are written during simulation.
    pub keyframes_dir: Option<PathBuf>,
}

impl Default for VideoOptions {
    fn default() -> Self {
        Self {
            output: PathBuf::from("simulation.mp4"),
            format: VideoFormat::Auto,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            fps: DEFAULT_FPS,
            cell_mm: None,
            setup: None,
            per_setup: false,
            toolpath: true,
            hud: true,
            view: CameraView::Iso,
            seconds: None,
            transition_seconds: None,
            spin_seconds: None,
            keyframes_dir: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SetupReport {
    pub name: String,
    pub wcs: String,
    pub operations: usize,
    pub segments: usize,
    pub rendered: bool,
    pub frames: usize,
    pub remaining_cm3: f64,
    pub removed_pct: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct VideoReport {
    pub output: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub view: String,
    pub total_frames: usize,
    pub duration_s: f64,
    pub cell_mm: f64,
    pub grid: [usize; 3],
    pub initial_cm3: f64,
    pub remaining_cm3: f64,
    pub removed_pct: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_volume_cm3: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_size_mm: Option<[f64; 3]>,
    pub per_setup_files: Vec<String>,
    pub keyframes: Vec<String>,
    pub setups: Vec<SetupReport>,
}

fn validate_options(opts: &VideoOptions) -> Result<VideoFormat, String> {
    if opts.output.as_os_str().is_empty() {
        return Err("output path must not be empty".into());
    }
    if !(MIN_EDGE..=MAX_EDGE).contains(&opts.width) {
        return Err(format!("width must be in {MIN_EDGE}..={MAX_EDGE}, got {}", opts.width));
    }
    if !(MIN_EDGE..=MAX_EDGE).contains(&opts.height) {
        return Err(format!("height must be in {MIN_EDGE}..={MAX_EDGE}, got {}", opts.height));
    }
    if !(1..=120).contains(&opts.fps) {
        return Err(format!("fps must be in 1..=120, got {}", opts.fps));
    }
    let fmt = opts.format.resolve(&opts.output);
    if matches!(fmt, VideoFormat::Mp4 | VideoFormat::Webm) && (!opts.width.is_multiple_of(2) || !opts.height.is_multiple_of(2)) {
        return Err(format!(
            "{} encoding requires even width and height (got {}x{})",
            fmt.as_str(),
            opts.width,
            opts.height
        ));
    }
    for (label, val) in [
        ("seconds", opts.seconds),
        ("transition_seconds", opts.transition_seconds),
        ("spin_seconds", opts.spin_seconds),
    ] {
        if let Some(v) = val
            && (!v.is_finite() || v < 0.0 || v > 600.0)
        {
            return Err(format!("{label} must be finite and in 0..=600, got {v}"));
        }
    }
    Ok(fmt)
}

#[inline]
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[inline]
fn smoothstep(t: f32) -> f32 {
    let x = t.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn orbit_camera(view: CameraView, span: Vec3, az_offset_deg: f32, elev_offset_deg: f32, w: u32, h: u32) -> Camera {
    let r_sphere = (span.len() * 0.5).max(1.0);
    let fov_deg = 36.0f32;
    let half_tan = (fov_deg * 0.5).to_radians().tan();
    let aspect = (w as f32 / h.max(1) as f32).min(1.0);
    let dist = r_sphere / (half_tan * aspect) * 1.32;
    let (base_az, base_el) = view.angles_deg();
    let az = (base_az + az_offset_deg).to_radians();
    let el = (base_el + elev_offset_deg).clamp(-20.0, 86.0).to_radians();
    let eye = Vec3::new(
        dist * el.cos() * az.cos(),
        dist * el.cos() * az.sin(),
        dist * el.sin() + r_sphere * 0.14,
    );
    let target = Vec3::new(0.0, 0.0, r_sphere * 0.04);
    Camera::look_at(eye, target, fov_deg, w, h)
}

/// Choose a slight horizontal azimuth offset for asymmetric setups (e.g. a quarter-turned fixture
/// whose machined features all sit on the far side of the billet from the default eye).
fn setup_az_bias(view: CameraView, setup: &VideoSetup, rot: Mat3, center: Vec3, span: Vec3) -> f32 {
    if view == CameraView::Top {
        return 0.0;
    }
    let (mut sx, mut sy, mut n) = (0.0f64, 0.0f64, 0u64);
    for op in &setup.ops {
        let step = (op.segs.len() / 256).max(1);
        for seg in op.segs.iter().step_by(step) {
            if !seg.is_rapid {
                let d = rot.mul(seg.b_part.sub(center));
                sx += f64::from(d.x);
                sy += f64::from(d.y);
                n += 1;
            }
        }
    }
    if n == 0 {
        return 0.0;
    }
    let mx = (sx / n as f64) as f32;
    let my = (sy / n as f64) as f32;
    let max_span = span.x.max(span.y).max(span.z).max(1.0);
    if mx.hypot(my) < max_span * 0.06 {
        return 0.0;
    }
    let (base_az, _) = view.angles_deg();
    let base_rad = base_az.to_radians();
    let dot = mx * base_rad.cos() + my * base_rad.sin();
    if dot >= 0.0 {
        return 0.0;
    }
    // Mirror camera onto the hemisphere containing the cutting centroid, offset ~38° for 3/4 view.
    let target_az = my.atan2(mx).to_degrees() + 38.0;
    let mut diff = (target_az - base_az) % 360.0;
    if diff > 180.0 { diff -= 360.0; }
    if diff < -180.0 { diff += 360.0; }
    diff
}

fn sanitize_tag(s: &str) -> String {
    let t: String = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    if t.is_empty() { "setup".into() } else { t }
}

fn setup_bars(setup: &VideoSetup) -> (Vec<OpBar>, usize) {
    let mut bars = Vec::with_capacity(setup.ops.len());
    let mut cur = 0usize;
    for op in &setup.ops {
        let len = op.segs.len();
        bars.push(OpBar {
            first_seg: cur,
            end_seg: cur + len,
            color: bar_color(op.tool.tag),
        });
        cur += len;
    }
    (bars, cur)
}

/// Distribute `budget` cutting frames across a setup's ops with `sqrt(len)` weighting so both
/// quick drills/contours and huge 3D ball-nose passes remain readable.
fn allocate_op_frames(ops: &[VideoOp], budget: usize) -> Vec<usize> {
    if ops.is_empty() {
        return Vec::new();
    }
    let weights: Vec<f64> = ops.iter().map(|o| (o.segs.len().max(1) as f64).sqrt()).collect();
    let sum: f64 = weights.iter().sum::<f64>().max(1e-6);
    let floor = if budget >= ops.len() * 6 { 6 } else { 1 };
    let mut counts: Vec<usize> = weights
        .iter()
        .map(|w| ((w / sum) * budget as f64).round().max(floor as f64) as usize)
        .collect();
    for (c, op) in counts.iter_mut().zip(ops) {
        // Keep tiny single-move ops bounded to their natural length unless spinning minimally.
        *c = (*c).clamp(1, (op.segs.len() * 8).max(4));
    }
    counts
}

fn save_keyframe(
    dir: Option<&Path>,
    name: &str,
    w: u32,
    h: u32,
    rgb: &[u8],
    out: &mut Vec<String>,
) -> Result<(), String> {
    if let Some(d) = dir {
        let path = d.join(format!("{name}.png"));
        write_png(&path, w, h, rgb)?;
        out.push(path.display().to_string());
    }
    Ok(())
}

fn push_frame(
    rgb: &[u8],
    main: &mut FrameSink,
    side: Option<&mut FrameSink>,
    total: &mut usize,
    setup_frames: &mut usize,
) -> Result<(), String> {
    main.write_frame(rgb)?;
    *total += 1;
    if let Some(s) = side {
        s.write_frame(rgb)?;
    }
    *setup_frames += 1;
    Ok(())
}

fn quats_differ(a: [f32; 4], b: [f32; 4]) -> bool {
    let a = norm_quat(a);
    let b = norm_quat(b);
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]).abs() < 0.999
}

struct SpinCtx<'a> {
    stock: &'a VoxelStock,
    center: Vec3,
    span: Vec3,
    view: CameraView,
    w: u32,
    h: u32,
    show_hud: bool,
    base_q: [f32; 4],
    upright_q: Option<[f32; 4]>,
    setup_title: &'a str,
    banner: &'a str,
    footer_left: &'a str,
    footer_right: &'a str,
    bars: &'a [OpBar],
    total_segs: usize,
    rgb: &'a mut Vec<u8>,
    zbuf: &'a mut Vec<f32>,
}

fn emit_spin_frames(
    ctx: &mut SpinCtx<'_>,
    frames: usize,
    az_start: f32,
    sweep_deg: f32,
    mut on_frame: impl FnMut(usize, &[u8]) -> Result<(), String>,
) -> Result<(), String> {
    if frames == 0 {
        return Ok(());
    }
    let rem = ctx.stock.volume_cm3();
    let pct = ctx.stock.removed_pct();
    for f in 0..frames {
        let u = (f as f32 + 1.0) / frames as f32;
        let q = match ctx.upright_q {
            Some(up) if u > 0.25 => {
                let s = smoothstep((u - 0.25) / 0.28);
                quat_slerp(ctx.base_q, up, s)
            }
            _ => ctx.base_q,
        };
        let rot = Mat3::from_quat(q);
        let az = if ctx.upright_q.is_some() {
            az_start + (sweep_deg - az_start) * u
        } else {
            az_start + sweep_deg * u
        };
        let cam = orbit_camera(ctx.view, ctx.span, az, 6.0 * (u * std::f32::consts::PI).sin(), ctx.w, ctx.h);
        let table_z = ctx.stock.lowest_display_z(rot, ctx.center) - 1.2;
        let hud = ctx.show_hud.then_some(HudInfo {
            setup_title: ctx.setup_title,
            op_title: "FINAL STOCK INSPECTION",
            tool_badge: "",
            rpm: 0.0,
            feed: 0.0,
            is_rapid: false,
            show_tool: false,
            wcs_pos: [0.0; 3],
            remaining_cm3: rem,
            removed_pct: pct,
            progress_frac: 1.0,
            footer_left: ctx.footer_left,
            footer_right: ctx.footer_right,
            banner: ctx.banner,
            move_num: ctx.total_segs,
            bars: ctx.bars,
            total_segs: ctx.total_segs,
        });
        render_frame(
            &FrameParams {
                stock: ctx.stock,
                cam,
                part_rot: rot,
                part_center: ctx.center,
                spindle_axis_part: Vec3::new(0.0, 0.0, 1.0),
                show_tool: false,
                tool_tip_part: Vec3::ZERO,
                tool: None,
                spin_phase: 0.0,
                is_cutting: false,
                show_toolpath: false,
                trail: &[],
                hud,
                table_z,
            },
            ctx.rgb,
            ctx.zbuf,
        );
        on_frame(f, ctx.rgb)?;
    }
    Ok(())
}

/// Simulate every generated operation in program order across multi-setup orientations and write
/// a combined movie (plus optional per-setup split videos and PNG keyframes).
pub fn render_simulation_video(world: &World, opts: &VideoOptions) -> Result<VideoReport, String> {
    let resolved_fmt = validate_options(opts)?;
    let mut plan = gather_plan(world, opts.cell_mm)?;
    let target_idx: Option<usize> = match opts.setup.as_deref() {
        None => None,
        Some(wanted) => {
            let pos = plan.setups.iter().position(|s| s.name == wanted);
            match pos {
                Some(i) => Some(i),
                None => {
                    let names: Vec<&str> = plan.setups.iter().map(|s| s.name.as_str()).collect();
                    return Err(format!("setup \"{wanted}\" not found (available: {})", names.join(", ")));
                }
            }
        }
    };
    let rendered_setups: usize = if target_idx.is_some() { 1 } else { plan.setups.len() };
    let total_cut_s = opts.seconds.unwrap_or_else(|| {
        plan.setups
            .iter()
            .enumerate()
            .filter(|(i, _)| target_idx.is_none_or(|t| *i == t))
            .map(|(_, s)| (s.ops.len() as f64 * 2.8).clamp(4.5, 16.0))
            .sum::<f64>()
    });
    let total_cut_frames = (total_cut_s * f64::from(opts.fps)).round().max(rendered_setups as f64) as usize;
    let trans_frames = (opts.transition_seconds.unwrap_or(1.1) * f64::from(opts.fps)).round() as usize;
    let spin_frames = (opts.spin_seconds.unwrap_or(3.0) * f64::from(opts.fps)).round() as usize;
    let hold_frames = if opts.seconds.is_some_and(|s| s <= 1.0) { 0 } else { (opts.fps / 3).max(1) as usize };

    let mut main_sink = FrameSink::open(&opts.output, resolved_fmt, opts.width, opts.height, opts.fps)?;
    let mut rgb = Vec::new();
    let mut zbuf = Vec::new();
    let center = plan.stock.center();
    let span = plan.stock.span_vec();
    let init_cm3 = plan.stock.initial_cm3();
    let first_q = plan.setups[0].rot_q;
    let footer_right = match plan.part_volume_cm3 {
        Some(pv) => format!("{}  |  CAD {pv:.1}cm3  |  Cell {:.2}mm", plan.stock_label, plan.stock.cell),
        None => format!("{}  |  Cell {:.2}mm", plan.stock_label, plan.stock.cell),
    };
    let rendered_segs_sum: f64 = plan
        .setups
        .iter()
        .enumerate()
        .filter(|(i, _)| target_idx.is_none_or(|t| *i == t))
        .map(|(_, s)| (s.ops.len().max(1) as f64).powf(0.8))
        .sum::<f64>()
        .max(1e-6);

    let mut total_frames = 0usize;
    let mut spin_phase = 0.0f32;
    let mut prev_q: Option<[f32; 4]> = None;
    let mut prev_az_bias = 0.0f32;
    let mut per_setup_files = Vec::new();
    let mut keyframes = Vec::new();
    let mut reports = Vec::with_capacity(plan.setups.len());
    let kf_dir = opts.keyframes_dir.as_deref();
    let mut trail_buf: VecDeque<TrailSeg> = VecDeque::with_capacity(MAX_TRAIL_SEGS);
    let mut trail_slice: Vec<TrailSeg> = Vec::with_capacity(MAX_TRAIL_SEGS);

    for (si, setup) in plan.setups.iter().enumerate() {
        let seg_total: usize = setup.ops.iter().map(|o| o.segs.len()).sum();
        if let Some(t) = target_idx
            && si > t
        {
            break;
        }
        let render_this = target_idx.is_none_or(|t| si == t);
        if !render_this {
            // Silently carve earlier setups so the requested setup starts from carried stock.
            for op in &setup.ops {
                for seg in &op.segs {
                    if !seg.is_rapid {
                        let ax = seg.axis_part.unwrap_or(setup.spindle_axis_part);
                        plan.stock.sweep_segment(seg.a_part, seg.b_part, ax, &op.tool);
                    }
                }
            }
            reports.push(SetupReport {
                name: setup.name.clone(),
                wcs: setup.wcs_label.clone(),
                operations: setup.ops.len(),
                segments: seg_total,
                rendered: false,
                frames: 0,
                remaining_cm3: round2(plan.stock.volume_cm3()),
                removed_pct: round2(plan.stock.removed_pct()),
                file: None,
            });
            continue;
        }

        let stag = format!("{:02}_{}", si + 1, sanitize_tag(&setup.name));
        let mut side_file: Option<String> = None;
        let mut side_sink: Option<FrameSink> = if opts.per_setup && target_idx.is_none() {
            let sp = per_setup_path(&opts.output, resolved_fmt, si, &setup.name);
            let s = FrameSink::open(&sp, resolved_fmt, opts.width, opts.height, opts.fps)?;
            side_file = Some(sp.display().to_string());
            per_setup_files.push(sp.display().to_string());
            Some(s)
        } else {
            None
        };

        let setup_title = format!("SETUP {}/{}: {} ({})", si + 1, plan.setups.len(), setup.name, setup.wcs_label);
        let footer_left = "CamCraft 3D Stock Simulation  |  blue:feed  amber:plunge  red:rapid";
        let (bars, total_segs) = setup_bars(setup);
        let mut setup_frames = 0usize;

        let rot = Mat3::from_quat(setup.rot_q);
        let cur_az_bias = setup_az_bias(opts.view, setup, rot, center, span);

        // Smooth flip/turn transition between differently oriented setups.
        if let Some(pq) = prev_q
            && quats_differ(pq, setup.rot_q)
            && trans_frames > 0
        {
            let banner = format!("M0 STOP - RE-ORIENT PART -> {} ({})", setup.name, setup.wcs_label);
            let rem = plan.stock.volume_cm3();
            let pct = plan.stock.removed_pct();
            for f in 0..trans_frames {
                let t = smoothstep((f as f32 + 1.0) / trans_frames as f32);
                let q = quat_slerp(pq, setup.rot_q, t);
                let t_rot = Mat3::from_quat(q);
                let az = prev_az_bias + (cur_az_bias - prev_az_bias) * t - 10.0 * t;
                let cam = orbit_camera(opts.view, span, az, 4.0 * (t * std::f32::consts::PI).sin(), opts.width, opts.height);
                let table_z = plan.stock.lowest_display_z(t_rot, center) - 1.2;
                let hud = opts.hud.then_some(HudInfo {
                    setup_title: &setup_title,
                    op_title: "RE-CLAMPING WORKPIECE...",
                    tool_badge: "",
                    rpm: 0.0,
                    feed: 0.0,
                    is_rapid: false,
                    show_tool: false,
                    wcs_pos: [0.0; 3],
                    remaining_cm3: rem,
                    removed_pct: pct,
                    progress_frac: 0.0,
                    footer_left,
                    footer_right: &footer_right,
                    banner: &banner,
                    move_num: 0,
                    bars: &bars,
                    total_segs,
                });
                render_frame(
                    &FrameParams {
                        stock: &plan.stock,
                        cam,
                        part_rot: t_rot,
                        part_center: center,
                        spindle_axis_part: setup.spindle_axis_part,
                        show_tool: false,
                        tool_tip_part: Vec3::ZERO,
                        tool: None,
                        spin_phase: 0.0,
                        is_cutting: false,
                        show_toolpath: false,
                        trail: &[],
                        hud,
                        table_z,
                    },
                    &mut rgb,
                    &mut zbuf,
                );
                main_sink.write_frame(&rgb)?;
                total_frames += 1;
            }
        }
        prev_q = Some(setup.rot_q);
        prev_az_bias = cur_az_bias + 18.0;

        let table_z = plan.stock.lowest_display_z(rot, center) - 1.2;
        if hold_frames > 0 {
            let cam = orbit_camera(opts.view, span, cur_az_bias - 10.0, 0.0, opts.width, opts.height);
            let hud = opts.hud.then_some(HudInfo {
                setup_title: &setup_title,
                op_title: "INCOMING STOCK",
                tool_badge: "",
                rpm: 0.0,
                feed: 0.0,
                is_rapid: false,
                show_tool: false,
                wcs_pos: [0.0; 3],
                remaining_cm3: plan.stock.volume_cm3(),
                removed_pct: plan.stock.removed_pct(),
                progress_frac: 0.0,
                footer_left,
                footer_right: &footer_right,
                banner: "",
                move_num: 0,
                bars: &bars,
                total_segs,
            });
            render_frame(
                &FrameParams {
                    stock: &plan.stock,
                    cam,
                    part_rot: rot,
                    part_center: center,
                    spindle_axis_part: setup.spindle_axis_part,
                    show_tool: false,
                    tool_tip_part: Vec3::ZERO,
                    tool: None,
                    spin_phase: 0.0,
                    is_cutting: false,
                    show_toolpath: false,
                    trail: &[],
                    hud,
                    table_z,
                },
                &mut rgb,
                &mut zbuf,
            );
            save_keyframe(kf_dir, &format!("{stag}_start"), opts.width, opts.height, &rgb, &mut keyframes)?;
            for _ in 0..hold_frames {
                push_frame(&rgb, &mut main_sink, side_sink.as_mut(), &mut total_frames, &mut setup_frames)?;
            }
        }

        let share = ((setup.ops.len().max(1) as f64).powf(0.8) / rendered_segs_sum * total_cut_frames as f64)
            .round()
            .max(setup.ops.len() as f64) as usize;
        let op_frames = allocate_op_frames(&setup.ops, share);
        let mut segs_done = 0usize;
        trail_buf.clear();

        for (oi, op) in setup.ops.iter().enumerate() {
            let nf = op_frames[oi].max(1);
            let op_n = op.segs.len();
            if op_n == 0 {
                continue;
            }
            let op_title = format!("OP {}/{}: {} [{}]", oi + 1, setup.ops.len(), op.name, op.kind_label);
            let mut cur_idx = 0usize;
            let mid_f = nf / 2;
            trail_buf.clear();

            for f in 0..nf {
                let next_idx = ((f + 1) * op_n).div_ceil(nf).min(op_n);
                let batch = next_idx.saturating_sub(cur_idx);
                let trail_stride = (batch / 10).max(1);
                let mut carved_any = false;
                for i in cur_idx..next_idx {
                    let seg = &op.segs[i];
                    if !seg.is_rapid {
                        let ax = seg.axis_part.unwrap_or(setup.spindle_axis_part);
                        if plan.stock.sweep_segment(seg.a_part, seg.b_part, ax, &op.tool) > 0 {
                            carved_any = true;
                        }
                    }
                    if opts.toolpath && ((i - cur_idx).is_multiple_of(trail_stride) || i + 1 == next_idx) {
                        if trail_buf.len() >= MAX_TRAIL_SEGS {
                            trail_buf.pop_front();
                        }
                        trail_buf.push_back(TrailSeg {
                            a_part: seg.a_part,
                            b_part: seg.b_part,
                            is_rapid: seg.is_rapid,
                            is_plunge: seg.is_plunge,
                        });
                    }
                }
                let last_i = next_idx.saturating_sub(1);
                let last_seg = &op.segs[last_i];
                let seg_prog = segs_done + next_idx;
                let setup_frac = if total_segs > 0 { seg_prog as f32 / total_segs as f32 } else { 1.0 };
                let az_off = cur_az_bias - 10.0 + 28.0 * setup_frac;
                let cam = orbit_camera(opts.view, span, az_off, 0.0, opts.width, opts.height);
                spin_phase += 0.9;
                trail_slice.clear();
                trail_slice.extend(trail_buf.iter().copied());
                let hud = opts.hud.then_some(HudInfo {
                    setup_title: &setup_title,
                    op_title: &op_title,
                    tool_badge: &op.tool.label,
                    rpm: op.rpm,
                    feed: last_seg.feed,
                    is_rapid: last_seg.is_rapid,
                    show_tool: true,
                    wcs_pos: last_seg.wcs_b,
                    remaining_cm3: plan.stock.volume_cm3(),
                    removed_pct: plan.stock.removed_pct(),
                    progress_frac: setup_frac,
                    footer_left,
                    footer_right: &footer_right,
                    banner: "",
                    move_num: last_seg.move_no,
                    bars: &bars,
                    total_segs,
                });
                let tip_axis = last_seg.axis_part.unwrap_or(setup.spindle_axis_part);
                render_frame(
                    &FrameParams {
                        stock: &plan.stock,
                        cam,
                        part_rot: rot,
                        part_center: center,
                        spindle_axis_part: tip_axis,
                        show_tool: true,
                        tool_tip_part: last_seg.b_part,
                        tool: Some(&op.tool),
                        spin_phase,
                        is_cutting: carved_any && !last_seg.is_rapid,
                        show_toolpath: opts.toolpath,
                        trail: &trail_slice,
                        hud,
                        table_z,
                    },
                    &mut rgb,
                    &mut zbuf,
                );
                push_frame(&rgb, &mut main_sink, side_sink.as_mut(), &mut total_frames, &mut setup_frames)?;
                if f == mid_f {
                    save_keyframe(
                        kf_dir,
                        &format!("{stag}_op{:02}_mid", oi + 1),
                        opts.width,
                        opts.height,
                        &rgb,
                        &mut keyframes,
                    )?;
                }
                cur_idx = next_idx;
            }
            segs_done += op_n;
        }

        // Write a turntable check at the end of per-setup split videos and the final program spin.
        let is_last_rendered = target_idx.is_some() || si + 1 == plan.setups.len();
        let upright_back = quats_differ(setup.rot_q, first_q).then_some(first_q);
        if let Some(mut side) = side_sink {
            let side_spin = if is_last_rendered {
                spin_frames
            } else if spin_frames == 0 {
                0
            } else {
                (spin_frames / 2).max(1)
            };
            let banner = format!("{} COMPLETE - {:.1} cm3 REMAINING", setup.name, plan.stock.volume_cm3());
            let mut sc = SpinCtx {
                stock: &plan.stock,
                center,
                span,
                view: opts.view,
                w: opts.width,
                h: opts.height,
                show_hud: opts.hud,
                base_q: setup.rot_q,
                upright_q: if is_last_rendered { upright_back } else { None },
                setup_title: &setup_title,
                banner: &banner,
                footer_left,
                footer_right: &footer_right,
                bars: &bars,
                total_segs,
                rgb: &mut rgb,
                zbuf: &mut zbuf,
            };
            let half = side_spin / 2;
            let last_f = side_spin.saturating_sub(1);
            emit_spin_frames(&mut sc, side_spin, cur_az_bias + 18.0, 360.0, |f, buf| {
                side.write_frame(buf)?;
                if is_last_rendered {
                    main_sink.write_frame(buf)?;
                    total_frames += 1;
                }
                setup_frames += 1;
                if f == 0 {
                    save_keyframe(kf_dir, &format!("{stag}_done"), opts.width, opts.height, buf, &mut keyframes)?;
                } else if is_last_rendered && f == half {
                    save_keyframe(kf_dir, &format!("{stag}_done_back"), opts.width, opts.height, buf, &mut keyframes)?;
                } else if is_last_rendered && f == last_f {
                    save_keyframe(kf_dir, &format!("{stag}_done_upright"), opts.width, opts.height, buf, &mut keyframes)?;
                }
                Ok(())
            })?;
            side.finish()?;
        } else if is_last_rendered && spin_frames > 0 {
            let banner = format!("SIMULATION COMPLETE - {:.1} cm3 REMAINING", plan.stock.volume_cm3());
            let mut sc = SpinCtx {
                stock: &plan.stock,
                center,
                span,
                view: opts.view,
                w: opts.width,
                h: opts.height,
                show_hud: opts.hud,
                base_q: setup.rot_q,
                upright_q: upright_back,
                setup_title: &setup_title,
                banner: &banner,
                footer_left,
                footer_right: &footer_right,
                bars: &bars,
                total_segs,
                rgb: &mut rgb,
                zbuf: &mut zbuf,
            };
            let last_f = spin_frames.saturating_sub(1);
            emit_spin_frames(&mut sc, spin_frames, cur_az_bias + 18.0, 360.0, |f, buf| {
                main_sink.write_frame(buf)?;
                total_frames += 1;
                setup_frames += 1;
                if f == 0 {
                    save_keyframe(kf_dir, &format!("{stag}_done"), opts.width, opts.height, buf, &mut keyframes)?;
                } else if f == last_f {
                    save_keyframe(kf_dir, &format!("{stag}_done_upright"), opts.width, opts.height, buf, &mut keyframes)?;
                }
                Ok(())
            })?;
        } else if kf_dir.is_some() {
            save_keyframe(kf_dir, &format!("{stag}_done"), opts.width, opts.height, &rgb, &mut keyframes)?;
        }

        reports.push(SetupReport {
            name: setup.name.clone(),
            wcs: setup.wcs_label.clone(),
            operations: setup.ops.len(),
            segments: seg_total,
            rendered: true,
            frames: setup_frames,
            remaining_cm3: round2(plan.stock.volume_cm3()),
            removed_pct: round2(plan.stock.removed_pct()),
            file: side_file,
        });
    }

    main_sink.finish()?;
    Ok(VideoReport {
        output: opts.output.display().to_string(),
        format: resolved_fmt.as_str().to_string(),
        width: opts.width,
        height: opts.height,
        fps: opts.fps,
        view: opts.view.as_str().to_string(),
        total_frames,
        duration_s: round2(total_frames as f64 / f64::from(opts.fps)),
        cell_mm: round2(f64::from(plan.stock.cell)),
        grid: [plan.stock.nx, plan.stock.ny, plan.stock.nz],
        initial_cm3: round2(init_cm3),
        remaining_cm3: round2(plan.stock.volume_cm3()),
        removed_pct: round2(plan.stock.removed_pct()),
        part_volume_cm3: plan.part_volume_cm3.map(round2),
        part_size_mm: plan.part_size_mm.map(|s| [round2(s[0]), round2(s[1]), round2(s[2])]),
        per_setup_files,
        keyframes,
        setups: reports,
    })
}
