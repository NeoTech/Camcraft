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
const MAX_ANGLE: f64 = 99_999.999_9;
/// Largest sweep of one arc move of a `CP` block, in degrees.
const PIECE_DEG: f64 = 90.0;

impl Machine<'_> {
    fn pole(&self, b: &Block, what: &str) -> Result<Point2, GcodeError> {
        self.cc
            .ok_or_else(|| b.invalid_modal(&format!("{what} needs a pole: a CC block before it")))
    }

    /// Polar angle (degrees) of the tool position about the pole.
    fn polar_angle(&self, pole: Point2) -> f64 {
        (self.pos.y - pole.y)
            .atan2(self.pos.x - pole.x)
            .to_degrees()
    }

    /// Polar radius of the tool position about the pole.
    fn polar_radius(&self, pole: Point2) -> f64 {
        (self.pos.x - pole.x).hypot(self.pos.y - pole.y)
    }

    /// Forget the last programmed polar coordinates when the tool left the end
    /// point `end` of the polar block (a cycle call by `M99` can move it).
    fn keep_polar_if_at(&mut self, end: Point3) {
        if self.pos.x != end.x || self.pos.y != end.y {
            self.polar = PolarRef::default();
        }
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
    /// A polar coordinate that is not written, and the base of `IPR` / `IPA`,
    /// is the last programmed value about this pole ([`PolarRef`]); when the
    /// reader does not know that value, the block is refused.
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
        if !moves {
            return self.post_m(b, &w.m, w.mb);
        }
        let last = self.polar;
        let r = polar_value(b, w.pr, w.ipr, last.pr, "PR", "radius")?;
        if r < 0.0 {
            return Err(b.invalid_modal(&format!("LP polar radius ({r}) is negative")));
        }
        // At the pole (PR 0) the angle does not change the end point.
        let a = match polar_value(b, w.pa, w.ipa, last.pa, "PA", "angle") {
            Ok(a) => Some(a),
            Err(_) if r <= 1e-9 => None,
            Err(e) => return Err(e),
        };
        if let Some(a) = a
            && a.abs() > 2.0 * MAX_ANGLE
        {
            return Err(b.invalid_modal(&format!("LP polar angle ({a}) is out of range")));
        }
        let (s, c) = a.unwrap_or(0.0).to_radians().sin_cos();
        let to = check_point(
            b.line,
            "end point",
            Point3::new(pole.x + r * c, pole.y + r * s, self.polar_z(b, &w)?),
        )?;
        let (kind, feed) = line_feed(b, &w, rate, "LP")?;
        self.line_to(b, kind, to, feed)?;
        self.polar = PolarRef { pr: Some(r), pa: a };
        self.post_m(b, &w.m, w.mb)?;
        self.keep_polar_if_at(to);
        Ok(())
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
        let (r, a0) = (self.polar_radius(pole), self.polar_angle(pole));
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
            self.current
                .push_arc(b.line, to, feed, pole, ccw, ArcPlane::Xy);
        }
        let (rx, ry) = ((to.x - pole.x) / r, (to.y - pole.y) / r);
        self.tangent = Some(if ccw {
            Point2::new(-ry, rx)
        } else {
            Point2::new(ry, -rx)
        });
        self.pos = to;
        // CP takes its radius from the tool position, so the last programmed
        // PR stays as it was. The end angle is known when the block writes PA,
        // or when the start angle was known (IPA adds to it).
        let pa = match w.pa {
            Some(a) if w.ipa.is_none() => Some(a),
            _ => self.polar.pa.map(|_| a0 + sweep),
        };
        self.polar.pa = pa;
        self.post_m(b, &w.m, None)?;
        self.keep_polar_if_at(to);
        Ok(())
    }
}

/// The polar coordinates last programmed about the current pole, as far as the
/// reader knows them exactly. A `CC` block that moves the pole, and each move
/// that is not `LP` / `CP` and changes the X/Y position, forget them: the
/// control's last programmed value can then differ from the tool position
/// (docs/decisions/2227-klartext-reader.md, "Polar reference").
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PolarRef {
    /// Last programmed polar radius `PR`.
    pub pr: Option<f64>,
    /// Last programmed polar angle `PA`, in degrees.
    pub pa: Option<f64>,
}

/// One polar coordinate of an `LP` block: the absolute word, the incremental
/// word added to the last programmed value, or the last programmed value.
fn polar_value(
    b: &Block,
    abs: Option<f64>,
    inc: Option<f64>,
    last: Option<f64>,
    word: &str,
    what: &str,
) -> Result<f64, GcodeError> {
    let unknown = |how: &str| {
        b.invalid_modal(&format!(
            "LP {how}: the last polar {what} about this pole is not known (write {word})"
        ))
    };
    match (abs, inc, last) {
        (Some(v), _, _) => Ok(v),
        (None, Some(d), Some(l)) => Ok(l + d),
        (None, Some(_), None) => Err(unknown(&format!("I{word}"))),
        (None, None, Some(l)) => Ok(l),
        (None, None, None) => Err(unknown(&format!("without {word}"))),
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
