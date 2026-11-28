//! Camera rays, cursor dolly and model-view-projection matrices.
//! Provenance: our existing projection math, from public graphics literature.

use super::{Camera, ProjectionMode, ORTHO_EYE_DIST};

impl Camera {
    /// Ray through an NDC cursor position (x right, y up, -1..1):
    /// (origin, normalized direction), or None on degenerate math.
    /// In [`ProjectionMode::Orthographic`] the ray is parallel to the
    /// view axis and its origin is the cursor's point on the near plane,
    /// [`Camera::ortho_behind_eye`] behind the eye plane - the mirror of
    /// the ortho box [`Camera::mvp`] draws, so a pick through the same
    /// matrix hits what the screen shows (#434), nearer faces included (#2375).
    #[must_use]
    pub fn ray_through(&self, ndc: [f32; 2], aspect: f64) -> Option<([f64; 3], [f64; 3])> {
        let (fx, fy, fz) = self.fwd();
        let (rx, ry, rz) = self.basis_right();
        let (ux, uy, uz) = self.basis_up();
        let g = (std::f64::consts::FRAC_PI_6).tan(); // half-height at d = 1
        let aspect = if aspect.is_finite() && aspect > 1e-6 {
            aspect
        } else {
            1.0
        };
        let (nx, ny) = (f64::from(ndc[0]), f64::from(ndc[1]));
        if self.projection == ProjectionMode::Orthographic {
            let h = self.ortho_half_height();
            let w = h * aspect;
            let e = self.eye();
            let back = self.ortho_behind_eye();
            let origin = [
                e[0] + nx * w * rx + ny * h * ux - back * fx,
                e[1] + nx * w * ry + ny * h * uy - back * fy,
                e[2] + nx * w * rz + ny * h * uz - back * fz,
            ];
            let dir = [fx, fy, fz];
            if !origin.iter().all(|v| v.is_finite())
                || !dir.iter().all(|v| v.is_finite())
                || dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2] < 1e-24
            {
                return None;
            }
            return Some((origin, dir));
        }
        let dir = [
            fx + nx * g * aspect * rx + ny * g * ux,
            fy + nx * g * aspect * ry + ny * g * uy,
            fz + nx * g * aspect * rz + ny * g * uz,
        ];
        let len2 = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
        if !len2.is_finite() || len2 < 1e-24 {
            return None;
        }
        let inv = 1.0 / len2.sqrt();
        let dir = [dir[0] * inv, dir[1] * inv, dir[2] * inv];
        let origin = self.eye();
        if !dir.iter().all(|v| v.is_finite()) || !origin.iter().all(|v| v.is_finite()) {
            return None;
        }
        Some((origin, dir))
    }

    /// Dolly toward the cursor, keeping the world point under it fixed.
    /// `ndc` is the cursor in normalized device coords (x right, y up,
    /// both -1..1) and `aspect` the viewport width/height. Returns the new
    /// target, or None on a degenerate ray (caller keeps plain behavior).
    /// In [`ProjectionMode::Orthographic`] the fixed point is the
    /// cursor's own point on the target plane - ortho's parallel rays
    /// have no "closest point to target", and shrinking `dist` shrinks
    /// the ortho box by the same factor the target moves, so the point
    /// under the cursor is fixed exactly (#434).
    pub fn dolly_cursor(
        &mut self,
        ndc: [f32; 2],
        aspect: f64,
        wheel_steps: f32,
    ) -> Option<[f64; 3]> {
        let (fx, fy, fz) = self.fwd();
        let (rx, ry, rz) = self.basis_right();
        let (ux, uy, uz) = self.basis_up();
        let g = (std::f64::consts::FRAC_PI_6).tan(); // half-height at d = 1
        let aspect = if aspect.is_finite() && aspect > 1e-6 {
            aspect
        } else {
            1.0
        };
        let (nx, ny) = (f64::from(ndc[0]), f64::from(ndc[1]));
        if !nx.is_finite() || !ny.is_finite() {
            return None;
        }
        let (tx, ty, tz) = (self.target[0], self.target[1], self.target[2]);
        let (px, py, pz) = if self.projection == ProjectionMode::Orthographic {
            let h = self.ortho_half_height();
            let w = h * aspect;
            (
                tx + nx * w * rx + ny * h * ux,
                ty + nx * w * ry + ny * h * uy,
                tz + nx * w * rz + ny * h * uz,
            )
        } else {
            let (ex, ey, ez) = (self.eye()[0], self.eye()[1], self.eye()[2]);
            let dir = [
                fx + nx * g * aspect * rx + ny * g * ux,
                fy + nx * g * aspect * ry + ny * g * uy,
                fz + nx * g * aspect * rz + ny * g * uz,
            ];
            // Closest point on the ray to the current target.
            let (wx, wy, wz) = (tx - ex, ty - ey, tz - ez);
            let dd = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
            if !dd.is_finite() || dd <= 1e-12 {
                return None;
            }
            let s = (wx * dir[0] + wy * dir[1] + wz * dir[2]) / dd;
            (ex + dir[0] * s, ey + dir[1] * s, ez + dir[2] * s)
        };
        let old_dist = self.dist;
        self.zoom(wheel_steps);
        let k = self.dist / old_dist;
        if !k.is_finite() || k <= 0.0 {
            return None;
        }
        self.target = [px + (tx - px) * k, py + (ty - py) * k, pz + (tz - pz) * k];
        if self.target.iter().any(|t| !t.is_finite()) {
            return None;
        }
        Some(self.target)
    }

    /// Column-major model-view-projection (right-handed, camera looks -Z).
    #[must_use]
    pub fn mvp(&self, aspect: f64) -> [[f32; 4]; 4] {
        let e = self.eye();
        let (f0, f1, f2) = self.fwd();
        let (rx, ry, rz) = self.basis_right();
        let (ux, uy, uz) = self.basis_up();
        let view = [
            [rx, ux, -f0, 0.0],
            [ry, uy, -f1, 0.0],
            [rz, uz, -f2, 0.0],
            [
                -(rx * e[0] + ry * e[1] + rz * e[2]),
                -(ux * e[0] + uy * e[1] + uz * e[2]),
                f0 * e[0] + f1 * e[1] + f2 * e[2],
                1.0,
            ],
        ];
        // The near plane rides the view distance (minimally far from
        // the eye): a fixed 0.1 clipped against any part smaller than
        // half a millimetre near the camera, so zooming magnified cut
        // the near faces away at the surface.
        let aspect = if aspect.is_finite() && aspect > 1e-6 {
            aspect
        } else {
            1.0
        };
        // Perspective clips 0.1 mm in front of the eye - enough to kill
        // the near-plane tunnel problem without a visible crop at sane
        // distances. `dist` clamps to 0.5 mm, so the 0.1 mm plane still
        // sits inside the clip volume at every zoom.
        let near = (self.dist * 0.001).clamp(1e-5, 0.1);
        let far = (self.dist * 40.0).max(1000.0);
        let proj = if self.projection == ProjectionMode::Orthographic {
            // The eye stays a fixed nominal distance behind the target,
            // so a near plane in front of it would sit 99.9 mm ahead of
            // the target plane and any nearer geometry would vanish.
            // The near plane reaches far behind the eye plane so nothing
            // inside the visible box clips away.
            let near = -self.ortho_behind_eye();
            // Parallel box (#434): half-height is what perspective sees at
            // the target plane, so `dist` keeps spelling zoom and the
            // mode switch never jumps the framing. Depth is linear and
            // clip.w is 1 everywhere, so every screen-space consumer that
            // divides by w degenerates cleanly onto this matrix too.
            // The depth row spans near -> 0.0 and far -> 1.0, NOT the
            // OpenGL [-1, +1] span: WebGPU clips on 0 <= z <= w, and with
            // clip.w held at 1 the OpenGL span puts every point nearer
            // than (near + far) / 2 - everything in view - at negative
            // clip z and blanks the viewport (#600).
            let h = self.ortho_half_height();
            let w = h * aspect;
            [
                [1.0 / w, 0.0, 0.0, 0.0],
                [0.0, 1.0 / h, 0.0, 0.0],
                [0.0, 0.0, 1.0 / (near - far), 0.0],
                [0.0, 0.0, near / (near - far), 1.0],
            ]
        } else {
            let g = (std::f64::consts::FRAC_PI_6).tan().recip(); // 60 deg vertical fov
            [
                [g / aspect, 0.0, 0.0, 0.0],
                [0.0, g, 0.0, 0.0],
                [0.0, 0.0, (far + near) / (near - far), -1.0],
                [0.0, 0.0, 2.0 * far * near / (near - far), 0.0],
            ]
        };
        mat_mul(proj, view)
    }

    /// Orthographic view half-height: the height the perspective cone
    /// sees at the target plane, so both modes frame identically at the
    /// same `dist` (#434). `dist` is clamped >= 0.5 by `zoom`, so this
    /// never vanishes.
    #[must_use]
    pub(crate) fn ortho_half_height(&self) -> f64 {
        self.dist * (std::f64::consts::FRAC_PI_6).tan()
    }

    /// How far the orthographic near plane sits behind the eye plane, mm:
    /// [`Camera::mvp`] clips there and [`Camera::ray_through`] starts there.
    #[must_use]
    pub(crate) fn ortho_behind_eye(&self) -> f64 {
        (self.dist * 20.0).max(1000.0) - ORTHO_EYE_DIST
    }
}

/// Column-major matrix product: `out[c][r] = sum_k a[k][r] * b[c][k]`.
/// Non-finite results become 0 rather than poisoning the whole matrix.
fn mat_mul(a: [[f64; 4]; 4], b: [[f64; 4]; 4]) -> [[f32; 4]; 4] {
    std::array::from_fn(|c| {
        std::array::from_fn(|r| {
            let mut s = 0.0f64;
            for k in 0..4 {
                s += a[k][r] * b[c][k];
            }
            if s.is_finite() { s as f32 } else { 0.0 }
        })
    })
}
