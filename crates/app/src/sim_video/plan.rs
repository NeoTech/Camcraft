//! Extracts setups, rigid transforms, tools, stock solid, and chord-tessellated toolpaths
//! from a live `World` in part/document coordinates for `sim_video`.
//!
//! Provenance: reads CamCraft's own ECS world (`setup`, `setup_align`, `model`, `posting`).

use flecs_ecs::prelude::World;

use super::math::Vec3;
use super::voxel::{
    SimTool, ToolShape, VoxelStock, TAG_BALL, TAG_BORE, TAG_BULL, TAG_CHAMFER, TAG_DRILL, TAG_FACED, TAG_FLAT_FIN,
    TAG_RECESS_BALL, TAG_RECESS_FLAT, TAG_ROUGH, TAG_SIDE_FINISH,
};
use crate::cl::MoveKind;
use crate::geometry::{quat_rotate_point, transform_point, Point3};
use crate::model;
use crate::operation::OpParams;
use crate::schema::Location;
use crate::setup;
use crate::setup_align;
use crate::stock::StockShape;
use crate::tools::ToolKind;

#[derive(Clone, Debug)]
pub(super) struct VideoSeg {
    pub a_part: Vec3,
    pub b_part: Vec3,
    pub axis_part: Option<Vec3>,
    pub wcs_b: [f32; 3],
    pub is_rapid: bool,
    pub is_plunge: bool,
    pub feed: f64,
    pub move_no: usize,
}

#[derive(Clone, Debug)]
pub(super) struct VideoOp {
    pub name: String,
    pub kind_label: String,
    pub tool: SimTool,
    pub rpm: f64,
    pub segs: Vec<VideoSeg>,
}

#[derive(Clone, Debug)]
pub(super) struct VideoSetup {
    pub name: String,
    pub wcs_label: String,
    /// Unit quaternion rotating part/document coords into this setup's table orientation (Z up).
    pub rot_q: [f32; 4],
    /// Unit spindle axis (`+Z_setup`) expressed in document/part frame.
    pub spindle_axis_part: Vec3,
    pub ops: Vec<VideoOp>,
}

pub(super) struct VideoPlan {
    pub setups: Vec<VideoSetup>,
    pub stock: VoxelStock,
    pub stock_label: String,
    pub part_volume_cm3: Option<f64>,
    pub part_size_mm: Option<[f64; 3]>,
}

fn p3_to_v3(p: Point3) -> Vec3 {
    Vec3::new(p.x as f32, p.y as f32, p.z as f32)
}

fn wcs_label_for(world: &World, setup_name: Option<&str>) -> String {
    let slot = setup::wcs_slot(world, setup_name);
    if slot < 6 {
        format!("G{}", 54 + slot)
    } else {
        format!("G54.1P{}", slot - 5)
    }
}

fn wcs_origin_in_setup(world: &World, setup_name: Option<&str>) -> Point3 {
    setup::wcs_entity(world, setup_name)
        .and_then(|e| model::transform_of(world, &e))
        .map_or(Point3::ZERO, |loc| loc.pos)
}

fn to_doc_location(world: &World, setup_name: Option<&str>) -> Location {
    setup_name
        .and_then(|s| setup_align::stored(world, s))
        .unwrap_or_default()
}

fn op_tag_and_label(params: &OpParams, kind: ToolKind, setup_idx: usize, is_side_on: bool) -> (u8, &'static str) {
    match params {
        OpParams::Face { .. } => (TAG_FACED, "Face"),
        OpParams::Drill { .. } => (TAG_DRILL, "Drill"),
        OpParams::Bore { .. } | OpParams::Circular { .. } | OpParams::Thread { .. } => (TAG_BORE, "Bore"),
        OpParams::Chamfer2D { .. } | OpParams::Deburr { .. } => (TAG_CHAMFER, "Chamfer"),
        OpParams::Contour { .. } | OpParams::Contour5D { .. } => {
            if is_side_on {
                (TAG_SIDE_FINISH, "Contour")
            } else {
                (TAG_FLAT_FIN, "Contour")
            }
        }
        OpParams::Finish3D { .. } => (if setup_idx == 1 { TAG_RECESS_BALL } else { TAG_BALL }, "3D Finish"),
        OpParams::Pocket3D { stock_to_leave, .. } => match kind {
            ToolKind::BallNose => (if setup_idx == 1 { TAG_RECESS_BALL } else { TAG_BALL }, "3D Ball"),
            ToolKind::BullNose => (TAG_BULL, "3D Bull"),
            _ if *stock_to_leave > 0.05 => (if setup_idx == 1 { TAG_RECESS_FLAT } else { TAG_ROUGH }, "3D Rough"),
            _ => (if setup_idx == 1 { TAG_RECESS_FLAT } else { TAG_FLAT_FIN }, "3D Pocket"),
        },
        OpParams::Pocket { .. } | OpParams::Adaptive2D { .. } | OpParams::Offset2D { .. } => {
            if setup_idx == 1 { (TAG_RECESS_FLAT, "2D Pocket") } else { (TAG_ROUGH, "2D Pocket") }
        }
        _ => match kind {
            ToolKind::BallNose => (TAG_BALL, "Ball"),
            ToolKind::BullNose => (TAG_BULL, "Bull"),
            ToolKind::Drill => (TAG_DRILL, "Drill"),
            ToolKind::Taper => (TAG_CHAMFER, "Chamfer"),
            _ => (TAG_FLAT_FIN, "Mill"),
        },
    }
}

fn tool_shape_and_label(tool: &crate::tools::Tool, tnum: u32, tag: u8, is_side_on: bool, z_span: f32) -> SimTool {
    let shape = match tool.kind {
        ToolKind::BallNose => ToolShape::Ball,
        ToolKind::BullNose if tool.corner_r > 0.0 => ToolShape::Bull,
        ToolKind::Drill => ToolShape::Drill,
        ToolKind::Taper => ToolShape::Chamfer,
        _ => ToolShape::Flat,
    };
    let d = tool.diameter.max(0.1) as f32;
    let flute = if tool.flute_len > 0.1 { tool.flute_len as f32 } else { (d * 2.5).max(10.0) };
    let shank = if tool.shank_len > 0.1 { tool.shank_len as f32 } else { (d * 2.0).max(15.0) };
    let total = flute + shank;
    let holder_d = if tool.holder_d > d as f64 { tool.holder_d as f32 } else { (d * 2.6).max(24.0) };
    let tip_angle = if tool.tip_angle > 1.0 {
        tool.tip_angle as f32
    } else if shape == ToolShape::Drill {
        118.0
    } else {
        90.0
    };
    let kind_txt = match shape {
        ToolShape::Flat => "Flat EM",
        ToolShape::Ball => "Ball-Nose",
        ToolShape::Bull => "Bull-Nose",
        ToolShape::Drill => "Drill",
        ToolShape::Chamfer => "Chamfer",
    };
    let label = format!("T{:02} {kind_txt} D{d:.1}", tnum);
    // Keep side-on contour cutters strictly inside their axial window so a relieved-shank end
    // mill working on a lower ear/flange does not re-paint an upper wall.
    let cut_height = match shape {
        ToolShape::Chamfer => (d * 0.5) / ((tip_angle * 0.5).to_radians().tan().max(1e-3)) + 0.2,
        ToolShape::Flat if is_side_on && tag == TAG_SIDE_FINISH => (z_span + 0.5).clamp(2.0, flute),
        ToolShape::Flat | ToolShape::Drill => {
            if is_side_on { flute } else { total }
        }
        ToolShape::Ball | ToolShape::Bull => flute,
    };
    SimTool {
        shape,
        diameter: d,
        corner_r: tool.corner_r as f32,
        tip_angle_deg: tip_angle,
        flute_len: flute,
        total_len: total,
        holder_d,
        label,
        tag,
        cut_height,
    }
}

/// Signed triangle mesh volume in cm^3 for any watertight part mesh in the document frame.
fn part_mesh_stats(world: &World) -> (Option<f64>, Option<[f64; 3]>) {
    let skip = crate::stock_source::stock_only_model(world);
    let mut vol6 = 0.0f64;
    let mut any = false;
    let (mut lo, mut hi) = (
        Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
        Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
    );
    for (_, e) in model::model_entities(world) {
        if !crate::stock_source::judged(world, &e, skip) {
            continue;
        }
        let Some(tris) = model::facets_of(world, &e) else { continue };
        let l = model::transform_of(world, &e).unwrap_or_default();
        for t in tris.iter() {
            any = true;
            let a = transform_point(t.a, l.pos, l.rot);
            let b = transform_point(t.b, l.pos, l.rot);
            let c = transform_point(t.c, l.pos, l.rot);
            vol6 += a.dot(b.cross(c));
            for p in [a, b, c] {
                lo = Point3::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
                hi = Point3::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
            }
        }
    }
    if !any {
        return (None, None);
    }
    let vol_cm3 = (vol6.abs() / 6.0) / 1000.0;
    let size = [hi.x - lo.x, hi.y - lo.y, hi.z - lo.z];
    let v = (vol_cm3.is_finite() && vol_cm3 > 1e-4).then_some(vol_cm3);
    let s = size.iter().all(|x| x.is_finite() && *x >= 0.0).then_some(size);
    (v, s)
}

/// Build the initial voxel stock billet from the first setup's stock in document frame.
fn build_initial_stock(world: &World, first_setup: Option<&str>, cell_override: Option<f64>) -> Result<(VoxelStock, String), String> {
    let stock_name = setup::stock_entity_name(world, first_setup);
    let e = model::find_by_name(world, &stock_name)
        .ok_or_else(|| format!("no stock '{stock_name}' defined in document"))?;
    let aabb = model::stock_world_aabb_for(world, first_setup)
        .ok_or_else(|| format!("stock '{stock_name}' has no finite bounding box"))?;
    let align = to_doc_location(world, first_setup);
    let inv = setup_align::inverse(align);
    let mut lo = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
    let mut hi = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for x in [aabb.min.x, aabb.max.x] {
        for y in [aabb.min.y, aabb.max.y] {
            for z in [aabb.min.z, aabb.max.z] {
                let p = transform_point(Point3::new(x, y, z), inv.pos, inv.rot);
                lo = Point3::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
                hi = Point3::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
            }
        }
    }
    let (sx, sy, sz) = (hi.x - lo.x, hi.y - lo.y, hi.z - lo.z);
    if !(sx.is_finite() && sy.is_finite() && sz.is_finite() && sx > 0.0 && sy > 0.0 && sz > 0.0) {
        return Err("stock has degenerate dimensions".into());
    }
    // Auto-select voxel pitch (~14M voxels target) unless the caller specifies `cell_mm`.
    let target_voxels = 14_000_000.0f64;
    let cell = match cell_override {
        Some(c) if c.is_finite() && c > 0.0 => c,
        Some(c) => return Err(format!("cell size must be positive and finite, got {c}")),
        None => ((sx * sy * sz) / target_voxels).cbrt().clamp(0.20, 4.0),
    };
    let est = (sx / cell).ceil() * (sy / cell).ceil() * (sz / cell).ceil();
    if !est.is_finite() || est > 120_000_000.0 {
        return Err(format!(
            "voxel cell {cell:.3} mm would allocate {:.1}M voxels (max 120M) - choose a larger cell size",
            est / 1_000_000.0
        ));
    }
    let mut stock = VoxelStock::from_box(p3_to_v3(lo), p3_to_v3(hi), cell as f32);
    if let Some(shape) = model::stock_def_of(world, &e) {
        match shape {
            StockShape::Box { .. } => {}
            StockShape::Cylinder { diameter, .. } => {
                let c = stock.center();
                stock.mask_cylinder_z(c.x, c.y, (diameter * 0.5) as f32);
            }
            StockShape::FromModel { .. } => {
                if let Some(tris) = model::facets_of(world, &e) {
                    let loc = model::transform_of(world, &e).unwrap_or_default();
                    let world_tris: Vec<_> = tris
                        .iter()
                        .map(|t| {
                            let map = |pt: Point3| {
                                let wpt = transform_point(pt, loc.pos, loc.rot);
                                transform_point(wpt, inv.pos, inv.rot)
                            };
                            crate::geometry::Triangle::new(map(t.a), map(t.b), map(t.c))
                        })
                        .collect();
                    stock.mask_mesh(&world_tris);
                }
            }
        }
    }
    let label = format!("Stock {sx:.0}x{sy:.0}x{sz:.0}mm");
    Ok((stock, label))
}

/// Gather the full generated multi-setup program in part/document world coordinates.
pub(super) fn gather_plan(world: &World, cell_mm: Option<f64>) -> Result<VideoPlan, String> {
    let ordered = crate::posting::program_ops(world)?;
    if ordered.is_empty() {
        return Err("no operations in program to simulate".into());
    }
    let first_setup = model::operation_of(world, &ordered[0].1)
        .and_then(|op| setup::effective(world, op.setup.as_deref()));
    let (stock, stock_label) = build_initial_stock(world, first_setup.as_deref(), cell_mm)?;
    let chord_tol = (f64::from(stock.cell) * 0.5).clamp(0.05, 0.4);

    let mut setups: Vec<VideoSetup> = Vec::new();
    for (op_name, op_ent) in ordered {
        let op = model::operation_of(world, &op_ent)
            .ok_or_else(|| format!("{op_name}: lost operation component"))?;
        let tp = model::toolpath_of(world, &op_ent)
            .ok_or_else(|| format!("{op_name}: has no generated toolpath (run generate first)"))?;
        let moves = &tp.path.moves;
        if moves.is_empty() {
            continue;
        }
        let eff_setup = setup::effective(world, op.setup.as_deref());
        let s_name = eff_setup.clone().unwrap_or_else(|| "Setup1".to_string());
        let need_new = setups.last().is_none_or(|cur| cur.name != s_name);
        if need_new {
            let align = to_doc_location(world, eff_setup.as_deref());
            let inv = setup_align::inverse(align);
            let axis = quat_rotate_point(inv.rot, Point3::new(0.0, 0.0, 1.0));
            let q = align.rot;
            setups.push(VideoSetup {
                name: s_name.clone(),
                wcs_label: wcs_label_for(world, eff_setup.as_deref()),
                rot_q: [q[0] as f32, q[1] as f32, q[2] as f32, q[3] as f32],
                spindle_axis_part: p3_to_v3(axis).norm(),
                ops: Vec::new(),
            });
        }
        let setup_idx = setups.len() - 1;
        let is_side_on = setups[setup_idx].spindle_axis_part.z.abs() < 0.5;
        let align = to_doc_location(world, eff_setup.as_deref());
        let inv = setup_align::inverse(align);
        let wcs_org = wcs_origin_in_setup(world, eff_setup.as_deref());

        let _tool_ent = model::find_by_name(world, &op.tool)
            .ok_or_else(|| format!("{op_name}: missing tool '{}'", op.tool))?;
        let tool_def = model::tool_of(world, &_tool_ent)
            .ok_or_else(|| format!("{op_name}: tool '{}' has no definition", op.tool))?;
        let tnum = crate::tool_numbers::number_of(world, &op.tool).unwrap_or(1);

        let (tag, kind_label) = op_tag_and_label(&op.params, tool_def.kind, setup_idx, is_side_on);
        let (mut zmin, mut zmax) = (f64::INFINITY, f64::NEG_INFINITY);
        for m in moves {
            if m.kind != MoveKind::Rapid {
                zmin = zmin.min(m.from.z).min(m.to.z);
                zmax = zmax.max(m.from.z).max(m.to.z);
            }
        }
        let z_span = if zmin.is_finite() && zmax.is_finite() { (zmax - zmin) as f32 } else { 12.0 };
        let sim_tool = tool_shape_and_label(&tool_def, tnum, tag, is_side_on, z_span);

        let to_part = |pt: Point3| p3_to_v3(transform_point(pt, inv.pos, inv.rot));
        let to_wcs = |pt: Point3| [
            (pt.x - wcs_org.x) as f32,
            (pt.y - wcs_org.y) as f32,
            (pt.z - wcs_org.z) as f32,
        ];

        let mut segs = Vec::with_capacity(moves.len());
        for (mi, m) in moves.iter().enumerate() {
            let is_rapid = m.kind == MoveKind::Rapid;
            let is_plunge = m.kind == MoveKind::Plunge;
            let feed = m.feed.unwrap_or(tool_def.feed);
            let axis_part = m.axis.map(|ax| {
                p3_to_v3(quat_rotate_point(inv.rot, Point3::new(ax[0], ax[1], ax[2]))).norm()
            });
            let push_seg = |segs: &mut Vec<VideoSeg>, a: Point3, b: Point3, rapid: bool| {
                segs.push(VideoSeg {
                    a_part: to_part(a),
                    b_part: to_part(b),
                    axis_part,
                    wcs_b: to_wcs(b),
                    is_rapid: rapid,
                    is_plunge,
                    feed,
                    move_no: mi + 1,
                });
            };
            if m.arc.is_none() || is_rapid {
                push_seg(&mut segs, m.from, m.to, is_rapid);
            } else {
                let chords = m.segments3(chord_tol);
                if chords.is_empty() {
                    push_seg(&mut segs, m.from, m.to, false);
                } else {
                    for (a, b) in chords {
                        push_seg(&mut segs, a, b, false);
                    }
                }
            }
        }
        setups[setup_idx].ops.push(VideoOp {
            name: op_name,
            kind_label: kind_label.to_string(),
            tool: sim_tool,
            rpm: tool_def.rpm,
            segs,
        });
    }

    if setups.iter().all(|s| s.ops.is_empty()) {
        return Err("no generated toolpath moves to simulate".into());
    }
    let (part_volume_cm3, part_size_mm) = part_mesh_stats(world);
    Ok(VideoPlan { setups, stock, stock_label, part_volume_cm3, part_size_mm })
}
