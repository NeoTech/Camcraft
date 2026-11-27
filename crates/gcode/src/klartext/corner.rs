//! Klartext transition elements `RND` (rounding arc) and `CHF` (chamfer) between two straight lines in the working plane: the lines are cut back and the arc or chamfer joins them.
//! Provenance: HEIDENHAIN iTNC 530 conversational user's manual 533 190-22 (path functions RND and CHF: the corner point is not machined, the rounding arc is tangent to both elements, the chamfer side length is cut from each line, the F word acts only in the RND/CHF block), as summarized in docs/research/heidenhain/differences.md; tangent-length formula t = R tan(phi/2) from plane geometry.

use camcraft_cl::cl::{ArcPlane, MoveKind};
use camcraft_geometry::geo2d::Point2;
use camcraft_geometry::geometry::Point3;

use super::exec::Machine;
use super::source::Block;
use super::words::{Words, value};
use crate::domain::{check_feed, check_len, check_point};
use crate::types::GcodeError;

/// A straight feed line in the working plane whose end is the corner of the
/// next `RND` / `CHF` block. It is not in the toolpath until that block and
/// the line after it set the cut-back point.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingLine {
    /// Physical line of the block that programmed the line.
    pub line: usize,
    /// Start of the line, after a cut-back by an earlier `RND` / `CHF`.
    pub start: Point3,
    /// Programmed end point: the corner.
    pub corner: Point3,
    /// Unit XY direction of the programmed line.
    pub dir: Point2,
    pub feed: f64,
}

/// The transition element of an `RND` / `CHF` block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CornerKind {
    /// `RND R..`: rounding arc of this radius.
    Round(f64),
    /// `CHF ..`: chamfer with this side length on each line.
    Chamfer(f64),
}

/// An `RND` / `CHF` block that waits for the line after it.
#[derive(Clone, Debug, PartialEq)]
pub struct Corner {
    pub kind: CornerKind,
    /// Feed of the transition element: the `F` of the block, else the modal feed.
    pub feed: f64,
    pub block: Block,
}

impl CornerKind {
    fn name(self) -> &'static str {
        match self {
            CornerKind::Round(_) => "RND",
            CornerKind::Chamfer(_) => "CHF",
        }
    }
}

impl Machine<'_> {
    fn next_is_corner(&self) -> Option<&'static str> {
        match self.next_word.as_deref() {
            Some("RND") => Some("RND"),
            Some("CHF") => Some("CHF"),
            _ => None,
        }
    }

    /// The name of the `RND` / `CHF` that touches the current straight-line block.
    fn corner_name(&self) -> Option<&'static str> {
        self.corner
            .as_ref()
            .map(|c| c.kind.name())
            .or_else(|| self.next_is_corner())
    }

    /// `RND R.. [F..]` and `CHF <length> [F..]`.
    pub fn corner_block(&mut self, b: &Block, toks: &[String]) -> Result<(), GcodeError> {
        let name = if b.words.first().map(String::as_str) == Some("RND") {
            "RND"
        } else {
            "CHF"
        };
        let (mut size, mut feed) = (None, None);
        for (i, t) in toks.iter().enumerate() {
            if let Some(v) = t.strip_prefix('F') {
                if v == "MAX" {
                    return Err(b.unsupported(name, "FMAX on a transition element is not read"));
                }
                if feed.replace(check_feed(b.line, value(b, "F", v, &self.q)?)?).is_some() {
                    return Err(b.syntax("F appears twice"));
                }
                continue;
            }
            let v = match (name, t.strip_prefix('R')) {
                ("RND", Some(v)) => v,
                ("CHF", None) if i == 0 => t.as_str(),
                _ => return Err(b.unknown(t)),
            };
            if size.replace(value(b, name, v, &self.q)?).is_some() {
                return Err(b.syntax(&format!("{name} has two sizes")));
            }
        }
        let size = size.ok_or_else(|| b.syntax(&format!("{name} needs its size")))?;
        let size = check_len(b.line, name, size)?;
        if size <= 1e-6 {
            return Err(b.syntax(&format!("{name} size ({size}) must be greater than 0")));
        }
        if self.corner.is_some() || self.pending.is_none() {
            return Err(b.unsupported(
                name,
                "a transition element needs a straight line (L or LP) in the working plane before it; next to an arc, a cycle or at the contour start it is not read in this reader version",
            ));
        }
        if !matches!(self.next_word.as_deref(), Some("L" | "LP")) {
            return Err(b.unsupported(
                name,
                "a transition element needs a straight line (L or LP) in the working plane after it; next to an arc or another block it is not read in this reader version",
            ));
        }
        let feed = match feed.or(self.modal.feed_rate_mm_min) {
            Some(f) => f,
            None => return Err(b.invalid_modal(&format!("{name} needs a feed rate F"))),
        };
        let kind = if name == "RND" {
            CornerKind::Round(size)
        } else {
            CornerKind::Chamfer(size)
        };
        self.corner = Some(Corner {
            kind,
            feed,
            block: b.clone(),
        });
        Ok(())
    }

    /// Word checks for a straight-line block next to an `RND` / `CHF`: the line
    /// must move, the line after a transition takes no M function, and the line
    /// before it takes only M functions that act at the block start.
    pub fn corner_words_check(
        &self,
        b: &Block,
        w: &Words,
        moves: bool,
    ) -> Result<(), GcodeError> {
        let Some(name) = self.corner_name() else {
            return Ok(());
        };
        if !moves {
            return Err(b.unsupported(name, "the straight line next to it has no end point"));
        }
        if self.corner.is_some() && (!w.m.is_empty() || w.mb.is_some()) {
            return Err(b.unsupported(
                name,
                "an M function in the line after a transition element is not read",
            ));
        }
        if self.next_is_corner().is_some()
            && (w.mb.is_some() || w.m.iter().any(|n| !matches!(n, 3 | 4 | 8 | 13 | 14)))
        {
            return Err(b.unsupported(
                name,
                "the line before a transition element takes only M3, M4, M8, M13 or M14 (an M function at the block end is not read there)",
            ));
        }
        Ok(())
    }

    /// A straight line of an `L` / `LP` block, with the `RND` / `CHF` before and
    /// after it: the line before a transition waits; the line after it sets the
    /// cut-back points, and the toolpath gets the shortened line, the
    /// transition element and the start of this line.
    pub fn line_to(
        &mut self,
        b: &Block,
        kind: MoveKind,
        to: Point3,
        feed: Option<f64>,
    ) -> Result<(), GcodeError> {
        let Some(name) = self.corner_name() else {
            self.push_line(b, kind, to, feed);
            return Ok(());
        };
        let from = self.pos;
        let (dx, dy) = (to.x - from.x, to.y - from.y);
        let len = dx.hypot(dy);
        let feed = match (kind, feed) {
            (MoveKind::Feed, Some(f)) => f,
            _ => return Err(b.unsupported(name, "the straight line next to it must be a feed move, not FMAX")),
        };
        if (to.z - from.z).abs() > 1e-9 {
            return Err(b.unsupported(name, "the straight line next to it must stay in the working plane (no Z change)"));
        }
        if len <= 1e-6 {
            return Err(b.unsupported(name, "the straight line next to it has no length in the working plane"));
        }
        let mut start = from;
        if let Some(c) = self.corner.take() {
            let p = self.pending.take().ok_or_else(|| {
                c.block
                    .unsupported(c.kind.name(), "no straight line waits before this block")
            })?;
            let v = Point2::new(dx / len, dy / len);
            let (t1, t2) = corner_points(&c, &p, v, len, b.line)?;
            check_point(c.block.line, "transition start", t1)?;
            check_point(c.block.line, "transition end", t2)?;
            self.current
                .push_move(p.line, MoveKind::Feed, t1, Some(p.feed));
            match c.kind {
                CornerKind::Round(r) => {
                    let (center, ccw) = round_center(p.dir, v, t1, r);
                    self.current
                        .push_arc(c.block.line, t2, c.feed, center, ccw, ArcPlane::Xy);
                }
                CornerKind::Chamfer(_) => {
                    self.current
                        .push_move(c.block.line, MoveKind::Feed, t2, Some(c.feed));
                }
            }
            start = t2;
        }
        self.tangent = Some(Point2::new(dx / len, dy / len));
        if self.next_is_corner().is_some() {
            self.pending = Some(PendingLine {
                line: b.line,
                start,
                corner: to,
                dir: Point2::new(dx / len, dy / len),
                feed,
            });
        } else {
            self.current.push_move(b.line, MoveKind::Feed, to, Some(feed));
        }
        self.pos = to;
        Ok(())
    }
}

/// The cut-back points on the line before (`t1`) and after (`t2`) the corner;
/// `v` and `len_out` are the unit direction and the length of the line after,
/// which starts on physical line `after`.
fn corner_points(
    c: &Corner,
    p: &PendingLine,
    v: Point2,
    len_out: f64,
    after: usize,
) -> Result<(Point3, Point3), GcodeError> {
    let name = c.kind.name();
    let u = p.dir;
    // The line before can be cut back already by an earlier transition.
    let len_in = (p.corner.x - p.start.x).hypot(p.corner.y - p.start.y);
    let cross = u.x * v.y - u.y * v.x;
    let dot = u.x * v.x + u.y * v.y;
    if cross.abs() <= 1e-9 {
        return Err(c.block.unsupported(
            name,
            "the lines before and after it are in line (no corner to round or chamfer)",
        ));
    }
    let dist = match c.kind {
        CornerKind::Round(r) => r * (cross.abs().atan2(dot) / 2.0).tan(),
        CornerKind::Chamfer(l) => l,
    };
    let room = len_in.min(len_out);
    if !dist.is_finite() || dist > room + 1e-9 {
        return Err(c.block.unsupported(
            name,
            &format!(
                "it needs {dist:.4} mm of each line, but the line before has {len_in:.4} mm and the line after (line {after}) has {len_out:.4} mm"
            ),
        ));
    }
    let z = p.corner.z;
    let t1 = Point3::new(p.corner.x - u.x * dist, p.corner.y - u.y * dist, z);
    let t2 = Point3::new(p.corner.x + v.x * dist, p.corner.y + v.y * dist, z);
    Ok((t1, t2))
}

/// Centre and direction of the rounding arc of radius `r` that starts at `t1`
/// on the line before the corner (direction `u`; the line after has `v`): on
/// the inner side of the turn.
fn round_center(u: Point2, v: Point2, t1: Point3, r: f64) -> (Point2, bool) {
    let ccw = u.x * v.y - u.y * v.x > 0.0;
    let n = if ccw {
        Point2::new(-u.y, u.x)
    } else {
        Point2::new(u.y, -u.x)
    };
    (Point2::new(t1.x + n.x * r, t1.y + n.y * r), ccw)
}
