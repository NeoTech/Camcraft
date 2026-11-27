//! Klartext tapping cycles: `CYCL DEF 206` (tapping with a floating tap holder) and `CYCL DEF 207` (rigid tapping), defined and run at the cycle call.
//! Provenance: HEIDENHAIN TNC cycle programming manuals (cycle 206 TAPPING NEW, 207 RIGID TAPPING NEW; Q lists in docs/research/heidenhain/differences.md section B3 and cycles.md), general machining knowledge for the Q meanings; the chain follows the G84 shape of `crate::cycle_expand`.

use camcraft_cl::cl::{Cycle, CycleKind, MoveKind, lattice};
use camcraft_geometry::geo2d::Point2;
use camcraft_geometry::geometry::Point3;

use super::cycles::{DrillDef, HoleShape, check_q_list, hole_levels, q_get};
use super::exec::Machine;
use super::source::Block;
use crate::domain::{check_dwell, check_feed, check_len};
use crate::types::GcodeError;

/// Read the Q parameters of `CYCL DEF 206` or `CYCL DEF 207`.
///
/// 206: Q200 set-up clearance, Q201 thread depth (negative), Q206 feed
/// (S x pitch), Q211 dwell at the bottom, Q203 surface, Q204 2nd set-up
/// clearance. 207: Q200, Q201, Q239 thread pitch (its sign is the hand of
/// the thread), Q203, Q204; the feed is S x pitch at the call.
pub fn tap_def(b: &Block, cycle: u32, params: &[(u32, f64)]) -> Result<DrillDef, GcodeError> {
    let (required, allowed): (&[u32], &[u32]) = if cycle == 206 {
        (&[200, 201, 206, 203, 204], &[200, 201, 206, 211, 203, 204])
    } else {
        (&[200, 201, 239, 203, 204], &[200, 201, 239, 203, 204])
    };
    check_q_list(b, cycle, params, required, allowed)?;
    let v = |q: u32| q_get(params, q).unwrap_or(0.0);
    let (r_z, bottom_z, surface_z, clear_z) = hole_levels(b, v(200), v(201), v(203), v(204))?;
    let (shape, feed, dwell) = if cycle == 206 {
        (
            HoleShape::FloatingTap,
            check_feed(b.line, v(206))?,
            check_dwell(b.line, v(211))?,
        )
    } else {
        let pitch = v(239);
        if pitch < 0.0 {
            return Err(b.unsupported(
                "CYCL DEF 207 Q239",
                "a left-hand thread (negative pitch) is not read: the spindle turns against the programmed M3",
            ));
        }
        if pitch == 0.0 {
            return Err(b.invalid_cycle("thread pitch Q239 must not be zero"));
        }
        let pitch = check_len(b.line, "Q239", pitch)?;
        (HoleShape::RigidTap { pitch }, 0.0, 0.0)
    };
    Ok(DrillDef {
        cycle,
        shape,
        r_z,
        bottom_z,
        surface_z,
        clear_z,
        peck: 0.0,
        feed,
        dwell,
    })
}

impl Machine<'_> {
    /// Run a tapping cycle at the current XY position: rapid to the set-up
    /// clearance, feed to the thread depth, (dwell,) feed back out to the
    /// set-up clearance, then rapid to the 2nd set-up clearance if it is higher.
    pub fn tap_call(&mut self, b: &Block, d: &DrillDef) -> Result<(), GcodeError> {
        let (feed, kind) = match d.shape {
            HoleShape::RigidTap { pitch } => {
                let rpm = self
                    .modal
                    .spindle_rpm
                    .filter(|s| *s > 0.0)
                    .ok_or_else(|| {
                        b.invalid_cycle(&format!(
                            "CYCL DEF {} needs a spindle speed S above 0 (TOOL CALL S) for its feed S x Q239",
                            d.cycle
                        ))
                    })?;
                (check_feed(b.line, rpm * pitch)?, CycleKind::Tap)
            }
            HoleShape::FloatingTap => (d.feed, CycleKind::FloatingTap),
            HoleShape::Drill => {
                return Err(b.invalid_cycle("a drilling cycle is not a tapping cycle"));
            }
        };
        let (hx, hy) = (self.pos.x, self.pos.y);
        let at = |z: f64| Point3::new(hx, hy, z);
        self.push_line(b, MoveKind::Rapid, at(d.r_z), None);
        let first_move = self.current.builder.moves_len();
        self.push_line(b, MoveKind::Feed, at(d.bottom_z), Some(feed));
        self.push_line(b, MoveKind::Feed, at(d.r_z), Some(feed));
        if d.clear_z > d.r_z {
            self.push_line(b, MoveKind::Rapid, at(d.clear_z), None);
        }
        self.current.builder.push_cycle(Cycle {
            kind,
            xy: Point2::new(lattice(hx), lattice(hy)),
            r_z: lattice(d.r_z),
            bottom_z: lattice(d.bottom_z),
            entry_z: lattice(d.surface_z),
            peck: 0.0,
            dwell: d.dwell,
            first_move,
        });
        Ok(())
    }
}
