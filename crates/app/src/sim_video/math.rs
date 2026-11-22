//! Linear algebra, quaternion SLERP, camera projection, and analytic ray-tool intersection
//! helpers for the built-in 3D stock-simulation movie renderer.
//!
//! Provenance: clean-room standard 3D graphics vector/quaternion math, slab AABB intersection
//! (Kay-Kajiya), and quadratic ray-cylinder/cone/sphere roots.

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }

    #[inline]
    pub fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }

    #[inline]
    pub fn scale(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    #[inline]
    pub fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }

    #[inline]
    pub fn dot(self, o: Self) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    #[inline]
    pub fn cross(self, o: Self) -> Self {
        Self::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    #[inline]
    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }

    #[inline]
    pub fn norm(self) -> Self {
        let l = self.len();
        if l > 1e-8 { self.scale(1.0 / l) } else { Self::new(0.0, 0.0, 1.0) }
    }
}

/// 3x3 orthogonal rotation matrix (row-major).
#[derive(Clone, Copy, Debug)]
pub(super) struct Mat3 {
    pub m: [[f32; 3]; 3],
}

impl Mat3 {
    /// Convert unit quaternion `[x, y, z, w]` (CamCraft's geometry convention) to a rotation matrix.
    pub fn from_quat(q: [f32; 4]) -> Self {
        let [x, y, z, w] = norm_quat(q);
        let (xx, yy, zz) = (x * x, y * y, z * z);
        let (xy, xz, yz) = (x * y, x * z, y * z);
        let (wx, wy, wz) = (w * x, w * y, w * z);
        Self {
            m: [
                [1.0 - 2.0 * (yy + zz), 2.0 * (xy - wz), 2.0 * (xz + wy)],
                [2.0 * (xy + wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - wx)],
                [2.0 * (xz - wy), 2.0 * (yz + wx), 1.0 - 2.0 * (xx + yy)],
            ],
        }
    }

    #[inline]
    pub fn mul(self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2] * v.z,
            self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2] * v.z,
            self.m[2][0] * v.x + self.m[2][1] * v.y + self.m[2][2] * v.z,
        )
    }

    #[inline]
    pub fn mul_t(self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.m[0][0] * v.x + self.m[1][0] * v.y + self.m[2][0] * v.z,
            self.m[0][1] * v.x + self.m[1][1] * v.y + self.m[2][1] * v.z,
            self.m[0][2] * v.x + self.m[1][2] * v.y + self.m[2][2] * v.z,
        )
    }
}

pub(super) fn norm_quat(q: [f32; 4]) -> [f32; 4] {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n > 1e-8 { [q[0] / n, q[1] / n, q[2] / n, q[3] / n] } else { [0.0, 0.0, 0.0, 1.0] }
}

/// Shortest-arc spherical linear interpolation of unit `[x, y, z, w]` quaternions.
pub(super) fn quat_slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let a = norm_quat(a);
    let mut b = norm_quat(b);
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    if dot < 0.0 {
        dot = -dot;
        b = [-b[0], -b[1], -b[2], -b[3]];
    }
    if dot > 0.9995 {
        return norm_quat([
            a[0] + t * (b[0] - a[0]),
            a[1] + t * (b[1] - a[1]),
            a[2] + t * (b[2] - a[2]),
            a[3] + t * (b[3] - a[3]),
        ]);
    }
    let theta = dot.clamp(-1.0, 1.0).acos();
    let sin_t = theta.sin();
    let wa = ((1.0 - t) * theta).sin() / sin_t;
    let wb = (t * theta).sin() / sin_t;
    norm_quat([
        wa * a[0] + wb * b[0],
        wa * a[1] + wb * b[1],
        wa * a[2] + wb * b[2],
        wa * a[3] + wb * b[3],
    ])
}

/// Pinhole perspective camera looking at `target` with +Z world up.
#[derive(Clone, Copy, Debug)]
pub(super) struct Camera {
    pub pos: Vec3,
    pub fwd: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub tan_half_fov: f32,
    pub width: u32,
    pub height: u32,
}

impl Camera {
    pub fn look_at(eye: Vec3, target: Vec3, fov_deg: f32, width: u32, height: u32) -> Self {
        let fwd = target.sub(eye).norm();
        let ref_up = if fwd.z.abs() > 0.99 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(0.0, 0.0, 1.0) };
        let right = fwd.cross(ref_up).norm();
        let up = right.cross(fwd);
        let tan_half_fov = (fov_deg * 0.5 * std::f32::consts::PI / 180.0).tan();
        Self { pos: eye, fwd, right, up, tan_half_fov, width, height }
    }

    pub fn project(&self, p: Vec3) -> Option<(f32, f32, f32)> {
        let d = p.sub(self.pos);
        let depth = d.dot(self.fwd);
        if depth < 1.0 {
            return None;
        }
        let aspect = self.width as f32 / self.height.max(1) as f32;
        let nx = d.dot(self.right) / (depth * self.tan_half_fov * aspect);
        let ny = d.dot(self.up) / (depth * self.tan_half_fov);
        let sx = (nx * 0.5 + 0.5) * self.width as f32;
        let sy = (0.5 - ny * 0.5) * self.height as f32;
        Some((sx, sy, depth))
    }
}

pub(super) fn intersect_aabb(ro: Vec3, rd: Vec3, b0: Vec3, b1: Vec3) -> Option<(f32, f32)> {
    let r = [ro.x, ro.y, ro.z];
    let d = [rd.x, rd.y, rd.z];
    let lo = [b0.x, b0.y, b0.z];
    let hi = [b1.x, b1.y, b1.z];
    let mut tmin = 0.0f32;
    let mut tmax = f32::INFINITY;
    for i in 0..3 {
        if d[i].abs() < 1e-8 {
            if r[i] < lo[i] || r[i] > hi[i] {
                return None;
            }
        } else {
            let inv = 1.0 / d[i];
            let mut t0 = (lo[i] - r[i]) * inv;
            let mut t1 = (hi[i] - r[i]) * inv;
            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
            }
            tmin = tmin.max(t0);
            tmax = tmax.min(t1);
            if tmin > tmax {
                return None;
            }
        }
    }
    (tmax > 0.0).then_some((tmin, tmax))
}

pub(super) fn intersect_cylinder_z(ro: Vec3, rd: Vec3, r: f32, z0: f32, z1: f32) -> Option<(f32, Vec3)> {
    let mut best = f32::INFINITY;
    let mut bn = Vec3::ZERO;
    let a = rd.x * rd.x + rd.y * rd.y;
    if a > 1e-9 {
        let b = 2.0 * (ro.x * rd.x + ro.y * rd.y);
        let c = ro.x * ro.x + ro.y * ro.y - r * r;
        let disc = b * b - 4.0 * a * c;
        if disc >= 0.0 {
            let t = (-b - disc.sqrt()) / (2.0 * a);
            if t > 0.001 && t < best {
                let zh = ro.z + rd.z * t;
                if zh >= z0 && zh <= z1 {
                    best = t;
                    bn = Vec3::new(ro.x + rd.x * t, ro.y + rd.y * t, 0.0).norm();
                }
            }
        }
    }
    if rd.z.abs() > 1e-6 {
        for (side, zp) in [(false, z0), (true, z1)] {
            let t = (zp - ro.z) / rd.z;
            if t > 0.001 && t < best {
                let hx = ro.x + rd.x * t;
                let hy = ro.y + rd.y * t;
                if hx * hx + hy * hy <= r * r {
                    best = t;
                    bn = Vec3::new(0.0, 0.0, if side { 1.0 } else { -1.0 });
                }
            }
        }
    }
    best.is_finite().then_some((best, bn))
}

pub(super) fn intersect_cone_z(ro: Vec3, rd: Vec3, r0: f32, r1: f32, z0: f32, z1: f32) -> Option<(f32, Vec3)> {
    let h = z1 - z0;
    if h < 1e-5 {
        return None;
    }
    let k = (r1 - r0) / h;
    let oz = ro.z - z0;
    let a = rd.x * rd.x + rd.y * rd.y - k * k * rd.z * rd.z;
    if a.abs() < 1e-9 {
        return None;
    }
    let c0 = r0 + k * oz;
    let b = 2.0 * (ro.x * rd.x + ro.y * rd.y - k * c0 * rd.z);
    let c = ro.x * ro.x + ro.y * ro.y - c0 * c0;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let sq = disc.sqrt();
    let mut best = f32::INFINITY;
    let mut bn = Vec3::ZERO;
    for t in [(-b - sq) / (2.0 * a), (-b + sq) / (2.0 * a)] {
        if t > 0.001 && t < best {
            let zh = ro.z + rd.z * t;
            if zh >= z0 && zh <= z1 {
                let hx = ro.x + rd.x * t;
                let hy = ro.y + rd.y * t;
                let hr = hx.hypot(hy).max(1e-4);
                let n = Vec3::new(hx / hr, hy / hr, -k).norm();
                if n.dot(rd) < 0.0 {
                    best = t;
                    bn = n;
                }
            }
        }
    }
    best.is_finite().then_some((best, bn))
}

pub(super) fn intersect_sphere(ro: Vec3, rd: Vec3, center: Vec3, r: f32) -> Option<(f32, Vec3)> {
    let oc = ro.sub(center);
    let b = oc.dot(rd);
    let c = oc.dot(oc) - r * r;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let t = -b - disc.sqrt();
    (t > 0.001).then(|| (t, ro.add(rd.scale(t)).sub(center).norm()))
}

/// Convert display `part_rot * (p - center)` -> display space.
#[inline]
pub(super) fn part_to_disp(p: Vec3, rot: Mat3, center: Vec3) -> Vec3 {
    rot.mul(p.sub(center))
}

/// Inverse transform display space -> part space.
#[inline]
pub(super) fn disp_to_part(d: Vec3, rot: Mat3, center: Vec3) -> Vec3 {
    rot.mul_t(d).add(center)
}
