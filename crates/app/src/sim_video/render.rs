//! Parallel (`rayon`) DDA ray-marcher rendering voxel billet stock, spinning cutter/holder,
//! soft metallic lighting, ambient occlusion, studio reference grid, toolpath ribbon, and HUD.
//!
//! Provenance: clean-room 3D DDA traversal (Amanatides-Woo 1987), Blinn-Phong metallic shading,
//! and analytic ray-cylinder/cone/sphere intersection over our `sim_video::voxel` billet.

use rayon::prelude::*;

use super::hud::{draw_hud, draw_line_2d, HudInfo};
use super::math::{
    disp_to_part, intersect_aabb, intersect_cone_z, intersect_cylinder_z, intersect_sphere, part_to_disp, Camera, Mat3,
    Vec3,
};
use super::voxel::{tag_color, SimTool, ToolShape, VoxelStock, TAG_RAW};

#[derive(Clone, Copy, Debug)]
pub(super) struct TrailSeg {
    pub a_part: Vec3,
    pub b_part: Vec3,
    pub is_rapid: bool,
    pub is_plunge: bool,
}

pub(super) struct FrameParams<'a> {
    pub stock: &'a VoxelStock,
    pub cam: Camera,
    pub part_rot: Mat3,
    pub part_center: Vec3,
    pub spindle_axis_part: Vec3,
    pub show_tool: bool,
    pub tool_tip_part: Vec3,
    pub tool: Option<&'a SimTool>,
    pub spin_phase: f32,
    pub is_cutting: bool,
    pub show_toolpath: bool,
    pub trail: &'a [TrailSeg],
    pub hud: Option<HudInfo<'a>>,
    pub table_z: f32,
}

#[inline]
fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0).powf(1.0 / 2.15) * 255.0).clamp(0.0, 255.0) as u8
}

/// Ray-trace one complete RGB24 frame (`width * height * 3` bytes) into `rgb`.
pub(super) fn render_frame(p: &FrameParams<'_>, rgb: &mut Vec<u8>, zbuf: &mut Vec<f32>) {
    let w = p.cam.width as usize;
    let h = p.cam.height as usize;
    rgb.resize(w * h * 3, 0);
    zbuf.resize(w * h, 1e9);

    let aspect = w as f32 / h.max(1) as f32;
    let b0 = Vec3::new(p.stock.x0, p.stock.y0, p.stock.z0);
    let b1 = Vec3::new(p.stock.bx1(), p.stock.by1(), p.stock.bz1());
    let span_max = (b1.x - b0.x).max(b1.y - b0.y).max(b1.z - b0.z).max(1.0);
    let max_iter = p.stock.nx + p.stock.ny + p.stock.nz + 16;
    let light1 = Vec3::new(0.58, -0.55, 0.85).norm();
    let light2 = Vec3::new(-0.65, 0.50, 0.45).norm();
    let grid_step = (span_max / 10.0).clamp(2.0, 25.0);
    let shadow_hx = span_max * 0.56;
    let shadow_hy = span_max * 0.44;
    let fade_r = span_max * 1.48;

    let disp_tip = part_to_disp(p.tool_tip_part, p.part_rot, p.part_center);
    let disp_axis = p.part_rot.mul(p.spindle_axis_part).norm();
    let ref_vec = if disp_axis.y.abs() < 0.9 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
    let t_u = ref_vec.cross(disp_axis).norm();
    let t_v = disp_axis.cross(t_u);

    let row_size = w * 3;
    rgb.par_chunks_mut(row_size)
        .zip(zbuf.par_chunks_mut(w))
        .enumerate()
        .for_each(|(py, (rgb_row, z_row))| {
            let ny = 1.0 - 2.0 * (py as f32 + 0.5) / h as f32;
            let bg_t = py as f32 / h.max(1) as f32;
            for (px, z_slot) in z_row.iter_mut().enumerate() {
                let nx = 2.0 * (px as f32 + 0.5) / w as f32 - 1.0;
                let rd = p
                    .cam
                    .fwd
                    .add(p.cam.right.scale(nx * aspect * p.cam.tan_half_fov))
                    .add(p.cam.up.scale(ny * p.cam.tan_half_fov))
                    .norm();
                let ro = p.cam.pos;

                let mut col = Vec3::new(
                    0.085 + 0.055 * (1.0 - bg_t),
                    0.105 + 0.065 * (1.0 - bg_t),
                    0.140 + 0.085 * (1.0 - bg_t),
                );
                let mut best_depth = 1e9f32;

                // 1) Faint engineering-studio floor grid & soft shadow beneath workpiece
                if rd.z < -1e-4 {
                    let tg = (p.table_z - ro.z) / rd.z;
                    if tg > 0.0 && tg < span_max * 8.0 {
                        let gp = ro.add(rd.scale(tg));
                        let fade = (-(gp.x * gp.x + gp.y * gp.y) / (fade_r * fade_r)).exp();
                        if fade > 0.01 {
                            let gx = ((gp.x + grid_step * 100.0).rem_euclid(grid_step) - grid_step * 0.5).abs();
                            let gy = ((gp.y + grid_step * 100.0).rem_euclid(grid_step) - grid_step * 0.5).abs();
                            let line_t = grid_step * 0.482;
                            let line = if gx > line_t || gy > line_t { 1.0 } else { 0.0 };
                            let sh = if gp.x.abs() < shadow_hx && gp.y.abs() < shadow_hy { 0.50 } else { 1.0 };
                            let gc = Vec3::new(
                                0.11 * sh + line * 0.045,
                                0.13 * sh + line * 0.055,
                                0.17 * sh + line * 0.075,
                            );
                            let blend = fade * 0.85;
                            col = col.scale(1.0 - blend).add(gc.scale(blend));
                        }
                    }
                }

                // 2) Exact 3D DDA traversal of the voxel stock billet
                let ro_p = disp_to_part(ro, p.part_rot, p.part_center);
                let rd_p = p.part_rot.mul_t(rd).norm();
                if let Some((t0, _t1)) = intersect_aabb(ro_p, rd_p, b0, b1) {
                    let cell = p.stock.cell;
                    let t_enter = (t0 + 1e-4).max(0.0);
                    let pos = ro_p.add(rd_p.scale(t_enter));
                    let mut ix = ((pos.x - p.stock.x0) / cell).floor().clamp(0.0, (p.stock.nx - 1) as f32) as i32;
                    let mut iy = ((pos.y - p.stock.y0) / cell).floor().clamp(0.0, (p.stock.ny - 1) as f32) as i32;
                    let mut iz = ((pos.z - p.stock.z0) / cell).floor().clamp(0.0, (p.stock.nz - 1) as f32) as i32;

                    let step_x = if rd_p.x >= 0.0 { 1 } else { -1 };
                    let step_y = if rd_p.y >= 0.0 { 1 } else { -1 };
                    let step_z = if rd_p.z >= 0.0 { 1 } else { -1 };
                    let td_x = if rd_p.x.abs() > 1e-8 { (cell / rd_p.x).abs() } else { 1e9 };
                    let td_y = if rd_p.y.abs() > 1e-8 { (cell / rd_p.y).abs() } else { 1e9 };
                    let td_z = if rd_p.z.abs() > 1e-8 { (cell / rd_p.z).abs() } else { 1e9 };
                    let next_x = p.stock.x0 + (ix + i32::from(step_x > 0)) as f32 * cell;
                    let next_y = p.stock.y0 + (iy + i32::from(step_y > 0)) as f32 * cell;
                    let next_z = p.stock.z0 + (iz + i32::from(step_z > 0)) as f32 * cell;
                    let mut tm_x = if rd_p.x.abs() > 1e-8 { t_enter + (next_x - pos.x) / rd_p.x } else { 1e9 };
                    let mut tm_y = if rd_p.y.abs() > 1e-8 { t_enter + (next_y - pos.y) / rd_p.y } else { 1e9 };
                    let mut tm_z = if rd_p.z.abs() > 1e-8 { t_enter + (next_z - pos.z) / rd_p.z } else { 1e9 };

                    let mut hit_norm_p = if (pos.x - b0.x).abs() < 1e-2 {
                        Vec3::new(-1.0, 0.0, 0.0)
                    } else if (pos.x - b1.x).abs() < 1e-2 {
                        Vec3::new(1.0, 0.0, 0.0)
                    } else if (pos.y - b0.y).abs() < 1e-2 {
                        Vec3::new(0.0, -1.0, 0.0)
                    } else if (pos.y - b1.y).abs() < 1e-2 {
                        Vec3::new(0.0, 1.0, 0.0)
                    } else if (pos.z - b0.z).abs() < 1e-2 {
                        Vec3::new(0.0, 0.0, -1.0)
                    } else {
                        Vec3::new(0.0, 0.0, 1.0)
                    };

                    let mut t_hit = t_enter;
                    for _ in 0..max_iter {
                        let val = p.stock.data[p.stock.idx(ix as usize, iy as usize, iz as usize)];
                        if val != 0 {
                            let hp_p = ro_p.add(rd_p.scale(t_hit));
                            let sn_p = p.stock.smooth_normal(ix, iy, iz, hit_norm_p);
                            let n_p = hit_norm_p.scale(0.25).add(sn_p.scale(0.75)).norm();
                            let n_d = p.part_rot.mul(n_p).norm();
                            let ao = p.stock.estimate_ao(hp_p, n_p);
                            let alb = tag_color(val);
                            let diff1 = n_d.dot(light1).max(0.0);
                            let diff2 = n_d.dot(light2).max(0.0);
                            let view_d = rd.neg();
                            let h1 = light1.add(view_d).norm();
                            let is_raw = val == TAG_RAW;
                            let spec_p = if is_raw { 18.0 } else { 42.0 };
                            let spec_k = if is_raw { 0.16 } else { 0.42 };
                            let spec = n_d.dot(h1).max(0.0).powf(spec_p) * spec_k;
                            let hemi = 0.5 + 0.5 * n_d.z;
                            let amb = Vec3::new(0.20, 0.23, 0.28).scale(0.6 + 0.4 * hemi);
                            let shade = 0.72 * diff1 + 0.28 * diff2;
                            col = Vec3::new(
                                alb.x * (amb.x + shade) * ao + spec,
                                alb.y * (amb.y + shade) * ao + spec,
                                alb.z * (amb.z + shade) * ao + spec * 1.05,
                            );
                            if p.show_tool && p.is_cutting
                                && let Some(tool) = p.tool
                            {
                                let hp_d = ro.add(rd.scale(t_hit));
                                let dt = hp_d.sub(disp_tip).len();
                                let glow_r = (tool.diameter * 0.9).max(3.0);
                                if dt < glow_r {
                                    let g = (1.0 - dt / glow_r).powi(2) * 0.45;
                                    col = col.add(Vec3::new(g * 1.0, g * 0.65, g * 0.20));
                                }
                            }
                            best_depth = t_hit * rd.dot(p.cam.fwd);
                            break;
                        }
                        if tm_x < tm_y && tm_x < tm_z {
                            t_hit = tm_x;
                            ix += step_x;
                            tm_x += td_x;
                            hit_norm_p = Vec3::new(-step_x as f32, 0.0, 0.0);
                            if ix < 0 || ix >= p.stock.nx as i32 { break; }
                        } else if tm_y < tm_z {
                            t_hit = tm_y;
                            iy += step_y;
                            tm_y += td_y;
                            hit_norm_p = Vec3::new(0.0, -step_y as f32, 0.0);
                            if iy < 0 || iy >= p.stock.ny as i32 { break; }
                        } else {
                            t_hit = tm_z;
                            iz += step_z;
                            tm_z += td_z;
                            hit_norm_p = Vec3::new(0.0, 0.0, -step_z as f32);
                            if iz < 0 || iz >= p.stock.nz as i32 { break; }
                        }
                    }
                }

                // 3) Analytic ray-intersection of spinning cutting tool + shank + holder
                if p.show_tool
                    && let Some(tool) = p.tool
                {
                    let rel = ro.sub(disp_tip);
                    let r_o = Vec3::new(rel.dot(t_u), rel.dot(t_v), rel.dot(disp_axis));
                    let r_d = Vec3::new(rd.dot(t_u), rd.dot(t_v), rd.dot(disp_axis));
                    let tr = tool.diameter * 0.5;
                    let mut best_t = f32::INFINITY;
                    let mut best_n = Vec3::ZERO;
                    let mut part_id = 0u8; // 1=flutes, 2=shank, 3=holder

                    match tool.shape {
                        ToolShape::Flat => {
                            if let Some((t, n)) = intersect_cylinder_z(r_o, r_d, tr, 0.0, tool.flute_len) {
                                best_t = t; best_n = n; part_id = 1;
                            }
                        }
                        ToolShape::Ball => {
                            if let Some((t, n)) = intersect_sphere(r_o, r_d, Vec3::new(0.0, 0.0, tr), tr)
                                && r_o.z + r_d.z * t <= tr + 0.02
                            {
                                best_t = t; best_n = n; part_id = 1;
                            }
                            if let Some((t, n)) = intersect_cylinder_z(r_o, r_d, tr, tr, tool.flute_len.max(tr + 1.0))
                                && t < best_t
                            {
                                best_t = t; best_n = n; part_id = 1;
                            }
                        }
                        ToolShape::Bull => {
                            let c = tool.corner_r.clamp(0.0, tr);
                            if let Some((t, n)) = intersect_cylinder_z(r_o, r_d, tr, c * 0.4, tool.flute_len.max(c + 1.0)) {
                                best_t = t; best_n = n; part_id = 1;
                            }
                            if c > 0.05
                                && let Some((t, n)) = intersect_cone_z(r_o, r_d, (tr - c).max(0.05), tr, 0.0, c * 0.5)
                                && t < best_t
                            {
                                best_t = t; best_n = n; part_id = 1;
                            }
                        }
                        ToolShape::Drill => {
                            let half = (tool.tip_angle_deg.clamp(30.0, 178.0) * 0.5).to_radians();
                            let ch = tr / half.tan().max(1e-3);
                            if let Some((t, n)) = intersect_cone_z(r_o, r_d, 0.01, tr, 0.0, ch) {
                                best_t = t; best_n = n; part_id = 1;
                            }
                            if let Some((t, n)) = intersect_cylinder_z(r_o, r_d, tr, ch, tool.flute_len.max(ch + 1.0))
                                && t < best_t
                            {
                                best_t = t; best_n = n; part_id = 1;
                            }
                        }
                        ToolShape::Chamfer => {
                            let half = (tool.tip_angle_deg.clamp(10.0, 175.0) * 0.5).to_radians();
                            let ch = tr / half.tan().max(1e-3);
                            if let Some((t, n)) = intersect_cone_z(r_o, r_d, 0.01, tr, 0.0, ch) {
                                best_t = t; best_n = n; part_id = 1;
                            }
                            if let Some((t, n)) = intersect_cylinder_z(r_o, r_d, tr, ch, tool.flute_len.max(ch + 1.0))
                                && t < best_t
                            {
                                best_t = t; best_n = n; part_id = 1;
                            }
                        }
                    }
                    if tool.total_len > tool.flute_len
                        && let Some((t, n)) = intersect_cylinder_z(r_o, r_d, tr * 0.99, tool.flute_len, tool.total_len)
                        && t < best_t
                    {
                        best_t = t; best_n = n; part_id = 2;
                    }
                    let hr = tool.holder_d * 0.5;
                    if let Some((t, n)) = intersect_cone_z(r_o, r_d, tr * 1.15, hr, tool.total_len, tool.total_len + 10.0)
                        && t < best_t
                    {
                        best_t = t; best_n = n; part_id = 3;
                    }
                    if let Some((t, n)) = intersect_cylinder_z(r_o, r_d, hr, tool.total_len + 10.0, tool.total_len + 42.0)
                        && t < best_t
                    {
                        best_t = t; best_n = n; part_id = 3;
                    }

                    if part_id != 0 {
                        let tool_depth = best_t * rd.dot(p.cam.fwd);
                        if tool_depth < best_depth {
                            let hp_l = r_o.add(r_d.scale(best_t));
                            let nw = t_u.scale(best_n.x).add(t_v.scale(best_n.y)).add(disp_axis.scale(best_n.z)).norm();
                            let d1 = nw.dot(light1).max(0.0);
                            let d2 = nw.dot(light2).max(0.0);
                            let h1 = light1.add(rd.neg()).norm();
                            let sp = nw.dot(h1).max(0.0).powf(56.0);
                            let base = match part_id {
                                1 => {
                                    let ang = hp_l.y.atan2(hp_l.x) + p.spin_phase + hp_l.z * 0.28;
                                    let band = 0.80 + 0.20 * (ang * 3.0).sin();
                                    let tint = match tool.shape {
                                        ToolShape::Ball => Vec3::new(0.42, 0.65, 0.88),
                                        ToolShape::Bull => Vec3::new(0.46, 0.72, 0.82),
                                        ToolShape::Drill => Vec3::new(0.32, 0.75, 0.80),
                                        ToolShape::Chamfer => Vec3::new(0.28, 0.78, 0.50),
                                        ToolShape::Flat => Vec3::new(0.84, 0.74, 0.44),
                                    };
                                    tint.scale(band)
                                }
                                2 => Vec3::new(0.58, 0.60, 0.64),
                                _ => Vec3::new(0.11, 0.13, 0.16),
                            };
                            let sp_k = if part_id == 3 { 0.35 } else { 0.65 };
                            col = base.scale(0.28 + 0.72 * d1 + 0.25 * d2).add(Vec3::new(sp, sp, sp).scale(sp_k));
                            best_depth = tool_depth;
                        }
                    }
                }

                let off = px * 3;
                rgb_row[off] = to_u8(col.x);
                rgb_row[off + 1] = to_u8(col.y);
                rgb_row[off + 2] = to_u8(col.z);
                *z_slot = best_depth;
            }
        });

    // 4) Z-buffered live 3D toolpath ribbon trail (cyan feed, amber plunge, faint red rapid)
    if p.show_toolpath && !p.trail.is_empty() {
        let len = p.trail.len().max(1) as f32;
        for (i, seg) in p.trail.iter().enumerate() {
            let age = (i + 1) as f32 / len;
            let da = part_to_disp(seg.a_part, p.part_rot, p.part_center);
            let db = part_to_disp(seg.b_part, p.part_rot, p.part_center);
            if let (Some((sx0, sy0, d0)), Some((sx1, sy1, d1))) = (p.cam.project(da), p.cam.project(db)) {
                let (col, alpha) = if seg.is_rapid {
                    ([245, 92, 92], 0.22 + 0.34 * age)
                } else if seg.is_plunge {
                    ([250, 195, 58], 0.35 + 0.60 * age)
                } else {
                    ([75, 195, 255], 0.35 + 0.65 * age)
                };
                draw_line_2d(rgb, zbuf, p.cam.width, p.cam.height, sx0, sy0, d0, sx1, sy1, d1, col, alpha);
            }
        }
    }

    // 5) HUD chrome overlay
    if let Some(ref hud) = p.hud {
        draw_hud(rgb, p.cam.width, p.cam.height, hud);
    }
}
