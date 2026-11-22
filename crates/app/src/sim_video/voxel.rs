//! Persistent 3D voxel billet carrying machined stock across arbitrary multi-setup rotations.
//!
//! Provenance: clean-room uniform-grid occupancy model with per-surface operation color tags,
//! axis-aligned column height bounds for fast upright/flipped carving, and 3x3x3 occupancy
//! gradient shading normals.

use super::math::{Mat3, Vec3};
use crate::geometry::Triangle;

/// Per-voxel surface tag: `0` = removed air, `1..=12` = machined face tags, `TAG_RAW` = uncut stock.
pub(super) const TAG_RAW: u8 = 10;

pub(super) const TAG_FACED: u8 = 1;
pub(super) const TAG_ROUGH: u8 = 2;
pub(super) const TAG_FLAT_FIN: u8 = 3;
pub(super) const TAG_BALL: u8 = 4;
pub(super) const TAG_DRILL: u8 = 5;
pub(super) const TAG_RECESS_BALL: u8 = 6;
pub(super) const TAG_BORE: u8 = 7;
pub(super) const TAG_CHAMFER: u8 = 8;
pub(super) const TAG_RECESS_FLAT: u8 = 9;
pub(super) const TAG_SIDE_FINISH: u8 = 11;
pub(super) const TAG_BULL: u8 = 12;

/// Normalized linear RGB albedo for a surface tag.
#[inline]
pub(super) fn tag_color(tag: u8) -> Vec3 {
    match tag {
        TAG_RAW => Vec3::new(0.42, 0.46, 0.52),          // raw billet slate
        TAG_FACED => Vec3::new(0.82, 0.85, 0.89),        // faced top/bottom mirror finish
        TAG_ROUGH => Vec3::new(0.66, 0.72, 0.78),        // rough pocket steel
        TAG_FLAT_FIN => Vec3::new(0.76, 0.80, 0.86),     // vertical wall/contour finish
        TAG_BALL => Vec3::new(0.46, 0.70, 0.92),         // ball-nose surfacing
        TAG_DRILL => Vec3::new(0.36, 0.82, 0.88),        // drilled holes
        TAG_RECESS_BALL => Vec3::new(0.88, 0.76, 0.44),  // secondary ball-nose / recess fillet
        TAG_RECESS_FLAT => Vec3::new(0.77, 0.74, 0.67),  // secondary flat recess
        TAG_BORE => Vec3::new(0.88, 0.83, 0.65),         // helical / interpolated bore
        TAG_CHAMFER => Vec3::new(0.20, 0.88, 0.44),      // emerald chamfer bevel
        TAG_SIDE_FINISH => Vec3::new(0.86, 0.90, 0.96),  // side-on smooth contour finish
        TAG_BULL => Vec3::new(0.56, 0.77, 0.88),         // bull-nose corner finish
        _ => Vec3::new(0.70, 0.72, 0.76),
    }
}

/// Matching HUD progress-bar RGB byte color for a surface tag.
#[inline]
pub(super) fn bar_color(tag: u8) -> [u8; 3] {
    match tag {
        TAG_FACED => [200, 215, 235],
        TAG_ROUGH => [100, 155, 220],
        TAG_FLAT_FIN => [140, 185, 240],
        TAG_BALL => [85, 175, 245],
        TAG_DRILL => [75, 210, 220],
        TAG_RECESS_BALL => [235, 190, 85],
        TAG_RECESS_FLAT => [210, 170, 100],
        TAG_BORE => [225, 205, 120],
        TAG_CHAMFER => [50, 225, 115],
        TAG_SIDE_FINISH => [170, 215, 255],
        TAG_BULL => [110, 190, 225],
        _ => [145, 165, 190],
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ToolShape {
    Flat,
    Ball,
    Bull,
    Drill,
    Chamfer,
}

#[derive(Clone, Debug)]
pub(super) struct SimTool {
    pub shape: ToolShape,
    pub diameter: f32,
    pub corner_r: f32,
    pub tip_angle_deg: f32,
    pub flute_len: f32,
    pub total_len: f32,
    pub holder_d: f32,
    pub label: String,
    pub tag: u8,
    /// Effective flute cutting length during voxel removal (`flute_len` or `total_len` along spindle axis).
    pub cut_height: f32,
}

impl SimTool {
    /// Local tool radius `r(dz)` at axial height `dz >= 0` above tip (`None` if outside cutting envelope).
    #[inline]
    pub fn radius_at(&self, dz: f32) -> Option<f32> {
        if dz < 0.0 || dz > self.cut_height {
            return None;
        }
        let r = self.diameter * 0.5;
        let rad = match self.shape {
            ToolShape::Flat => r,
            ToolShape::Ball => {
                if dz < r {
                    (r * r - (r - dz) * (r - dz)).max(0.0).sqrt()
                } else {
                    r
                }
            }
            ToolShape::Bull => {
                let c = self.corner_r.clamp(0.0, r);
                if c > 0.0 && dz < c {
                    (r - c) + (c * c - (c - dz) * (c - dz)).max(0.0).sqrt()
                } else {
                    r
                }
            }
            ToolShape::Drill => {
                let half_rad = (self.tip_angle_deg.clamp(30.0, 178.0) * 0.5).to_radians();
                let cone_h = r / half_rad.tan().max(1e-3);
                if dz < cone_h { r * (dz / cone_h) } else { r }
            }
            ToolShape::Chamfer => {
                let half_rad = (self.tip_angle_deg.clamp(10.0, 175.0) * 0.5).to_radians();
                let tan_h = half_rad.tan().max(1e-3);
                let cone_h = r / tan_h;
                if dz > cone_h + 0.25 {
                    return None;
                }
                (dz * tan_h).min(r)
            }
        };
        Some(rad)
    }

    /// Axial tip clearance height `min_dz(rho)` for radial distance `rho <= r` (for fast Z-column cuts).
    #[inline]
    fn min_dz_for_r2(&self, rho2: f32) -> f32 {
        let r = self.diameter * 0.5;
        let r2 = r * r;
        match self.shape {
            ToolShape::Flat => 0.0,
            ToolShape::Ball => r - (r2 - rho2).max(0.0).sqrt(),
            ToolShape::Bull => {
                let c = self.corner_r.clamp(0.0, r);
                let flat = (r - c).max(0.0);
                let rho = rho2.sqrt();
                if rho <= flat || c <= 0.0 {
                    0.0
                } else {
                    let dr = (rho - flat).min(c);
                    c - (c * c - dr * dr).max(0.0).sqrt()
                }
            }
            ToolShape::Drill => {
                let half_rad = (self.tip_angle_deg.clamp(30.0, 178.0) * 0.5).to_radians();
                let cone_h = r / half_rad.tan().max(1e-3);
                cone_h * (rho2.sqrt() / r.max(1e-5))
            }
            ToolShape::Chamfer => {
                let half_rad = (self.tip_angle_deg.clamp(10.0, 175.0) * 0.5).to_radians();
                rho2.sqrt() / half_rad.tan().max(1e-3)
            }
        }
    }
}

/// Dense single-byte voxel grid held in document/part world coordinates.
pub(super) struct VoxelStock {
    pub cell: f32,
    pub x0: f32,
    pub y0: f32,
    pub z0: f32,
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub data: Vec<u8>,
    pub initial_count: u64,
    pub solid_count: u64,
    col_max_z: Vec<i32>,
    col_min_z: Vec<i32>,
}

impl VoxelStock {
    /// Allocate a rectangular raw billet grid covering `[lo, hi]`.
    pub fn from_box(lo: Vec3, hi: Vec3, cell: f32) -> Self {
        let nx = ((hi.x - lo.x) / cell).ceil().max(1.0) as usize;
        let ny = ((hi.y - lo.y) / cell).ceil().max(1.0) as usize;
        let nz = ((hi.z - lo.z) / cell).ceil().max(1.0) as usize;
        let total = nx * ny * nz;
        let mut stock = Self {
            cell,
            x0: lo.x - (nx as f32 * cell - (hi.x - lo.x)) * 0.5,
            y0: lo.y - (ny as f32 * cell - (hi.y - lo.y)) * 0.5,
            z0: lo.z - (nz as f32 * cell - (hi.z - lo.z)) * 0.5,
            nx,
            ny,
            nz,
            data: vec![TAG_RAW; total],
            initial_count: total as u64,
            solid_count: total as u64,
            col_max_z: vec![nz as i32 - 1; nx * ny],
            col_min_z: vec![0; nx * ny],
        };
        stock.recompute_z_bounds();
        stock
    }

    /// Carve away voxels outside an upright cylinder (`cx, cy, radius`).
    pub fn mask_cylinder_z(&mut self, cx: f32, cy: f32, radius: f32) {
        let r2 = radius * radius;
        let mut solid = 0u64;
        for iy in 0..self.ny {
            let dy = (self.y0 + (iy as f32 + 0.5) * self.cell) - cy;
            for ix in 0..self.nx {
                let dx = (self.x0 + (ix as f32 + 0.5) * self.cell) - cx;
                let inside = dx * dx + dy * dy <= r2;
                for iz in 0..self.nz {
                    let k = self.idx(ix, iy, iz);
                    if inside && self.data[k] != 0 {
                        solid += 1;
                    } else {
                        self.data[k] = 0;
                    }
                }
            }
        }
        self.initial_count = solid.max(1);
        self.solid_count = solid;
        self.recompute_z_bounds();
    }

    /// Mask an arbitrary closed stock mesh via Z-ray parity crossings.
    pub fn mask_mesh(&mut self, tris: &[Triangle]) {
        if tris.is_empty() {
            return;
        }
        let mut z_hits: Vec<f32> = Vec::with_capacity(8);
        let mut solid = 0u64;
        for iy in 0..self.ny {
            let py = f64::from(self.y0 + (iy as f32 + 0.5) * self.cell);
            for ix in 0..self.nx {
                let px = f64::from(self.x0 + (ix as f32 + 0.5) * self.cell);
                z_hits.clear();
                for t in tris {
                    if let Some(z) = ray_z_tri(px, py, t) {
                        z_hits.push(z as f32);
                    }
                }
                z_hits.sort_by(|a, b| a.total_cmp(b));
                for iz in 0..self.nz {
                    let pz = self.z0 + (iz as f32 + 0.5) * self.cell;
                    let below = z_hits.partition_point(|&hz| hz < pz);
                    let k = self.idx(ix, iy, iz);
                    if below % 2 == 1 {
                        self.data[k] = TAG_RAW;
                        solid += 1;
                    } else {
                        self.data[k] = 0;
                    }
                }
            }
        }
        if solid > 0 {
            self.initial_count = solid;
            self.solid_count = solid;
        }
        self.recompute_z_bounds();
    }

    #[inline]
    pub fn idx(&self, ix: usize, iy: usize, iz: usize) -> usize {
        (iz * self.ny + iy) * self.nx + ix
    }

    #[inline]
    pub fn at(&self, ix: i32, iy: i32, iz: i32) -> u8 {
        if ix < 0 || iy < 0 || iz < 0 || ix >= self.nx as i32 || iy >= self.ny as i32 || iz >= self.nz as i32 {
            return 0;
        }
        self.data[self.idx(ix as usize, iy as usize, iz as usize)]
    }

    #[inline]
    pub fn bx1(&self) -> f32 {
        self.x0 + self.nx as f32 * self.cell
    }
    #[inline]
    pub fn by1(&self) -> f32 {
        self.y0 + self.ny as f32 * self.cell
    }
    #[inline]
    pub fn bz1(&self) -> f32 {
        self.z0 + self.nz as f32 * self.cell
    }

    #[inline]
    pub fn center(&self) -> Vec3 {
        Vec3::new((self.x0 + self.bx1()) * 0.5, (self.y0 + self.by1()) * 0.5, (self.z0 + self.bz1()) * 0.5)
    }

    #[inline]
    pub fn span_vec(&self) -> Vec3 {
        Vec3::new(self.bx1() - self.x0, self.by1() - self.y0, self.bz1() - self.z0)
    }

    #[inline]
    pub fn volume_cm3(&self) -> f64 {
        let c = f64::from(self.cell);
        (self.solid_count as f64) * (c * c * c) / 1000.0
    }

    #[inline]
    pub fn initial_cm3(&self) -> f64 {
        let c = f64::from(self.cell);
        (self.initial_count as f64) * (c * c * c) / 1000.0
    }

    #[inline]
    pub fn removed_pct(&self) -> f64 {
        if self.initial_count == 0 {
            return 0.0;
        }
        100.0 * (1.0 - self.solid_count as f64 / self.initial_count as f64)
    }

    pub fn recompute_z_bounds(&mut self) {
        for iy in 0..self.ny {
            for ix in 0..self.nx {
                let mut hi = -1i32;
                let mut lo = self.nz as i32;
                for iz in 0..self.nz {
                    if self.data[self.idx(ix, iy, iz)] != 0 {
                        if lo == self.nz as i32 {
                            lo = iz as i32;
                        }
                        hi = iz as i32;
                    }
                }
                let ci = iy * self.nx + ix;
                self.col_max_z[ci] = hi;
                self.col_min_z[ci] = lo;
            }
        }
    }

    #[inline]
    fn remove_voxel(&mut self, ix: usize, iy: usize, iz: usize, tag: u8) {
        let k = self.idx(ix, iy, iz);
        if self.data[k] == 0 {
            return;
        }
        self.data[k] = 0;
        self.solid_count -= 1;
        if ix + 1 < self.nx {
            let j = self.idx(ix + 1, iy, iz);
            if self.data[j] != 0 { self.data[j] = tag; }
        }
        if ix > 0 {
            let j = self.idx(ix - 1, iy, iz);
            if self.data[j] != 0 { self.data[j] = tag; }
        }
        if iy + 1 < self.ny {
            let j = self.idx(ix, iy + 1, iz);
            if self.data[j] != 0 { self.data[j] = tag; }
        }
        if iy > 0 {
            let j = self.idx(ix, iy - 1, iz);
            if self.data[j] != 0 { self.data[j] = tag; }
        }
        if iz + 1 < self.nz {
            let j = self.idx(ix, iy, iz + 1);
            if self.data[j] != 0 { self.data[j] = tag; }
        }
        if iz > 0 {
            let j = self.idx(ix, iy, iz - 1);
            if self.data[j] != 0 { self.data[j] = tag; }
        }
    }

    /// Sweep `tool` along a linear segment `a -> b` in part coords with unit spindle axis `axis`.
    pub fn sweep_segment(&mut self, a: Vec3, b: Vec3, axis: Vec3, tool: &SimTool) -> u32 {
        let d = b.sub(a);
        let len = d.len();
        let step = (tool.diameter * 0.14).min(self.cell * 0.9).max(0.05);
        let n = (len / step).ceil().max(1.0) as usize;
        let fast_pos_z = axis.z > 0.9999;
        let fast_neg_z = axis.z < -0.9999;
        let mut carved = 0u32;
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let tip = a.add(d.scale(t));
            carved += if fast_pos_z {
                self.carve_pos_z(tip, tool)
            } else if fast_neg_z {
                self.carve_neg_z(tip, tool)
            } else {
                self.carve_oriented(tip, axis, tool)
            };
        }
        carved
    }

    fn carve_pos_z(&mut self, tip: Vec3, tool: &SimTool) -> u32 {
        let r = tool.diameter * 0.5;
        let r2 = r * r;
        let c = self.cell;
        let ix0 = ((tip.x - r - self.x0) / c).floor().max(0.0) as usize;
        let ix1 = ((tip.x + r - self.x0) / c).ceil().min((self.nx - 1) as f32) as usize;
        let iy0 = ((tip.y - r - self.y0) / c).floor().max(0.0) as usize;
        let iy1 = ((tip.y + r - self.y0) / c).ceil().min((self.ny - 1) as f32) as usize;
        let iz0 = ((tip.z - self.z0) / c).floor().max(0.0) as i32;
        let iz1_base = ((tip.z + tool.cut_height - self.z0) / c).ceil().min((self.nz - 1) as f32) as i32;
        if ix0 > ix1 || iy0 > iy1 || iz0 > iz1_base {
            return 0;
        }
        let mut removed = 0u32;
        for iy in iy0..=iy1 {
            let dy = (self.y0 + (iy as f32 + 0.5) * c) - tip.y;
            let dy2 = dy * dy;
            if dy2 > r2 {
                continue;
            }
            let row = iy * self.nx;
            for ix in ix0..=ix1 {
                let dx = (self.x0 + (ix as f32 + 0.5) * c) - tip.x;
                let dxy2 = dx * dx + dy2;
                if dxy2 > r2 {
                    continue;
                }
                let hi = self.col_max_z[row + ix];
                if hi < iz0 {
                    continue;
                }
                let iz_top = hi.min(iz1_base);
                let min_dz = tool.min_dz_for_r2(dxy2);
                let iz_start = iz0.max((((tip.z + min_dz) - self.z0) / c - 0.5).ceil() as i32);
                if iz_start > iz_top {
                    continue;
                }
                for iz in (iz_start..=iz_top).rev() {
                    if self.data[self.idx(ix, iy, iz as usize)] != 0 {
                        self.remove_voxel(ix, iy, iz as usize, tool.tag);
                        removed += 1;
                    }
                }
                if iz_top == hi {
                    let mut nh = iz_start - 1;
                    while nh >= 0 && self.data[self.idx(ix, iy, nh as usize)] == 0 {
                        nh -= 1;
                    }
                    self.col_max_z[row + ix] = nh;
                }
            }
        }
        removed
    }

    fn carve_neg_z(&mut self, tip: Vec3, tool: &SimTool) -> u32 {
        let r = tool.diameter * 0.5;
        let r2 = r * r;
        let c = self.cell;
        let ix0 = ((tip.x - r - self.x0) / c).floor().max(0.0) as usize;
        let ix1 = ((tip.x + r - self.x0) / c).ceil().min((self.nx - 1) as f32) as usize;
        let iy0 = ((tip.y - r - self.y0) / c).floor().max(0.0) as usize;
        let iy1 = ((tip.y + r - self.y0) / c).ceil().min((self.ny - 1) as f32) as usize;
        let iz0_base = ((tip.z - tool.cut_height - self.z0) / c).floor().max(0.0) as i32;
        let iz1 = ((tip.z - self.z0) / c).ceil().min((self.nz - 1) as f32) as i32;
        if ix0 > ix1 || iy0 > iy1 || iz0_base > iz1 {
            return 0;
        }
        let mut removed = 0u32;
        for iy in iy0..=iy1 {
            let dy = (self.y0 + (iy as f32 + 0.5) * c) - tip.y;
            let dy2 = dy * dy;
            if dy2 > r2 {
                continue;
            }
            let row = iy * self.nx;
            for ix in ix0..=ix1 {
                let dx = (self.x0 + (ix as f32 + 0.5) * c) - tip.x;
                let dxy2 = dx * dx + dy2;
                if dxy2 > r2 {
                    continue;
                }
                let lo = self.col_min_z[row + ix];
                if lo > iz1 {
                    continue;
                }
                let iz_bot = lo.max(iz0_base);
                let min_dz = tool.min_dz_for_r2(dxy2);
                let iz_stop = iz1.min((((tip.z - min_dz) - self.z0) / c - 0.5).floor() as i32);
                if iz_stop < iz_bot {
                    continue;
                }
                for iz in iz_bot..=iz_stop {
                    if self.data[self.idx(ix, iy, iz as usize)] != 0 {
                        self.remove_voxel(ix, iy, iz as usize, tool.tag);
                        removed += 1;
                    }
                }
                if iz_bot == lo {
                    let mut nl = iz_stop + 1;
                    while nl < self.nz as i32 && self.data[self.idx(ix, iy, nl as usize)] == 0 {
                        nl += 1;
                    }
                    self.col_min_z[row + ix] = nl;
                }
            }
        }
        removed
    }

    /// Arbitrary oriented spindle axis (handles ±X, ±Y side-on quarter turns and any general 3D axis).
    fn carve_oriented(&mut self, tip: Vec3, axis: Vec3, tool: &SimTool) -> u32 {
        let r = tool.diameter * 0.5;
        let h = tool.cut_height;
        let top = tip.add(axis.scale(h));
        let ext_x = r * (1.0 - axis.x * axis.x).max(0.0).sqrt();
        let ext_y = r * (1.0 - axis.y * axis.y).max(0.0).sqrt();
        let ext_z = r * (1.0 - axis.z * axis.z).max(0.0).sqrt();
        let c = self.cell;
        let ix0 = ((tip.x.min(top.x) - ext_x - self.x0) / c).floor().max(0.0) as usize;
        let ix1 = ((tip.x.max(top.x) + ext_x - self.x0) / c).ceil().min((self.nx - 1) as f32) as usize;
        let iy0 = ((tip.y.min(top.y) - ext_y - self.y0) / c).floor().max(0.0) as usize;
        let iy1 = ((tip.y.max(top.y) + ext_y - self.y0) / c).ceil().min((self.ny - 1) as f32) as usize;
        let iz0 = ((tip.z.min(top.z) - ext_z - self.z0) / c).floor().max(0.0) as usize;
        let iz1 = ((tip.z.max(top.z) + ext_z - self.z0) / c).ceil().min((self.nz - 1) as f32) as usize;
        if ix0 > ix1 || iy0 > iy1 || iz0 > iz1 {
            return 0;
        }
        let mut removed = 0u32;
        for iz in iz0..=iz1 {
            let vz = self.z0 + (iz as f32 + 0.5) * c;
            let pz = vz - tip.z;
            for iy in iy0..=iy1 {
                let vy = self.y0 + (iy as f32 + 0.5) * c;
                let py = vy - tip.y;
                for ix in ix0..=ix1 {
                    let vx = self.x0 + (ix as f32 + 0.5) * c;
                    let px = vx - tip.x;
                    let dz = px * axis.x + py * axis.y + pz * axis.z;
                    let Some(rad) = tool.radius_at(dz) else { continue };
                    let rx = px - axis.x * dz;
                    let ry = py - axis.y * dz;
                    let rz = pz - axis.z * dz;
                    if rx * rx + ry * ry + rz * rz <= rad * rad && self.data[self.idx(ix, iy, iz)] != 0 {
                        self.remove_voxel(ix, iy, iz, tool.tag);
                        removed += 1;
                    }
                }
            }
        }
        removed
    }

    /// Smoothed 3x3x3 occupancy gradient for soft metallic shading.
    #[inline]
    pub fn smooth_normal(&self, ix: i32, iy: i32, iz: i32, fallback: Vec3) -> Vec3 {
        let (mut gx, mut gy, mut gz) = (0.0f32, 0.0f32, 0.0f32);
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    if self.at(ix + dx, iy + dy, iz + dz) != 0 {
                        let w = 1.0 / (dx.abs() + dy.abs() + dz.abs()) as f32;
                        gx -= dx as f32 * w;
                        gy -= dy as f32 * w;
                        gz -= dz as f32 * w;
                    }
                }
            }
        }
        let l2 = gx * gx + gy * gy + gz * gz;
        if l2 < 1e-4 {
            fallback
        } else {
            let inv = 1.0 / l2.sqrt();
            Vec3::new(gx * inv, gy * inv, gz * inv)
        }
    }

    /// Cheap local ambient occlusion via sparse voxel neighborhood probes along surface normal.
    #[inline]
    pub fn estimate_ao(&self, hit_p: Vec3, n: Vec3) -> f32 {
        let up = if n.z.abs() < 0.9 { Vec3::new(0.0, 0.0, 1.0) } else { Vec3::new(1.0, 0.0, 0.0) };
        let u = n.cross(up).norm();
        let v = n.cross(u);
        let dirs = [
            n,
            n.add(u.scale(0.6)).norm(),
            n.sub(u.scale(0.6)).norm(),
            n.add(v.scale(0.6)).norm(),
            n.sub(v.scale(0.6)).norm(),
        ];
        let mut occ = 0.0f32;
        for d in dirs {
            for s in [1.8f32, 4.2f32, 7.5f32] {
                let q = hit_p.add(d.scale(self.cell * s));
                let ix = ((q.x - self.x0) / self.cell).floor() as i32;
                let iy = ((q.y - self.y0) / self.cell).floor() as i32;
                let iz = ((q.z - self.z0) / self.cell).floor() as i32;
                if self.at(ix, iy, iz) != 0 {
                    occ += 1.0 / s;
                }
            }
        }
        (1.0 - occ * 0.24).clamp(0.40, 1.0)
    }

    /// Compute lowest rotated display Z coordinate of the raw stock corners (for the studio grid floor).
    pub fn lowest_display_z(&self, rot: Mat3, center: Vec3) -> f32 {
        let mut lo = f32::INFINITY;
        for x in [self.x0, self.bx1()] {
            for y in [self.y0, self.by1()] {
                for z in [self.z0, self.bz1()] {
                    let p = rot.mul(Vec3::new(x, y, z).sub(center));
                    lo = lo.min(p.z);
                }
            }
        }
        lo
    }
}

fn ray_z_tri(px: f64, py: f64, t: &Triangle) -> Option<f64> {
    let (a, b, c) = (t.a, t.b, t.c);
    let min_x = a.x.min(b.x).min(c.x);
    let max_x = a.x.max(b.x).max(c.x);
    let min_y = a.y.min(b.y).min(c.y);
    let max_y = a.y.max(b.y).max(c.y);
    if px < min_x || px > max_x || py < min_y || py > max_y {
        return None;
    }
    let den = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
    if den.abs() < 1e-12 {
        return None;
    }
    let u = ((b.y - c.y) * (px - c.x) + (c.x - b.x) * (py - c.y)) / den;
    let v = ((c.y - a.y) * (px - c.x) + (a.x - c.x) * (py - c.y)) / den;
    let w = 1.0 - u - v;
    (u >= 0.0 && v >= 0.0 && w >= 0.0).then_some(u * a.z + v * b.z + w * c.z)
}
