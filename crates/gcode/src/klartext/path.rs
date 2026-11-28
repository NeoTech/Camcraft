//! Klartext path blocks: `L` lines, `CC` circle centre, `C` / `CR` / `CT` arcs in the XY plane, and the `M140` tool-axis retract.
//! Provenance: HEIDENHAIN iTNC 530 conversational user's manual 533 190-22 (path functions L, CC, C, CR, CT; M140), as summarized in docs/research/heidenhain/differences.md; arc resolution shared with the RS-274 reader (`crate::arc`).

use camcraft_cl::cl::{ArcPlane, MoveKind};
use camcraft_geometry::geo2d::Point2;
use camcraft_geometry::geometry::Point3;

use super::exec::Machine;
use super::source::Block;
use super::words::{Feed, Mb, Words, decode};
use crate::arc::resolve_arc;
use crate::domain::{check_feed, check_len, check_point};
use crate::types::{ArcCenterMode, GcodeError};

impl Machine<'_> {
    pub fn line_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let w = decode(b, toks, &self.q)?;
        reject_arc_words(b, &w, "L")?;
        self.pre_m(b, &w.m)?;
        let rate = self.feed_words(b, &w)?;
        if w.has_coords() {
            let to = self.target(b, &w)?;
            let (kind, feed) = match w.feed {
                Some(Feed::Max) => (MoveKind::Rapid, None),
                _ => (
                    MoveKind::Feed,
                    Some(rate.ok_or_else(|| b.invalid_modal("L needs a feed rate F or FMAX"))?),
                ),
            };
            self.push_line(b, kind, to, feed);
        }
        self.post_m(b, &w.m, w.mb)
    }

    pub fn cc_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let w = decode(b, toks, &self.q)?;
        if w.abs[2].is_some() || w.inc[2].is_some() {
            return Err(b.unsupported("CC Z", "only XY circle centres are read (tool axis Z)"));
        }
        if w.feed.is_some()
            || w.r0
            || w.ccw.is_some()
            || w.radius.is_some()
            || !w.m.is_empty()
            || w.mb.is_some()
        {
            return Err(b.syntax("CC takes only X/Y or IX/IY"));
        }
        let given = |k: usize| w.abs[k].is_some() || w.inc[k].is_some();
        if given(0) != given(1) {
            return Err(
                b.invalid_arc("CC needs both X and Y, or no coordinate (the last position)")
            );
        }
        let axis = |k: usize, cur: f64| match (w.abs[k], w.inc[k]) {
            (Some(v), _) => v,
            (None, Some(d)) => cur + d,
            (None, None) => cur,
        };
        let c = Point2::new(
            check_len(b.line, "CC X", axis(0, self.pos.x))?,
            check_len(b.line, "CC Y", axis(1, self.pos.y))?,
        );
        self.cc = Some(c);
        Ok(())
    }

    pub fn arc_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let w = decode(b, toks, &self.q)?;
        if w.radius.is_some() {
            return Err(b.syntax("C takes no radius; use CR for a radius arc"));
        }
        let ccw = w.ccw.ok_or_else(|| b.invalid_arc("C needs DR+ or DR-"))?;
        let c = self
            .cc
            .ok_or_else(|| b.invalid_arc("C needs a circle centre CC before it"))?;
        self.run_arc(b, &w, |m, to| {
            resolve_arc(
                b.line,
                m.pos,
                to,
                ArcPlane::Xy,
                ccw,
                ArcCenterMode::Absolute,
                [Some(c.x), Some(c.y), None],
                None,
                m.opts.arc_radius_tolerance_mm,
            )
            .map(|r| (r, ccw))
        })
    }

    pub fn radius_arc_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let w = decode(b, toks, &self.q)?;
        let ccw = w.ccw.ok_or_else(|| b.invalid_arc("CR needs DR+ or DR-"))?;
        let r = w
            .radius
            .ok_or_else(|| b.invalid_arc("CR needs a radius R+ or R-"))?;
        let r = check_len(b.line, "CR radius", r)?;
        self.run_arc(b, &w, |m, to| {
            resolve_arc(
                b.line,
                m.pos,
                to,
                ArcPlane::Xy,
                ccw,
                ArcCenterMode::Absolute,
                [None, None, None],
                Some(r),
                m.opts.arc_radius_tolerance_mm,
            )
            .map(|res| (res, ccw))
        })
    }

    pub fn tangent_arc_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let w = decode(b, toks, &self.q)?;
        if w.ccw.is_some() || w.radius.is_some() {
            return Err(b.syntax("CT takes no DR and no R; the tangent sets the arc"));
        }
        let t = self.tangent.ok_or_else(|| {
            b.invalid_arc("CT needs a previous contour element with an XY direction")
        })?;
        self.run_arc(b, &w, |m, to| {
            let d = Point2::new(to.x - m.pos.x, to.y - m.pos.y);
            let n = Point2::new(-t.y, t.x);
            let k = n.x * d.x + n.y * d.y;
            if k.abs() <= 1e-9 * (1.0 + d.x.hypot(d.y)) {
                return Err(
                    b.invalid_arc("CT end point lies on the tangent line; no arc is possible")
                );
            }
            let r = (d.x * d.x + d.y * d.y) / (2.0 * k);
            let c = Point2::new(m.pos.x + n.x * r, m.pos.y + n.y * r);
            check_len(b.line, "CT centre X", c.x)?;
            check_len(b.line, "CT centre Y", c.y)?;
            let ccw = k > 0.0;
            resolve_arc(
                b.line,
                m.pos,
                to,
                ArcPlane::Xy,
                ccw,
                ArcCenterMode::Absolute,
                [Some(c.x), Some(c.y), None],
                None,
                m.opts.arc_radius_tolerance_mm,
            )
            .map(|res| (res, ccw))
        })
    }

    /// Shared arc tail: M functions, feed, end point, resolution and the moves.
    fn run_arc<F>(&mut self, b: &Block, w: &Words, resolve: F) -> Result<(), GcodeError>
    where
        F: Fn(&Self, Point3) -> Result<(crate::arc::ResolvedArc, bool), GcodeError>,
    {
        if w.mb.is_some() {
            return Err(b.syntax("MB on an arc block"));
        }
        if w.feed == Some(Feed::Max) {
            return Err(b.invalid_arc("FMAX on an arc is not read; give a feed rate F"));
        }
        self.pre_m(b, &w.m)?;
        let feed = self
            .feed_words(b, w)?
            .ok_or_else(|| b.invalid_modal("an arc needs a feed rate F"))?;
        let to = self.target(b, w)?;
        if (to.z - self.pos.z).abs() > 1e-9 {
            return Err(b.unsupported(
                "helix",
                "an arc with a Z change is not read in this reader version",
            ));
        }
        let (res, ccw) = resolve(self, to)?;
        let c = res.center_words;
        if let Some(mid) = res.split_mid {
            check_point(b.line, "arc mid point", mid)?;
            self.current
                .push_arc(b.line, mid, feed, c, ccw, ArcPlane::Xy);
        }
        self.current
            .push_arc(b.line, to, feed, c, ccw, ArcPlane::Xy);
        let rx = to.x - c.x;
        let ry = to.y - c.y;
        let len = rx.hypot(ry);
        self.tangent = (len > 1e-12).then(|| {
            if ccw {
                Point2::new(-ry / len, rx / len)
            } else {
                Point2::new(ry / len, -rx / len)
            }
        });
        self.pos = to;
        self.post_m(b, &w.m, None)
    }

    /// Apply an `F` word to the modal feed and return the active feed rate.
    fn feed_words(&mut self, b: &Block, w: &Words) -> Result<Option<f64>, GcodeError> {
        if let Some(Feed::Rate(f)) = w.feed {
            self.modal.feed_rate_mm_min = Some(check_feed(b.line, f)?);
        }
        Ok(self.modal.feed_rate_mm_min)
    }

    /// End point of a path block; an axis that is not written keeps its position.
    pub fn target(&self, b: &Block, w: &Words) -> Result<Point3, GcodeError> {
        let cur = [self.pos.x, self.pos.y, self.pos.z];
        let mut v = [0.0; 3];
        for k in 0..3 {
            v[k] = match (w.abs[k], w.inc[k]) {
                (Some(a), _) => a,
                (None, Some(d)) => cur[k] + d,
                (None, None) => cur[k],
            };
        }
        check_point(b.line, "end point", Point3::new(v[0], v[1], v[2]))
    }

    pub fn push_line(&mut self, b: &Block, kind: MoveKind, to: Point3, feed: Option<f64>) {
        let dx = to.x - self.pos.x;
        let dy = to.y - self.pos.y;
        let len = dx.hypot(dy);
        if len > 1e-9 {
            self.tangent = Some(Point2::new(dx / len, dy / len));
        } else if (to.z - self.pos.z).abs() > 1e-9 {
            self.tangent = None;
        }
        self.current.push_move(b.line, kind, to, feed);
        self.pos = to;
    }

    /// `M140 MB MAX` retracts to the configured traverse end; `M140 MB n` by n mm.
    pub fn m140(&mut self, b: &Block, mb: Option<Mb>) -> Result<(), GcodeError> {
        let z =
            match mb {
                Some(Mb::Max) => self.opts.tool_axis_max_z.ok_or_else(|| {
                    GcodeError::MissingReferencePosition {
                        line: b.line,
                        code: format!("{}M140 MB MAX (KlartextOptions::tool_axis_max_z)", b.tag()),
                    }
                })?,
                Some(Mb::Dist(d)) if d >= 0.0 => check_len(b.line, "M140 MB", self.pos.z + d)?,
                Some(Mb::Dist(d)) => {
                    return Err(
                        b.unsupported("M140 MB", &format!("a negative retract ({d}) is not read"))
                    );
                }
                None => return Err(b.syntax("M140 needs MB MAX or MB <distance>")),
            };
        if z < self.pos.z {
            return Err(b.unsupported(
                "M140 MB MAX",
                &format!(
                    "the traverse end Z {z} is below the tool position Z {}",
                    self.pos.z
                ),
            ));
        }
        let to = Point3::new(self.pos.x, self.pos.y, z);
        self.sync_section();
        self.push_line(b, MoveKind::Rapid, to, None);
        Ok(())
    }
}

fn reject_arc_words(b: &Block, w: &Words, what: &str) -> Result<(), GcodeError> {
    if w.ccw.is_some() || w.radius.is_some() {
        return Err(b.syntax(&format!("{what} takes no DR and no R radius")));
    }
    Ok(())
}
