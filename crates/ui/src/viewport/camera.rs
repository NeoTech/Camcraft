//! Orbit camera state, movement and world-space basis.
//! Provenance: our existing Z-up orbit camera, from public graphics literature.

mod projection;

use super::ORTHO_EYE_DIST;

/// A world direction spelled as its three components - what `normalize`,
/// `fwd`, `basis_right` and `basis_up` all produce, so the orthonormal
/// basis `view_basis` returns is three of these.
type UnitVec = (f64, f64, f64);

/// How the viewport flattens the world to the screen (#434).
/// Ephemeral view state, like the camera itself: a direct field flip,
/// never a command, never saved in the project.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProjectionMode {
    /// The house default: a 60-degree perspective cone, converging lines.
    #[default]
    Perspective,
    /// Parallel projection (issue #434): the view half-height is fixed
    /// at `dist * tan(30 deg)` - the height perspective sees at the
    /// target plane - so switching mid-session never jumps the framing
    /// and `dist` keeps spelling zoom for orbit, pan and `frame` alike.
    Orthographic,
}

/// Orbit camera around a target point, Z-up (the CAM convention).
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub target: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub dist: f64,
    /// How this camera projects; carried by every camera transform so
    /// draw, pick and dolly math stay on one matrix by construction.
    pub projection: ProjectionMode,
    /// The orbit frame (#1993): which world axis stays up. Yaw and pitch
    /// are read in this frame; the eye and basis map back to the world.
    pub frame: crate::view_frame::ViewFrame,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            target: [0.0, 0.0, 0.0],
            yaw: 0.9,
            pitch: 0.5,
            dist: 80.0,
            projection: ProjectionMode::default(),
            frame: crate::view_frame::ViewFrame::default(),
        }
    }
}

impl Camera {
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw -= f64::from(dx) * 0.01;
        self.pitch = (self.pitch + f64::from(dy) * 0.01).clamp(-1.5, 1.5);
        if !self.yaw.is_finite() {
            self.yaw = self.yaw.rem_euclid(std::f64::consts::TAU);
            if !self.yaw.is_finite() {
                self.yaw = 0.0;
            }
        }
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        let scale = self.dist * 0.0015;
        let (rx, ry, rz) = self.basis_right();
        let (ux, uy, uz) = self.basis_up();
        self.target = [
            self.target[0] - f64::from(dx) * scale * rx + f64::from(dy) * scale * ux,
            self.target[1] - f64::from(dx) * scale * ry + f64::from(dy) * scale * uy,
            self.target[2] - f64::from(dx) * scale * rz + f64::from(dy) * scale * uz,
        ];
        if self.target.iter().any(|t| !t.is_finite()) {
            self.target = [0.0, 0.0, 0.0];
        }
    }

    pub fn zoom(&mut self, wheel_steps: f32) {
        // One wheel click ~= 10% dolly; clamped to sane machining distances.
        let factor = (1.0 - f64::from(wheel_steps) * 0.1).clamp(0.5, 2.0);
        self.dist = (self.dist * factor).clamp(0.5, 100_000.0);
        if !self.dist.is_finite() {
            self.dist = 80.0;
        }
    }


    /// Center the camera on a bounding sphere.
    #[must_use]
    pub fn frame(center: [f64; 3], radius: f64) -> Camera {
        Camera {
            target: center,
            yaw: 0.9,
            pitch: 0.5,
            dist: (radius * 3.2).clamp(1.0, 50_000.0),
            projection: ProjectionMode::default(),
            frame: crate::view_frame::ViewFrame::default(),
        }
    }

    /// The same camera in orbit frame `frame`, at that frame's home
    /// (yaw, pitch) - the view a newly mounted machine kind opens on
    /// (#1993). Target, distance and projection are kept.
    #[must_use]
    pub fn with_frame(&self, frame: crate::view_frame::ViewFrame) -> Camera {
        let (yaw, pitch) = frame.home();
        Camera { frame, ..self.oriented(yaw, pitch) }
    }

    /// Look at the same target, from the same distance, along the world
    /// direction `d` (the eye stands at `target + dist * d`): the view-cube
    /// law of [`crate::egui_app::viewcube`], and the one place a
    /// standard orientation turns yaw/pitch. Non-finite `d` is a no-op
    /// that answers `self` - the cube's classification is finite by
    /// construction, so this is the Prime Directive's belt, not a path.
    /// The straight-up/straight-down cases need a special yaw: `d.x`,
    /// `d.y` vanish there, so `atan2` answers 0 and every pole click
    /// would land on one canonical roll, spinning a yawed approach up to
    /// half a turn on screen (#622). Instead the pole takes the *pre-
    /// click* yaw snapped to the nearest quarter turn - the roll the view
    /// had, kept; `basis_right`'s zero-guard then evaluates that yaw, so
    /// a pole view rolls like its yaw and a second pole click changes
    /// nothing (`docs/decisions/622-viewcube-roll-snap.md`).
    #[must_use]
    pub fn toward(&self, d: [f64; 3]) -> Camera {
        if d.iter().any(|v| !v.is_finite()) {
            return *self;
        }
        let mut n = d;
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        // Degenerate or non-finite length is a no-op (a finite `d` can
        // overflow its squares to infinity, never to NaN).
        if l <= f64::EPSILON || l.is_nan() {
            return *self;
        }
        n = [n[0] / l, n[1] / l, n[2] / l];
        // The cube names world directions; yaw and pitch live in the
        // orbit frame (#1993).
        let (a, b, c) = self.frame.to_local((n[0], n[1], n[2]));
        n = [a, b, c];
        // At an exact Z pole the atan2 vanishes; take the roll from the
        // yaw this camera already had, snapped to the nearest quarter
        // turn, so the click levels the view without turning it (#622).
        let quarter = std::f64::consts::FRAC_PI_2;
        let yaw = if n[2].abs() == 1.0 {
            (self.yaw / quarter).round() * quarter
        } else {
            n[1].atan2(n[0])
        };
        Camera {
            target: self.target,
            yaw,
            pitch: n[2].clamp(-1.0, 1.0).asin(),
            dist: self.dist,
            projection: self.projection,
            frame: self.frame,
        }
    }

    /// The same camera at an exact (yaw, pitch) - the house isometric the
    /// frame default keeps, spelled for the nav bar.
    #[must_use]
    pub fn oriented(&self, yaw: f64, pitch: f64) -> Camera {
        Camera {
            target: self.target,
            yaw,
            pitch,
            dist: self.dist,
            projection: self.projection,
            frame: self.frame,
        }
    }

    /// The camera's orthonormal world basis (right, up, forward) - the
    /// rotation a view-cube projection is drawn through, and the frame
    /// an edge or corner hit is classified in.
    #[must_use]
    pub fn view_basis(&self) -> (UnitVec, UnitVec, UnitVec) {
        (self.basis_right(), self.basis_up(), self.fwd())
    }

    #[must_use]
    pub(crate) fn eye(&self) -> [f64; 3] {
        let d = if self.projection == ProjectionMode::Orthographic {
            ORTHO_EYE_DIST
        } else {
            self.dist
        };
        let cp = self.pitch.cos();
        let (ox, oy, oz) = self.frame.to_world((
            d * cp * self.yaw.cos(),
            d * cp * self.yaw.sin(),
            d * self.pitch.sin(),
        ));
        [self.target[0] + ox, self.target[1] + oy, self.target[2] + oz]
    }

    /// Camera position as f32, for the shader's specular term.
    #[must_use]
    pub fn eye_f32(&self) -> [f32; 3] {
        let finite = |v: f64| if v.is_finite() { v as f32 } else { 0.0 };
        std::array::from_fn(|i| finite(self.eye()[i]))
    }

    /// Forward from eye toward target, normalized.
    #[must_use]
    fn fwd(&self) -> (f64, f64, f64) {
        let e = self.eye();
        normalize([self.target[0] - e[0], self.target[1] - e[1], self.target[2] - e[2]])
    }

    /// Right = normalize(cross(forward, up)), with up the orbit frame's
    /// local +Z (world Z on a mill, #1993); at an exact pole the cross
    /// vanishes, and the guard evaluates the same expression at the camera's
    /// (quarter-snapped, see `toward`) yaw instead - the limit every
    /// near-pole view already rides (#622).
    #[must_use]
    fn basis_right(&self) -> (f64, f64, f64) {
        let (fx, fy, _) = self.frame.to_local(self.fwd());
        let (x, y, _) = normalize([fy, -fx, 0.0]);
        if x == 0.0 && y == 0.0 {
            // looking straight along the frame's up axis
            return self.frame.to_world((-self.yaw.sin(), self.yaw.cos(), 0.0));
        }
        self.frame.to_world((x, y, 0.0))
    }

    /// Up = cross(right, forward).
    #[must_use]
    fn basis_up(&self) -> (f64, f64, f64) {
        let f = {
            let (x, y, z) = self.fwd();
            [x, y, z]
        };
        let (rx, ry, rz) = self.basis_right();
        normalize([ry * f[2] - rz * f[1], rz * f[0] - rx * f[2], rx * f[1] - ry * f[0]])
    }
}

fn normalize(v: [f64; 3]) -> (f64, f64, f64) {
    let l = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if l <= f64::EPSILON {
        return (0.0, 0.0, 0.0);
    }
    let l = l.sqrt();
    (v[0] / l, v[1] / l, v[2] / l)
}
