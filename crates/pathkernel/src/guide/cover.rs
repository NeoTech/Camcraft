//! The shared parts of the guide planners (#4176): the checked inputs, the
//! raw curves before they are checked, and the completeness guard of the
//! record (#3744: the swept discs cover the cutter-centre area up to
//! 0.1 mm2, else the result is incomplete with each gap named).
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 2: Guide curves"; the swept area of a disc as the Minkowski sum of
//! a path and the disc (the published polygon-offsetting literature).

use super::{Circle, Curve, Family, GuideSet};
use crate::frame::PlanePoint;
use crate::guide::mark::Mark;
use crate::region::{KernelRegion, PlanarRegion};
use crate::verdict::{Gap, Note, Refusal, Stage, Verdict};
use crate::{KERNEL_TOL, Loop, Point2, Region};

/// Refusal reason: a guide tolerance that is not a finite number at or
/// above the kernel tolerance.
pub const BAD_TOL: &str = "the guide tolerance must be a finite number at or above the kernel tolerance";
/// Refusal reason: a region that is not a planar area.
pub const NOT_PLANAR: &str = "this guide planner plans only a planar region";
/// Refusal reason: an area planner given the chains of a trace.
pub const NOT_AN_AREA: &str = "this guide planner plans an area: the chains of a trace have no area";
/// Refusal reason: a polygon operation that cannot be computed.
pub const GUIDE_GEOMETRY: &str = "the guide geometry cannot be computed: a value is out of range";
/// Refusal reason: a stepover larger than the tool diameter.
pub const STEP_ABOVE_DIAMETER: &str = "the stepover is larger than the tool diameter: material between two curves stays uncut";

/// The largest flattening error (mm) of a round join of the polygon
/// offset of `camcraft-geometry` (5 um): a swept disc can fall short of the
/// true disc by this much.
pub(crate) const ARC_SAG: f64 = 5e-3;
/// The total uncovered area (mm2) that the completeness guard accepts
/// (record "Stage 2": 0.1 mm2).
pub(crate) const GUARD_AREA: f64 = 0.1;
/// The area (mm2) under which an uncovered piece is polygon noise of the
/// micron lattice, not a gap (one square micron).
pub(crate) const NOISE_AREA: f64 = 1e-6;

/// A curve before its checks: plane points at depth 0 and its index.
#[derive(Clone, Debug)]
pub(crate) struct Raw {
    pub pts: Vec<Point2>,
    pub closed: bool,
    pub k: u32,
}

impl Raw {
    /// The walk of the curve as an open path (a closed curve repeats its
    /// first point at the end), for a sweep or a clip.
    pub fn walk(&self) -> Vec<Point2> {
        let mut w = self.pts.clone();
        if self.closed
            && let Some(&f) = self.pts.first()
        {
            w.push(f);
        }
        w
    }
}

/// The planar region of `r`, or the refusal.
pub(crate) fn planar(r: &KernelRegion) -> Result<&PlanarRegion, Refusal> {
    match r {
        KernelRegion::Planar(p) => Ok(p),
        KernelRegion::Chains(_) => Err(Refusal::new(Stage::Guide, NOT_AN_AREA)),
    }
}

/// The guide tolerance, checked.
pub(crate) fn check_tol(tol: f64) -> Result<f64, Refusal> {
    if tol.is_finite() && tol >= KERNEL_TOL { Ok(tol) } else { Err(Refusal::new(Stage::Guide, BAD_TOL)) }
}

/// The stepover, checked against the tool diameter.
pub(crate) fn check_step(step: f64, tool_r: f64) -> Result<f64, Refusal> {
    let limit = 2.0 * tool_r + KERNEL_TOL;
    if step > limit { Err(Refusal::new(Stage::Guide, STEP_ABOVE_DIAMETER).fix("use a stepover of the tool diameter or less")) } else { Ok(step) }
}

/// A geometry result as a guide refusal.
pub(crate) fn geo<T, E>(r: Result<T, E>) -> Result<T, Refusal> {
    r.map_err(|_| Refusal::new(Stage::Guide, GUIDE_GEOMETRY))
}

/// The closed walks of loops, for a sweep.
pub(crate) fn loop_walks(loops: &[Loop]) -> Vec<Vec<Point2>> {
    loops.iter().map(|l| l.points().iter().chain(l.points().first()).copied().collect()).collect()
}

/// The area of `target` that the discs of radius `r` along `curves` do not
/// cover, with `margin` for the polygon flattening.
pub(crate) fn uncovered(target: &Region, curves: &[Raw], r: f64, margin: f64) -> Result<Region, Refusal> {
    let walks: Vec<Vec<Point2>> = curves.iter().map(Raw::walk).collect();
    geo(target.difference(&geo(Region::sweep(&walks, r + margin))?))
}

/// The pieces of a region: each outer loop with the holes inside it, as
/// its own region, with its area (mm2), largest first; then by place.
pub(crate) fn pieces(r: &Region) -> Result<Vec<(Region, f64)>, Refusal> {
    let (outer, holes): (Vec<&Loop>, Vec<&Loop>) = r.loops().iter().partition(|l| l.area() > 0.0);
    let mut out = Vec::with_capacity(outer.len());
    for o in outer {
        let shell = geo(Region::new(vec![o.clone()]))?;
        let mine: Vec<Loop> = holes.iter().filter(|h| h.points().first().is_some_and(|p| shell.contains(*p))).map(|h| (*h).clone()).collect();
        let mut loops = vec![o.clone()];
        loops.extend(mine);
        let piece = geo(Region::new(loops))?;
        let area = piece.area();
        out.push((piece, area));
    }
    out.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| low(&a.0).total_cmp(&low(&b.0))));
    Ok(out)
}

/// A sort key of a region: the least y of its points.
fn low(r: &Region) -> f64 {
    r.loops().iter().flat_map(|l| l.points()).map(|p| p.y).fold(f64::INFINITY, f64::min)
}

/// The lowest point of a region (least y, then x): the place of a gap.
pub(crate) fn lowest(r: &Region) -> Point2 {
    r.loops().iter().flat_map(|l| l.points()).copied().min_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x))).unwrap_or_default()
}

/// The checked curves, their marks and the notes of [`finish`].
pub(crate) type Finished = (Vec<Curve>, Vec<Mark>, Vec<Note>);

/// The checked curves of the raw curves, with each mark moved to the index
/// of its curve. A raw curve that the curve checks refuse (a piece below the
/// kernel tolerance) is named in a note, never dropped silently.
pub(crate) fn finish(raws: &[Raw], marks: Vec<(usize, Mark)>, family: impl Fn(&Raw) -> Family) -> Result<Finished, Refusal> {
    finish_arcs(raws, &[], marks, family)
}

/// [`finish`] with the exact circle of raw curve `i` at `arcs[i]` (none
/// where `arcs` is shorter).
pub(crate) fn finish_arcs(raws: &[Raw], arcs: &[Option<Circle>], marks: Vec<(usize, Mark)>, family: impl Fn(&Raw) -> Family) -> Result<Finished, Refusal> {
    let mut curves = Vec::with_capacity(raws.len());
    let mut index = vec![None; raws.len()];
    let mut notes = Vec::new();
    for (i, raw) in raws.iter().enumerate() {
        let pts: Result<Vec<PlanePoint>, Refusal> = raw.pts.iter().map(|p| PlanePoint::flat(*p)).collect();
        match Curve::new(pts?, arcs.get(i).copied().flatten(), raw.closed, raw.k, family(raw)) {
            Ok(c) => {
                index[i] = Some(curves.len());
                curves.push(c);
            }
            Err(e) => {
                // Nothing that a planner made is dropped without a note:
                // the note names the curve and each mark on it.
                let at = raw.pts.first().copied().unwrap_or_default();
                let lost = marks.iter().filter(|(j, _)| *j == i).count();
                let text = format!("a piece of curve {} at ({:.3}, {:.3}) is not a curve ({}); {lost} mark(s) on it are not kept", raw.k, at.x, at.y, e.reason());
                notes.push(Note::new(Stage::Guide, text)?);
            }
        }
    }
    let marks = marks.into_iter().filter_map(|(i, m)| index.get(i).copied().flatten().map(|j| m.renumber(j))).collect();
    Ok((curves, marks, notes))
}

/// What a guide choice asks of the area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Ask {
    /// Clear the area: more than [`GUARD_AREA`] uncut makes the verdict
    /// incomplete.
    Clear,
    /// A fixed count of rings (Contour): the uncut area is named in a note.
    Count(u32),
    /// A line that does not clear the area (the medial line): the uncut
    /// area is named in a note.
    Line,
}

/// The verdict of a guide set by the completeness guard (#4176 review: one
/// guard for each planner). The guard measures the WHOLE cutter-centre
/// area of the region (`p.center()`), never an area that a planner derives
/// from its own parameters: what the tool discs along `raws` leave uncut of
/// it decides. For [`Ask::Clear`]: done when at most [`GUARD_AREA`] stays
/// uncut (a remnant above the noise is named in a note); else incomplete
/// with each gap named. For [`Ask::Count`]: done, with the uncut area named.
/// The discs are read along the curves of the finished set, so a curve that
/// the curve checks refused covers nothing.
pub(crate) fn judge(p: &PlanarRegion, set: GuideSet, mut notes: Vec<Note>, ask: Ask, margin: f64) -> Result<Verdict<GuideSet>, Refusal> {
    let cut: Vec<Raw> = set.curves().iter().map(|c| Raw { pts: c.pts().iter().map(PlanePoint::at).collect(), closed: c.closed(), k: c.k() }).collect();
    let left = uncovered(p.center(), &cut, p.tool().radius(), margin)?;
    let left = &left;
    let gaps: Vec<(Region, f64)> = pieces(left)?.into_iter().filter(|(_, a)| *a > NOISE_AREA).collect();
    let total: f64 = gaps.iter().map(|(_, a)| a).sum();
    let named = match ask {
        Ask::Count(n) => Some(format!("a count of {n} ring(s) leaves {total:.3} mm2 of the area uncut, as the choice asks")),
        Ask::Line => Some(format!("the tool along the line(s) leaves {total:.3} mm2 of the area uncut: the guide does not clear the area")),
        Ask::Clear => None,
    };
    if let Some(text) = named {
        if total > 0.0 {
            notes.push(Note::new(Stage::Guide, text)?);
        }
        return Ok(Verdict::Done { out: set, fallbacks: Vec::new(), notes });
    }
    if total > GUARD_AREA {
        let list: Result<Vec<Gap>, Refusal> = gaps.iter().map(|(g, a)| Gap::new(*a, lowest(g))).collect();
        let mut v = Verdict::incomplete(Stage::Guide, set, list?)?;
        if let Verdict::Incomplete { notes: n, .. } = &mut v {
            n.append(&mut notes);
        }
        return Ok(v);
    }
    if total > 0.0 {
        notes.push(Note::new(Stage::Guide, format!("{} uncovered piece(s) of {total:.4} mm2 in all stay under the 0.1 mm2 guard", gaps.len()))?);
    }
    Ok(Verdict::Done { out: set, fallbacks: Vec::new(), notes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guide::mark::MarkKind;

    #[test]
    fn a_refused_curve_names_its_marks_in_its_note() {
        // A closed curve that repeats its first point is refused by the
        // curve checks; its mark is not kept, and the note says so.
        let p = |x, y| Point2::new(x, y);
        let bad = Raw { pts: vec![p(0.0, 0.0), p(1.0, 0.0), p(0.0, 0.0)], closed: true, k: 2 };
        let good = Raw { pts: vec![p(0.0, 0.0), p(4.0, 0.0)], closed: false, k: 0 };
        let marks = vec![(0, Mark::whole(0, MarkKind::Fill)), (1, Mark::whole(0, MarkKind::FullWidth))];
        let (curves, kept, notes) = finish(&[bad, good], marks, |_| Family::Ring).unwrap();
        assert_eq!(curves.len(), 1);
        assert_eq!(kept.len(), 1);
        assert_eq!((kept[0].curve(), kept[0].kind()), (0, MarkKind::FullWidth));
        assert_eq!(notes.len(), 1);
        assert!(notes[0].text().contains("1 mark(s) on it are not kept"), "{}", notes[0].text());
    }
}
