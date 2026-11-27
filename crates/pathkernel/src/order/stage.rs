//! The order planner (#4177, K4): it resolves the start side, sets the
//! direction of each curve, puts the curves in the walk's order and picks
//! the start point of each closed curve. It changes nothing else (law 3):
//! the ordered set refuses any other change, and the marks go with their
//! curves.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! "Stage 4: Order" and `docs/decisions/4177-kernel-order-link.md`; the
//! nearest-start sequence of contour-parallel pocketing (Held 1991); the
//! chain order of split pieces as ported in `chain.rs`.

use super::{Direction, OrderChoice, OrderPlanner, OrderedSet, chain, side};
use crate::link::{self, Level};
use crate::plan::Cut;
use crate::guide::{Curve, Family, GuideSet, MarkKind, Span};
use crate::region::{KernelRegion, PlanarRegion};
use crate::verdict::{Refusal, Stage, Verdict};
use crate::{Point2, Region};

/// Refusal reason: the order stage was given a region that is not planar.
pub const ORDER_NOT_PLANAR: &str = "the order stage plans planar regions only";

/// The reach of the chain order, in tool radii (as the air plan's chains).
const CHAIN_REACH: f64 = 3.0;
/// The vertices of a closed curve that the walk tries as its start.
const START_TRIES: usize = 8;
/// The starts that the walk asks the link stage to prove at each step.
const PICK_TRIES: usize = 16;

/// The order planner of the kernel. It reads the region, never a material
/// side (law 2), and changes only the order, the start points and the
/// directions (law 3).
#[derive(Clone, Copy, Debug, Default)]
pub struct OrderStage;

/// Where a full-width stretch of a curve ends, as the curve is cut.
#[derive(Clone, Copy)]
enum Heavy {
    /// No full-width mark.
    None,
    /// The whole curve is full-width (a `Span::Whole` mark). The curve is
    /// taken after the other ready curves, as a curve with a stretch. It
    /// has no stretch end to start at, so it starts as a plain curve does
    /// (`starts`): no point of it is less heavy than another.
    Whole,
    /// The end of the longest full-width stretch: a closed curve starts
    /// there, so that the stretch is cut last on the curve.
    EndsAt(Point2),
}

fn dist(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// `c` in the other direction (a closed curve keeps its start).
fn reversed(c: &Curve) -> Result<Curve, Refusal> {
    let mut pts = c.pts().to_vec();
    match (c.closed(), pts.get_mut(1..)) {
        (true, Some(rest)) => rest.reverse(),
        _ => pts.reverse(),
    }
    Curve::new(pts, c.arc(), c.closed(), c.k(), c.family())
}

/// `c` in the direction that puts the deeper curves on the left when
/// `want_left` (on the right else); and whether it was reversed.
fn orient(p: &PlanarRegion, c: &Curve, want_left: bool) -> Result<(Curve, bool), Refusal> {
    match side::deeper_left(p, c) {
        Some(left) if left != want_left => Ok((reversed(c)?, true)),
        _ => Ok((c.clone(), false)),
    }
}

/// The full-width stretch of each run: the end of its longest stretch in
/// the run's direction, or `Heavy::Whole` when a mark names the whole
/// curve (a whole mark wins over the stretches of the same curve).
fn heavy(g: &GuideSet, turned: &[bool]) -> Vec<Heavy> {
    let mut out = vec![Heavy::None; g.curves().len()];
    let mut longest = vec![0.0f64; g.curves().len()];
    for m in g.marks().iter().filter(|m| m.kind() == MarkKind::FullWidth) {
        let i = m.curve();
        let (Some(slot), Some(len)) = (out.get_mut(i), longest.get_mut(i)) else {
            continue;
        };
        match m.span() {
            Span::Whole => *slot = Heavy::Whole,
            Span::Part(pts) => {
                let l: f64 = pts.windows(2).map(|w| dist(w[0].at(), w[1].at())).sum();
                let end = if turned.get(i).copied().unwrap_or(false) { pts.first() } else { pts.last() };
                if let (Some(e), false) = (end, matches!(slot, Heavy::Whole))
                    && l > *len
                {
                    *len = l;
                    *slot = Heavy::EndsAt(e.at());
                }
            }
        }
    }
    out
}

/// The vertices of `c` by their distance to `q`, the nearest first.
fn by_distance(c: &Curve, q: Point2) -> Vec<usize> {
    let mut v: Vec<(f64, usize)> = c.pts().iter().enumerate().map(|(i, p)| (dist(p.at(), q), i)).collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v.into_iter().map(|(_, i)| i).collect()
}

/// `c` restarted at its vertex `j` (a closed curve).
fn restarted(c: &Curve, j: usize) -> Result<Curve, Refusal> {
    let mut pts = c.pts().to_vec();
    let n = pts.len();
    pts.rotate_left(j.min(n));
    Curve::new(pts, c.arc(), true, c.k(), c.family())
}

/// The ways to start `c` from `at`: a closed curve at the end of its
/// full-width stretch (cut last on the curve) or at one of its vertices
/// nearest to `at`; a row from its end nearer to `at`; another open
/// curve as it is.
///
/// On the first step there is no tool position (`at` is `None`): a closed
/// curve without a stretch starts at its first vertex, and a row is never
/// reversed. The row then keeps the direction that the guide planner gave
/// it, so the first cut of a level is the guide's own (deterministic, and
/// the order invents no direction that no tool position asks for). Each
/// later row starts at its end nearer to the tool (a zigzag).
fn starts(c: &Curve, at: Option<Point2>, h: Heavy) -> Result<Vec<Curve>, Refusal> {
    if c.closed() {
        let js = match (h, at) {
            (Heavy::EndsAt(e), _) => by_distance(c, e).into_iter().take(1).collect(),
            (Heavy::None | Heavy::Whole, Some(q)) => by_distance(c, q).into_iter().take(START_TRIES).collect(),
            (Heavy::None | Heavy::Whole, None) => vec![0],
        };
        return js.into_iter().map(|j| restarted(c, j)).collect();
    }
    let ends = (c.pts().first().map(|p| p.at()), c.pts().last().map(|p| p.at()));
    match (c.family(), at, ends) {
        (Family::Row, Some(q), (Some(a), Some(b))) if dist(b, q) < dist(a, q) => Ok(vec![reversed(c)?]),
        // A row on the first step (no `at`), a row already from its nearer
        // end, and any other open curve: as it is.
        _ => Ok(vec![c.clone()]),
    }
}

/// Where the tool stands after `c`.
fn end_of(c: &Curve) -> Option<Point2> {
    if c.closed() { c.pts().first() } else { c.pts().last() }.map(|p| p.at())
}

fn first_of(c: &Curve) -> Point2 {
    c.pts().first().map_or(Point2::new(0.0, 0.0), |q| q.at())
}

/// The walk: each step takes a run whose neighbours on the side the walk
/// comes from are cut. Of those, it takes the first start (a curve with a
/// full-width stretch or a whole full-width mark last; then the run that
/// goes on the chain of the last run; then the nearest) to which the link
/// stage proves an in-plane link; if none is proved, the first start (the
/// link will lift, named).
fn walk(p: &PlanarRegion, runs: &[Curve], heavy: &[Heavy], deep: bool) -> Result<Vec<Curve>, Refusal> {
    let reach = 2.0 * p.tool().radius() + crate::KERNEL_TOL;
    let needs = chain::needs(runs, deep, reach);
    let levels = chain::levels(runs, deep);
    let ends: Vec<(Point2, Point2)> = runs.iter().map(|c| (first_of(c), end_of(c).unwrap_or_else(|| first_of(c)))).collect();
    let chain_of = chain::chains(&levels, &ends, CHAIN_REACH * p.tool().radius());
    let mut done = vec![false; runs.len()];
    let mut made: Vec<Cut> = Vec::with_capacity(runs.len());
    let mut out: Vec<Curve> = Vec::with_capacity(runs.len());
    let mut last: Option<usize> = None;
    let mut paths: Vec<Vec<Point2>> = Vec::new();
    // The cuts still planned (their run indices beside them): built once,
    // and a run leaves the list when the walk takes it.
    let mut planned: Vec<Cut> = runs.iter().map(|c| Cut::new(c.clone(), None)).collect::<Result<_, _>>()?;
    let mut planned_of: Vec<usize> = (0..runs.len()).collect();
    while out.len() < runs.len() {
        let at = out.last().and_then(end_of);
        let mut options: Vec<((bool, bool, f64), usize, Curve)> = Vec::new();
        for (i, c) in runs.iter().enumerate() {
            let ready = needs.get(i).is_some_and(|n| n.iter().all(|&j| done.get(j).copied().unwrap_or(true)));
            if done.get(i).copied().unwrap_or(true) || !ready {
                continue;
            }
            let h = heavy.get(i).copied().unwrap_or(Heavy::None);
            let late = !matches!(h, Heavy::None);
            let other = last.is_some_and(|l| chain_of.get(l) != chain_of.get(i));
            for s in starts(c, at, h)? {
                let far = at.map_or(0.0, |q| dist(q, first_of(&s)));
                options.push(((late, other, far), i, s));
            }
        }
        options.sort_by(|a, b| a.0.0.cmp(&b.0.0).then(a.0.1.cmp(&b.0.1)).then(a.0.2.total_cmp(&b.0.2)));
        let proved = |s: &Curve| -> Result<Option<Vec<Point2>>, Refusal> {
            let Some(from) = made.last() else { return Ok(Some(Vec::new())) };
            let to = Cut::new(s.clone(), None)?;
            Ok(link::proved_path(from, &to, p, Level::new(&made, &planned).with_links(&paths)))
        };
        let mut pick = None;
        for (n, o) in options.iter().enumerate().take(PICK_TRIES) {
            if let Some(path) = proved(&o.2)? {
                pick = Some((n, path));
                break;
            }
        }
        let (n, path) = pick.map_or((0, None), |(n, path)| (n, Some(path)));
        let Some((_, i, s)) = options.into_iter().nth(n) else {
            return Err(Refusal::new(Stage::Order, super::NOT_A_PERMUTATION));
        };
        paths.extend(path.filter(|p| !p.is_empty()));
        if let Some(d) = done.get_mut(i) {
            *d = true;
        }
        if let Some(k) = planned_of.iter().position(|&j| j == i) {
            planned.remove(k);
            planned_of.remove(k);
        }
        last = Some(i);
        made.push(Cut::new(s.clone(), None)?);
        out.push(s);
    }
    Ok(out)
}

impl OrderPlanner for OrderStage {
    fn order(&self, g: GuideSet, r: &KernelRegion, o: &OrderChoice, _stock: Option<&Region>) -> Result<Verdict<OrderedSet>, Refusal> {
        let p = r.planar().ok_or(Refusal::new(Stage::Order, ORDER_NOT_PLANAR))?;
        let resolved = side::resolve(p, o.start);
        let deep = side::deeper_first(p, g.curves(), resolved);
        // Climb puts the uncut material on the right of the travel: the
        // deeper side when the deeper curves come later, else the other.
        let want_left = deep == (o.direction == Direction::Climb);
        let (runs, turned): (Vec<Curve>, Vec<bool>) =
            g.curves().iter().map(|c| orient(p, c, want_left)).collect::<Result<Vec<_>, _>>()?.into_iter().unzip();
        let heavy = heavy(&g, &turned);
        let seq = walk(p, &runs, &heavy, deep)?;
        let start = seq.first().map_or(Point2::new(0.0, 0.0), first_of);
        Ok(Verdict::done(OrderedSet::new(&g, seq, start, resolved)?))
    }
}
