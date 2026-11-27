//! Klartext polar path blocks about the pole `CC`: `LP PR.. PA..` straight lines and `CP PA../IPA.. DR±` arcs, with `Z`/`IZ` in `CP` as a helix.
//! Provenance: HEIDENHAIN iTNC 530 conversational user's manual 533 190-22 (polar coordinates: pole CC, polar radius PR, polar angle PA from the +X axis, counter-clockwise positive; LP and CP; helix as CP with IPA and IZ, the signs of IPA and DR agree), as summarized in docs/research/heidenhain/differences.md.

use camcraft_cl::cl::ArcPlane;
use camcraft_geometry::geo2d::Point2;
use camcraft_geometry::geometry::Point3;

use super::exec::Machine;
use super::path::line_feed;
use super::source::Block;
use super::words::{Feed, Words, decode_polar};
use crate::domain::{check_len, check_point};
use crate::types::GcodeError;

/// Largest polar angle input of the control, in degrees.
const MAX_ANGLE: f64 = 99_999.9999;
/// Largest sweep of one arc move of a `CP` block, in degrees.
const PIECE_DEG: f64 = 90.0;

impl Machine<'_> {
    fn pole(&self, b: &Block, what: &str) -> Result<Point2, GcodeError> {
        self.cc
            .ok_or_else(|| b.invalid_modal(&format!("{what} needs a pole: a CC block before it")))
    }

    /// Polar radius and angle (degrees) of the tool position about the pole.
    fn polar_pos(&self, pole: Point2) -> (f64, f64) {
        let (dx, dy) = (self.pos.x - pole.x, self.pos.y - pole.y);
        (dx.hypot(dy), dy.atan2(dx).to_degrees())
    }

    /// The tool-axis end of a polar block: `Z`, `IZ`, or the current Z.
    fn polar_z(&self, b: &Block, w: &Words) -> Result<f64, GcodeError> {
        let z = match (w.abs[2], w.inc[2]) {
            (Some(z), _) => z,
            (None, Some(d)) => self.pos.z + d,
            (None, None) => self.pos.z,
        };
        check_len(b.line, "Z", z)
    }

    /// `LP PR.. PA.. [Z..] [R0] F../FMAX`: a straight line to polar coordinates.
    /// A polar coordinate that is not written keeps its value of the tool position.
    pub fn polar_line_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let w = decode_polar(b, toks, &self.q)?;
        if w.ccw.is_some() || w.radius.is_some() {
            return Err(b.syntax("LP takes no DR and no R radius"));
        }
        let pole = self.pole(b, "LP")?;
        let moves = w.pr.or(w.ipr).or(w.pa).or(w.ipa).is_some() || w.has_coords();
        self.corner_words_check(b, &w, moves)?;
        self.pre_m(b, &w.m)?;
        let rate = self.feed_words(b, &w)?;
        if moves {
            let (r0, a0) = self.polar_pos(pole);
            let r = match (w.pr, w.ipr) {
                (Some(r), _) => r,
                (None, Some(d)) => r0 + d,
                (None, None) => r0,
            };
            if r < 0.0 {
                return Err(b.invalid_modal(&format!("LP polar radius ({r}) is negative")));
            }
            let angle_from_pos = w.pa.is_none();
            if angle_from_pos && r0 <= 1e-9 && r > 1e-9 {
                return Err(b.invalid_modal(
                    "LP needs PA: the tool is at the pole, so its polar angle is not defined",
                ));
            }
            let a = match (w.pa, w.ipa) {
                (Some(a), _) => a,
                (None, Some(d)) => a0 + d,
                (None, None) => a0,
            };
            if a.abs() > 2.0 * MAX_ANGLE {
                return Err(b.invalid_modal(&format!("LP polar angle ({a}) is out of range")));
            }
            let (s, c) = a.to_radians().sin_cos();
            let to = check_point(
                b.line,
                "end point",
                Point3::new(pole.x + r * c, pole.y + r * s, self.polar_z(b, &w)?),
            )?;
            let (kind, feed) = line_feed(b, &w, rate, "LP")?;
            self.line_to(b, kind, to, feed)?;
        }
        self.post_m(b, &w.m, w.mb)
    }

    /// `CP PA.. DR±` or `CP IPA.. DR±` (with `Z`/`IZ`: a helix): an arc about the
    /// pole at the radius of the tool position.
    pub fn polar_arc_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let w = decode_polar(b, toks, &self.q)?;
        if w.pr.is_some() || w.ipr.is_some() || w.radius.is_some() {
            return Err(b.syntax("CP takes no PR and no R; the radius is the distance to the pole"));
        }
        if w.mb.is_some() {
            return Err(b.syntax("MB on an arc block"));
        }
        if w.feed == Some(Feed::Max) {
            return Err(b.invalid_arc("FMAX on an arc is not read; give a feed rate F"));
        }
        let ccw = w.ccw.ok_or_else(|| b.invalid_arc("CP needs DR+ or DR-"))?;
        let pole = self.pole(b, "CP")?;
        let (r, a0) = self.polar_pos(pole);
        if r <= 1e-6 {
            return Err(b.invalid_arc("CP starts at the pole; the arc has no radius"));
        }
        let sweep = cp_sweep(b, &w, a0, ccw)?;
        self.pre_m(b, &w.m)?;
        let feed = self
            .feed_words(b, &w)?
            .ok_or_else(|| b.invalid_modal("an arc needs a feed rate F"))?;
        let z0 = self.pos.z;
        let z1 = self.polar_z(b, &w)?;
        // At most 1112 pieces: the sweep is bounded by MAX_ANGLE.
        let n = (sweep.abs() / PIECE_DEG).ceil().max(1.0) as usize;
        let mut to = self.pos;
        for k in 1..=n {
            let f = k as f64 / n as f64;
            let (s, c) = (a0 + sweep * f).to_radians().sin_cos();
            to = check_point(
                b.line,
                "arc point",
                Point3::new(pole.x + r * c, pole.y + r * s, z0 + (z1 - z0) * f),
            )?;
            self.current.push_arc(b.line, to, feed, pole, ccw, ArcPlane::Xy);
        }
        let (rx, ry) = ((to.x - pole.x) / r, (to.y - pole.y) / r);
        self.tangent = Some(if ccw {
            Point2::new(-ry, rx)
        } else {
            Point2::new(ry, -rx)
        });
        self.pos = to;
        self.post_m(b, &w.m, None)
    }
}

/// Signed sweep of a `CP` block in degrees (positive counter-clockwise).
fn cp_sweep(b: &Block, w: &Words, a0: f64, ccw: bool) -> Result<f64, GcodeError> {
    match (w.pa, w.ipa) {
        (_, Some(d)) => {
            if !(d.abs() > 1e-9 && d.abs() <= MAX_ANGLE) {
                return Err(b.invalid_arc(&format!(
                    "CP IPA ({d}) must be non-zero and at most {MAX_ANGLE} degrees"
                )));
            }
            if (d > 0.0) != ccw {
                return Err(b.invalid_arc(
                    "CP IPA and DR have different signs; the direction of the arc is not clear",
                ));
            }
            Ok(d)
        }
        (Some(a), None) => {
            if a.abs() > MAX_ANGLE {
                return Err(b.invalid_arc(&format!("CP PA ({a}) is out of range")));
            }
            let d = if ccw {
                (a - a0).rem_euclid(360.0)
            } else {
                -(a0 - a).rem_euclid(360.0)
            };
            if d.abs() <= 1e-9 || (360.0 - d.abs()) <= 1e-9 {
                return Err(b.invalid_arc(
                    "CP PA is the start angle; write IPA+360 or IPA-360 for a full circle",
                ));
            }
            Ok(d)
        }
        (None, None) => Err(b.invalid_arc("CP needs the end angle PA or IPA")),
    }
}
