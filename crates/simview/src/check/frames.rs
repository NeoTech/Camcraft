//! The frames of `camcraft-sim check` (#2586): a section's moves are in
//! its work offset; the stock and the fixtures are in the first
//! section's. A machine position is the work position plus the origin.
//! Provenance: the RS-274/NGC work offset rule (NISTIR 6556, G54..G59).

use camcraft_sim::cl::Move;
use camcraft_sim::geo2d::Point2;
use camcraft_sim::geometry::Point3;

/// The shift from the frame of `to` into the frame of `base`, each an
/// (offset, origin) pair: zero for the same offset, the difference of
/// the origins where both are given, else `None`.
#[must_use]
pub fn shift(base: (u8, Option<[f64; 3]>), to: (u8, Option<[f64; 3]>)) -> Option<[f64; 3]> {
    if base.0 == to.0 {
        return Some([0.0; 3]);
    }
    let (b, t) = (base.1?, to.1?);
    Some([t[0] - b[0], t[1] - b[1], t[2] - b[2]])
}

/// `mv` moved by `d`.
#[must_use]
pub fn shifted(mv: &Move, d: [f64; 3]) -> Move {
    let at = |p: Point3| Point3::new(p.x + d[0], p.y + d[1], p.z + d[2]);
    let mut out = *mv;
    out.from = at(mv.from);
    out.to = at(mv.to);
    if let Some(arc) = out.arc.as_mut() {
        // The centre is the plane's word-order pair (`ArcPlane::words`).
        let dd = arc.plane.words(Point3::new(d[0], d[1], d[2]));
        arc.center = Point2::new(arc.center.x + dd.x, arc.center.y + dd.y);
    }
    out
}

/// The first and last NC line of the moves that are not skipped.
#[must_use]
pub fn span(lines: &[usize], skip: &[bool]) -> Option<(usize, usize)> {
    let mut it = lines.iter().zip(skip).filter(|(_, k)| !**k).map(|(l, _)| *l);
    let first = it.next()?;
    Some((first, it.next_back().unwrap_or(first)))
}
