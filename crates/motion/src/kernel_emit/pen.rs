//! The pen of the converter: the tool's place and state between two steps,
//! and the one move rule for each step kind (see the module doc of
//! [`super`]).
//! Provenance: our own design record `docs/decisions/4269-kernel-plan-converter.md`;
//! the retract-traverse-plunge order of general machining practice.

use super::arcs::arc_runs;
use super::{
    APPROACH_IN_MATERIAL, AXIS_NOT_NORMAL, CUT_DROPPED, ENDS_DOWN, ENTRY_NOT_CLEAR, EmitFeeds, EmitHeights, EmitRefusal,
    Emitted, NOT_FINITE, NOT_SETUP_XY, STEP_NOT_DOWN, STEP_OFF_TOOL,
};
use crate::cl::{MoveKind, PathBuilder};
use crate::geo2d::Point2;
use crate::geometry::Point3;
use crate::links::{CLEARANCE_PLANE, LinkTally};
use camcraft_pathkernel::entry::{Entry, Exit};
use camcraft_pathkernel::link::Hop;
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::{PlanePoint, WorkPlane};

/// The tolerance (mm) of "where the tool stands" and of the plane checks:
/// ten times the kernel tolerance, for the rounding of the mapping to XYZ.
pub(super) const EMIT_TOL: f64 = 1e-5;

/// The word of the lift notes: "the keep-down in-plane link".
const LINK_WORD: &str = "in-plane";

/// Where the tool is between two steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// At or above the clearance height (the start, after a lift or an exit).
    Clear,
    /// At depth, at the end of a proved step.
    Down,
}

/// The setup-XY frame of a level: the Z of its origin.
#[derive(Clone, Copy, Debug)]
struct Frame {
    z0: f64,
}

impl Frame {
    fn of(plane: WorkPlane) -> Result<Self, EmitRefusal> {
        let near = |a: [f64; 3], b: [f64; 3]| a.iter().zip(&b).all(|(p, q)| (p - q).abs() <= EMIT_TOL);
        match plane {
            WorkPlane::Planar(f) => {
                let o = f.origin();
                if near(f.normal(), [0.0, 0.0, 1.0]) && near(f.x_dir(), [1.0, 0.0, 0.0]) && near([o[0], o[1], 0.0], [0.0; 3]) {
                    Ok(Self { z0: o[2] })
                } else {
                    Err(EmitRefusal::new(NOT_SETUP_XY))
                }
            }
            _ => Err(EmitRefusal::new(NOT_SETUP_XY)),
        }
    }
    /// The XYZ of a plane point: its place, and its depth along +Z above the origin.
    fn xyz(self, p: &PlanePoint) -> Result<Point3, EmitRefusal> {
        let q = Point3::new(p.at().x, p.at().y, self.z0 + p.depth());
        if q.x.is_finite() && q.y.is_finite() && q.z.is_finite() { Ok(q) } else { Err(EmitRefusal::new(NOT_FINITE)) }
    }
}

pub(super) struct Pen {
    h: EmitHeights,
    f: EmitFeeds,
    frame: Option<Frame>,
    b: Option<PathBuilder>,
    state: State,
    tally: LinkTally,
    out: Emitted,
}

fn near3(a: Point3, b: Point3) -> bool {
    (a.x - b.x).abs() <= EMIT_TOL && (a.y - b.y).abs() <= EMIT_TOL && (a.z - b.z).abs() <= EMIT_TOL
}

impl Pen {
    pub(super) fn new(h: EmitHeights, f: EmitFeeds) -> Self {
        Self { h, f, frame: None, b: None, state: State::Clear, tally: LinkTally::default(), out: Emitted::default() }
    }

    pub(super) fn set_plane(&mut self, plane: WorkPlane) -> Result<(), EmitRefusal> {
        self.frame = Some(Frame::of(plane)?);
        Ok(())
    }

    fn frame(&self) -> Result<Frame, EmitRefusal> {
        self.frame.ok_or(EmitRefusal::new(NOT_SETUP_XY))
    }

    fn xyz(&self, p: &PlanePoint) -> Result<Point3, EmitRefusal> {
        self.frame()?.xyz(p)
    }

    /// The builder, made at the clearance height over `over` at the first move.
    fn builder(&mut self, over: Point3) -> &mut PathBuilder {
        let z = self.h.clearance_z();
        self.b.get_or_insert_with(|| PathBuilder::new(Point3::new(over.x, over.y, z)))
    }

    /// Where the tool stands (none before the first move).
    fn at(&self) -> Option<Point3> {
        self.b.as_ref().map(PathBuilder::at)
    }

    fn expect_down(&self, start: Point3) -> Result<(), EmitRefusal> {
        if self.state != State::Down {
            return Err(EmitRefusal::new(STEP_NOT_DOWN));
        }
        match self.at() {
            Some(at) if near3(at, start) => Ok(()),
            _ => Err(EmitRefusal::new(STEP_OFF_TOOL)),
        }
    }

    /// A move of the air part of an entry: a rapid when both ends are at or
    /// above the clearance height, else a plunge-feed move down or a feed.
    fn air_move(&mut self, to: Point3) {
        let clear = self.h.clearance_z() - EMIT_TOL;
        let f = self.f;
        let b = self.builder(to);
        let from = b.at();
        if from.z >= clear && to.z >= clear {
            b.push(MoveKind::Rapid, to, None);
        } else if to.z < from.z {
            b.push(MoveKind::Plunge, to, Some(f.plunge()));
        } else {
            b.push(MoveKind::Feed, to, Some(f.cut()));
        }
    }

    fn feed(&mut self, kind: MoveKind, to: Point3) {
        let rate = if kind == MoveKind::Plunge { self.f.plunge() } else { self.f.cut() };
        self.builder(to).push(kind, to, Some(rate));
    }

    /// Straight up along the tool axis to `z` (at least the clearance height).
    fn retract(&mut self, z: f64) {
        let z = z.max(self.h.clearance_z());
        if let Some(b) = self.b.as_mut() {
            let at = b.at();
            if at.z < z {
                b.push(MoveKind::Rapid, Point3::new(at.x, at.y, z), None);
            }
        }
        self.state = State::Clear;
    }

    /// An entry: a rapid at the clearance height over the first point, the
    /// move down to it, the approach (at or above the stock top), the
    /// proved descent at the plunge feed, and the lead at the cut feed.
    pub(super) fn entry(&mut self, e: &Entry) -> Result<(), EmitRefusal> {
        if self.state != State::Clear {
            return Err(EmitRefusal::new(ENTRY_NOT_CLEAR));
        }
        let approach = e.approach().iter().map(|p| self.xyz(p)).collect::<Result<Vec<_>, _>>()?;
        if approach.iter().any(|p| p.z < self.h.top_z() - EMIT_TOL) {
            return Err(EmitRefusal::new(APPROACH_IN_MATERIAL));
        }
        let descent = e.descent().iter().map(|p| self.xyz(p)).collect::<Result<Vec<_>, _>>()?;
        let lead = e.lead().map_or(Ok(Vec::new()), |l| l.pts().iter().map(|p| self.xyz(p)).collect::<Result<Vec<_>, _>>())?;
        let Some(&first) = approach.first().or(descent.first()) else {
            return Err(EmitRefusal::new(STEP_OFF_TOOL));
        };
        let clear = self.h.clearance_z();
        let at_z = self.at().map_or(clear, |a| a.z).max(clear);
        self.builder(first).push(MoveKind::Rapid, Point3::new(first.x, first.y, at_z), None);
        for p in &approach {
            self.air_move(*p);
        }
        for p in &descent {
            self.feed(MoveKind::Plunge, *p);
        }
        // The lead starts where the descent ends (`Entry::led` checks it).
        for p in lead.iter().skip(1) {
            self.feed(MoveKind::Feed, *p);
        }
        self.state = State::Down;
        self.out.entries = self.out.entries.saturating_add(1);
        Ok(())
    }

    /// A cut: from where the tool stands, along the curve (closed curves
    /// back to their start), as arcs where the curve holds an exact circle.
    pub(super) fn cut(&mut self, c: &Cut) -> Result<(), EmitRefusal> {
        if c.axis().is_some_and(|a| !near3(Point3::new(a[0], a[1], a[2]), Point3::new(0.0, 0.0, 1.0))) {
            return Err(EmitRefusal::new(AXIS_NOT_NORMAL));
        }
        let curve = c.curve();
        let mut pts = curve.pts().iter().map(|p| self.xyz(p)).collect::<Result<Vec<_>, _>>()?;
        let Some(&start) = pts.first() else {
            return Err(EmitRefusal::new(CUT_DROPPED));
        };
        self.expect_down(start)?;
        if curve.closed() {
            pts.push(start);
        }
        let begin = self.b.as_ref().map_or(0, PathBuilder::moves_len);
        let rate = self.f.cut();
        match curve.arc() {
            Some(circle) => {
                let runs = arc_runs(&pts, circle).map_err(EmitRefusal::new)?;
                let b = self.builder(start);
                for (to, ccw) in runs {
                    b.push_arc(MoveKind::Feed, to, Some(rate), circle.center(), ccw);
                }
            }
            None => {
                for p in pts.iter().skip(1) {
                    self.feed(MoveKind::Feed, *p);
                }
            }
        }
        let end = self.b.as_ref().map_or(0, PathBuilder::moves_len);
        if end <= begin {
            return Err(EmitRefusal::new(CUT_DROPPED));
        }
        self.out.cut_moves.push(begin..end);
        Ok(())
    }

    /// A link: a stay-down hop is fed along its proved points at the depth
    /// where the tool stands; a lift is a retract to the clearance height,
    /// counted by its named reason for the notes.
    pub(super) fn link(&mut self, hop: &Hop) -> Result<(), EmitRefusal> {
        match hop {
            Hop::Stay(path) => {
                let pts = path.points();
                let at = self.at().ok_or(EmitRefusal::new(STEP_NOT_DOWN))?;
                let to3 = |q: Point2| Point3::new(q.x, q.y, at.z);
                let Some(&first) = pts.first() else {
                    return Err(EmitRefusal::new(STEP_OFF_TOOL));
                };
                self.expect_down(to3(first))?;
                for q in pts.iter().skip(1) {
                    self.feed(MoveKind::Feed, to3(*q));
                }
                self.out.stays = self.out.stays.saturating_add(1);
            }
            Hop::Lift(reason) => {
                let at = self.at().ok_or(EmitRefusal::new(STEP_NOT_DOWN))?;
                if self.state != State::Down {
                    return Err(EmitRefusal::new(STEP_NOT_DOWN));
                }
                self.tally.lift_to(reason.get(), CLEARANCE_PLANE, Point2::new(at.x, at.y), at.z);
                self.retract(self.h.clearance_z());
                self.out.lifts = self.out.lifts.saturating_add(1);
            }
        }
        Ok(())
    }

    /// An exit: the proved lead-out at the cut feed, then the retract to the
    /// exit's height or the clearance height, whichever is higher.
    pub(super) fn exit(&mut self, x: &Exit) -> Result<(), EmitRefusal> {
        let at = self.at().ok_or(EmitRefusal::new(STEP_NOT_DOWN))?;
        if let Some(lead) = x.lead_out() {
            let pts = lead.iter().map(|p| self.xyz(p)).collect::<Result<Vec<_>, _>>()?;
            self.expect_down(pts.first().copied().unwrap_or(at))?;
            for p in pts.iter().skip(1) {
                self.feed(MoveKind::Feed, *p);
            }
        } else if self.state != State::Down {
            return Err(EmitRefusal::new(STEP_NOT_DOWN));
        }
        let z = self.frame()?.z0 + x.retract();
        if !z.is_finite() {
            return Err(EmitRefusal::new(NOT_FINITE));
        }
        self.retract(z);
        self.out.exits = self.out.exits.saturating_add(1);
        Ok(())
    }

    pub(super) fn end_level(&mut self) -> Result<(), EmitRefusal> {
        if self.state == State::Clear { Ok(()) } else { Err(EmitRefusal::new(ENDS_DOWN)) }
    }

    /// The toolpath, with one note for each lift reason.
    pub(super) fn finish(mut self) -> Emitted {
        if let Some(mut b) = self.b.take() {
            self.tally.say(&mut b, LINK_WORD);
            self.out.path = b.finish();
        }
        self.out
    }
}
