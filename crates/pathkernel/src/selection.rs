//! The selection of stage 1 (K1, #4025): the picked chains in the plane of
//! the recipe, valid by construction. An edge, a face boundary, a chain or a
//! list of points that straight lines join all become a [`Chain`]: a
//! polyline that is open or closed. A [`Selection`] holds one or more chains
//! that do not cross themselves or each other.
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 1: Region"; the segment intersection test is the orientation
//! predicate of de Berg et al., "Computational Geometry" (3rd ed., 2008),
//! section 2.1, with a sweep over the x-sorted segment boxes.

use crate::verdict::{Refusal, Stage};
use crate::{KERNEL_TOL, Point2, finite, near};

/// Refusal reason: a chain with too few points (two for an open chain,
/// three for a closed chain).
pub const SHORT_CHAIN: &str = "a selected chain needs two points, or three points when it is closed";
/// Refusal reason: a selection with a point that is not a finite number.
pub const SELECTION_NOT_FINITE: &str = "a selected point is not a finite number";
/// Refusal reason: two neighbour points of a chain at the same place.
pub const REPEATED_POINT: &str = "a selected chain has two neighbour points at the same place";
/// Refusal reason: a selection with no chain.
pub const EMPTY_SELECTION: &str = "the selection is empty: pick an edge, a face, a chain or points";
/// Refusal reason: a chain that crosses or touches itself.
pub const SELF_CROSSING: &str = "a selected chain crosses itself";
/// Refusal reason: two chains that cross or touch.
pub const CHAINS_CROSS: &str = "two selected chains cross or touch each other";
/// Refusal reason: a closed chain with no area (all its points on one line).
pub const ZERO_AREA: &str = "a selected closed chain has no area";

/// One picked polyline in the plane of the recipe: finite points, no two
/// neighbours at the same place, open or closed. A closed chain does not
/// repeat its first point at its end.
#[derive(Clone, Debug, PartialEq)]
pub struct Chain {
    pts: Vec<Point2>,
    closed: bool,
}

impl Chain {
    /// A closed chain (a face boundary, a closed edge loop, or points that
    /// straight lines join back to the first point). A last point equal to
    /// the first is dropped.
    pub fn closed(pts: Vec<Point2>) -> Result<Self, Refusal> {
        Self::new(pts, true)
    }

    /// An open chain (an edge or a run of edges, or points that straight
    /// lines join, first to last).
    pub fn open(pts: Vec<Point2>) -> Result<Self, Refusal> {
        Self::new(pts, false)
    }

    fn new(mut pts: Vec<Point2>, closed: bool) -> Result<Self, Refusal> {
        if !pts.iter().all(|p| finite(&[p.x, p.y])) {
            return Err(Refusal::new(Stage::Region, SELECTION_NOT_FINITE));
        }
        if closed && pts.len() > 1 && pts.first().zip(pts.last()).is_some_and(|(a, b)| near(*a, *b)) {
            pts.pop();
        }
        if pts.len() < if closed { 3 } else { 2 } {
            return Err(Refusal::new(Stage::Region, SHORT_CHAIN));
        }
        let c = Self { pts, closed };
        if let Some(a) = c.segments().find(|(a, b)| near(*a, *b)).map(|(a, _)| a) {
            return Err(Refusal::new(Stage::Region, REPEATED_POINT).at(a));
        }
        if closed && collinear(&c.pts) {
            let r = Refusal::new(Stage::Region, ZERO_AREA);
            return Err(match c.pts.first() {
                Some(p) => r.at(*p),
                None => r,
            });
        }
        if let Some(p) = first_crossing(&[&c]) {
            return Err(Refusal::new(Stage::Region, SELF_CROSSING).at(p));
        }
        Ok(c)
    }

    #[must_use]
    pub fn points(&self) -> &[Point2] {
        &self.pts
    }
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// The segments of the chain; a closed chain includes the closing one.
    fn segments(&self) -> impl Iterator<Item = (Point2, Point2)> + '_ {
        let n = self.pts.len();
        let count = if self.closed { n } else { n.saturating_sub(1) };
        (0..count).filter_map(move |i| Some((*self.pts.get(i)?, *self.pts.get((i + 1) % n)?)))
    }
}

/// The selection of one plan: one or more chains, none of which crosses
/// itself or another one. Nested closed chains are holes (even-odd).
#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    chains: Vec<Chain>,
}

impl Selection {
    /// The selection of `chains`. Refused when it is empty, or when two
    /// chains cross or touch.
    pub fn new(chains: Vec<Chain>) -> Result<Self, Refusal> {
        if chains.is_empty() {
            return Err(Refusal::new(Stage::Region, EMPTY_SELECTION));
        }
        let refs: Vec<&Chain> = chains.iter().collect();
        if let Some(p) = first_crossing(&refs) {
            return Err(Refusal::new(Stage::Region, CHAINS_CROSS).at(p));
        }
        Ok(Self { chains })
    }

    /// A face: its outer boundary and its holes, each a closed chain.
    pub fn face(outer: Vec<Point2>, holes: Vec<Vec<Point2>>) -> Result<Self, Refusal> {
        let mut chains = vec![Chain::closed(outer)?];
        for h in holes {
            chains.push(Chain::closed(h)?);
        }
        Self::new(chains)
    }

    /// Points that straight lines join, first to last, and back to the
    /// first when `closed`.
    pub fn points(pts: Vec<Point2>, closed: bool) -> Result<Self, Refusal> {
        Self::new(vec![Chain::new(pts, closed)?])
    }

    #[must_use]
    pub fn chains(&self) -> &[Chain] {
        &self.chains
    }

    /// The first open chain, if any.
    #[must_use]
    pub fn first_open(&self) -> Option<&Chain> {
        self.chains.iter().find(|c| !c.closed)
    }
}

/// True when all the points lie on one line within the kernel tolerance.
fn collinear(pts: &[Point2]) -> bool {
    let Some(a) = pts.first().copied() else { return true };
    let far = pts.iter().copied().max_by(|p, q| (p.x - a.x).hypot(p.y - a.y).total_cmp(&(q.x - a.x).hypot(q.y - a.y)));
    let Some(b) = far else { return true };
    let len = (b.x - a.x).hypot(b.y - a.y);
    len <= KERNEL_TOL || pts.iter().all(|p| orient(a, b, *p).abs() / len <= KERNEL_TOL)
}

fn orient(a: Point2, b: Point2, c: Point2) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn on_seg(a: Point2, b: Point2, p: Point2) -> bool {
    p.x >= a.x.min(b.x) - KERNEL_TOL
        && p.x <= a.x.max(b.x) + KERNEL_TOL
        && p.y >= a.y.min(b.y) - KERNEL_TOL
        && p.y <= a.y.max(b.y) + KERNEL_TOL
}

/// True when the closed segments `a-b` and `c-d` share a point (a touch
/// within the tolerance counts).
fn seg_meet(a: Point2, b: Point2, c: Point2, d: Point2) -> bool {
    let tol = |p: Point2, q: Point2| KERNEL_TOL * (q.x - p.x).hypot(q.y - p.y).max(1.0);
    let (d1, d2) = (orient(c, d, a), orient(c, d, b));
    let (d3, d4) = (orient(a, b, c), orient(a, b, d));
    let (t_cd, t_ab) = (tol(c, d), tol(a, b));
    if ((d1 > t_cd && d2 < -t_cd) || (d1 < -t_cd && d2 > t_cd)) && ((d3 > t_ab && d4 < -t_ab) || (d3 < -t_ab && d4 > t_ab)) {
        return true;
    }
    (d1.abs() <= t_cd && on_seg(c, d, a))
        || (d2.abs() <= t_cd && on_seg(c, d, b))
        || (d3.abs() <= t_ab && on_seg(a, b, c))
        || (d4.abs() <= t_ab && on_seg(a, b, d))
}

/// A segment with its chain and its index in the chain.
struct Seg {
    chain: usize,
    idx: usize,
    a: Point2,
    b: Point2,
}

/// True when two segments of one chain are neighbours (they share the
/// vertex between them, which is not a crossing).
fn neighbours(s: &Seg, t: &Seg, chains: &[&Chain]) -> bool {
    if s.chain != t.chain {
        return false;
    }
    let Some(c) = chains.get(s.chain) else { return false };
    t.idx == s.idx.saturating_add(1) || (c.closed && s.idx.saturating_add(1) == c.pts.len() && t.idx == 0)
}

/// The first place where a chain crosses or touches itself or another
/// chain, other than at the shared vertex of two neighbour segments. A
/// sweep over the segments sorted by their least x.
fn first_crossing(chains: &[&Chain]) -> Option<Point2> {
    let mut segs: Vec<Seg> = chains
        .iter()
        .enumerate()
        .flat_map(|(ci, c)| c.segments().enumerate().map(move |(idx, (a, b))| Seg { chain: ci, idx, a, b }))
        .collect();
    segs.sort_by(|s, t| s.a.x.min(s.b.x).total_cmp(&t.a.x.min(t.b.x)));
    for (i, s) in segs.iter().enumerate() {
        let s_max = s.a.x.max(s.b.x) + KERNEL_TOL;
        for t in segs.iter().skip(i + 1) {
            if t.a.x.min(t.b.x) > s_max {
                break;
            }
            if neighbours(s, t, chains) || neighbours(t, s, chains) {
                if s.chain == t.chain && overlap_back(s, t) {
                    return Some(s.b);
                }
                continue;
            }
            if seg_meet(s.a, s.b, t.a, t.b) {
                return Some(s.a);
            }
        }
    }
    None
}

/// Two neighbour segments that fold back on each other (a spike): the
/// angle between them is zero.
fn overlap_back(s: &Seg, t: &Seg) -> bool {
    let (u, v) = if near(s.b, t.a) { ((s.a, s.b), (t.a, t.b)) } else { ((t.a, t.b), (s.a, s.b)) };
    let (ux, uy, vx, vy) = (u.1.x - u.0.x, u.1.y - u.0.y, v.1.x - v.0.x, v.1.y - v.0.y);
    let cross = ux * vy - uy * vx;
    let dot = ux * vx + uy * vy;
    cross.abs() <= KERNEL_TOL * ux.hypot(uy).max(1.0) * vx.hypot(vy).max(1.0) && dot < 0.0
}
