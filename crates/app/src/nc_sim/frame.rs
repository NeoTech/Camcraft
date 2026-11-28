//! NC program coordinates to world millimetres (#2218): `world = origin +
//! sign * program_mm`, the inverse of the post's origin shift
//! (`posting::wcs_context`), its WCS flip and the machine's inverted axes
//! (`post::invert`). Provenance: plane geometry and the RS-274 rule that
//! G2/G3 read from the plane normal's positive end.

use flecs_ecs::prelude::World;

use crate::cl::{ArcPlane, Move};
use crate::geo2d::Point2;
use crate::geometry::Point3;
use crate::machine::Machine;

use super::NcSimOptions;

/// The map of one work offset's program frame onto the world frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NcFrame {
    /// The work offset origin, world millimetres.
    pub origin: [f64; 3],
    /// The setup's WCS flip, as `true` per negated axis.
    pub flip: [bool; 3],
    /// Program to world sign per axis: the flip times the machine's axis direction.
    pub sign: [f64; 3],
}

fn offset_word(index: usize) -> String {
    format!("G{}", 54 + index)
}

/// The frame of work offset `index` (0 for `G54` .. 5 for `G59`).
///
/// The origin is the caller's override for that offset, else the setup's
/// own WCS when the setup states that offset. Any other offset is refused
/// by name, never read as the setup's origin. A setup WCS that is turned
/// is refused too: this map holds a translation and a sign only.
///
/// # Errors
/// Returns the refusal text.
pub fn frame_for(world: &World, opts: &NcSimOptions, machine: Option<&Machine>, index: usize) -> Result<NcFrame, String> {
    let setup = opts.setup.as_deref();
    let origin = match opts.wcs_origins.get(index).copied().flatten() {
        Some(o) => o,
        None => {
            let slot = crate::setup::wcs_slot(world, setup);
            if slot != index {
                return Err(format!(
                    "the program uses {}, the setup states {} - give an origin for {} to simulate it",
                    offset_word(index),
                    crate::setup::wcs_name(world, setup),
                    offset_word(index)
                ));
            }
            if let Some(loc) = crate::setup::wcs_entity(world, setup).and_then(|e| crate::model::transform_of(world, &e))
                && !crate::setup_align::is_identity_rot(loc.rot)
            {
                return Err(format!(
                    "the setup's WCS ({}) is turned - the NC simulation maps a square WCS only",
                    crate::setup::wcs_name(world, setup)
                ));
            }
            crate::posting::wcs_context(world, setup).1
        }
    };
    if !origin.iter().all(|v| v.is_finite()) {
        return Err(format!("the origin of {} is not finite: {origin:?}", offset_word(index)));
    }
    let flip = crate::setup_wcs_flip::of_setup(world, setup);
    let mut sign = [1.0; 3];
    for (k, s) in sign.iter_mut().enumerate() {
        let dir = machine.and_then(|m| m.axis(k)).map_or(1.0, |a| a.direction.sign());
        *s = dir * if flip[k] { -1.0 } else { 1.0 };
    }
    Ok(NcFrame { origin, flip, sign })
}

impl NcFrame {
    fn point(&self, p: Point3) -> Point3 {
        Point3::new(
            self.origin[0] + self.sign[0] * p.x,
            self.origin[1] + self.sign[1] * p.y,
            self.origin[2] + self.sign[2] * p.z,
        )
    }

    /// One program move in world millimetres. An arc's center maps with
    /// its plane's two axes, and its sense reverses when exactly one of
    /// them is negated.
    ///
    /// # Errors
    /// Refuses a posed or rotary move and a move that is not finite.
    pub fn move_to_world(&self, m: &Move) -> Result<Move, String> {
        if m.axis.is_some() || m.rotary.is_some() {
            return Err("a posed or rotary move - the NC simulation is 3-axis".into());
        }
        if !m.is_finite() {
            return Err("a move with a coordinate or feed that is not finite".into());
        }
        let mut out = *m;
        out.from = self.point(m.from);
        out.to = self.point(m.to);
        if let Some(mut a) = m.arc {
            let (u, v) = match a.plane {
                ArcPlane::Xy => (0, 1),
                ArcPlane::Xz => (0, 2),
                ArcPlane::Yz => (1, 2),
            };
            a.center = Point2::new(
                self.origin[u] + self.sign[u] * a.center.x,
                self.origin[v] + self.sign[v] * a.center.y,
            );
            if self.sign[u] * self.sign[v] < 0.0 {
                a.ccw = !a.ccw;
            }
            out.arc = Some(a);
        }
        Ok(out)
    }
}
