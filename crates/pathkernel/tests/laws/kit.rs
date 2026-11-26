//! The law kit of the K7 suite (#4027): the verdict of a law check, the
//! planner set of a kind (its five stage planners), one case (a region, a
//! recipe of choices and a stock), and the run of the stages into a plan.
//! A kind that moves to the kernel registers its planner set and its cases
//! in `registry.rs`; each law check reads only what this kit gives it.
//! Provenance: our own design record (RECOMMENDED.md "#4024 - Path kernel
//! K0", "The laws as checks").

use std::fmt;

use camcraft_pathkernel::entry::{Entry, EntryPlanner};
use camcraft_pathkernel::guide::{Curve, GuidePlanner, GuideSet};
use camcraft_pathkernel::link::{Hop, Level, LinkProof, LinkTally};
use camcraft_pathkernel::load::{Controlled, LoadPlanner, StockRead};
use camcraft_pathkernel::order::{OrderChoice, OrderPlanner, OrderedSet, StartSide};
use camcraft_pathkernel::plan::{Cut, Plan, Step};
use camcraft_pathkernel::region::{KernelRegion, MaterialSide, PlanarRegion};
use camcraft_pathkernel::{Loop, Point2, Recipe, Refusal, Region, Stage, Verdict};

/// Two curves match within 1 um (record "The laws as checks").
pub const MATCH_TOL: f64 = 1e-3;
/// The tolerance that the runner gives to the guide stage.
pub const GUIDE_TOL: f64 = 1e-3;
/// Refusal reason of the kit: a region that is not planar (phase 1 has only planar ones).
pub const NOT_PLANAR: &str = "the law kit reads planar regions only";
/// Refusal reason of the kit: a stage that is not refused gave no output.
pub const NO_OUTPUT: &str = "a stage that is not refused gave no output";

/// The laws of #4023 as the record numbers them, and the link rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Law {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
    Link,
}

impl Law {
    pub fn text(self) -> &'static str {
        match self {
            Self::L1 => "the same region and the same guide choice give the same curves",
            Self::L2 => "the material side and the start side are separate parameters",
            Self::L3 => "the start side changes only the order and the start point",
            Self::L4 => "a fallback is visible: the status line names it",
            Self::L5 => "an entry into material needs a proof",
            Self::L6 => "a cut above the engagement limit is prevented or named",
            Self::L7 => "a move that migrates keeps its output",
            Self::Link => "stay at depth; lift only with a named reason",
        }
    }
}

/// One measured value of a check, the limit that it is held to, and its place.
#[derive(Clone, Debug, PartialEq)]
pub struct Measure {
    pub value: f64,
    pub limit: f64,
    pub place: String,
    pub at: Option<Point2>,
}

impl Measure {
    pub fn new(value: f64, limit: f64, place: impl Into<String>, at: Option<Point2>) -> Self {
        Self { value, limit, place: place.into(), at }
    }
    /// A count of events that must not occur (limit 0).
    pub fn count(n: usize, place: impl Into<String>, at: Option<Point2>) -> Self {
        Self::new(n as f64, 0.0, place, at)
    }
    fn held(&self) -> bool {
        self.value <= self.limit
    }
    fn excess(&self) -> f64 {
        if self.value.is_nan() { f64::INFINITY } else { self.value - self.limit }
    }
}

/// The verdict of one law on one case: the law, held or broken, and the
/// worst measure (its place and value).
#[derive(Clone, Debug, PartialEq)]
pub struct LawVerdict {
    pub law: Law,
    pub held: bool,
    pub worst: Measure,
}

impl fmt::Display for LawVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = if self.held { "held" } else { "BROKEN" };
        let w = &self.worst;
        write!(f, "{:?} {state} ({}): at {}", self.law, self.law.text(), w.place)?;
        if let Some(p) = w.at {
            write!(f, " ({:.4}, {:.4})", p.x, p.y)?;
        }
        write!(f, ", measured {:.6} against the limit {:.6}", w.value, w.limit)
    }
}

/// The verdict of `law` over its measures: held when each measure is at or
/// below its limit; the worst measure is the one with the largest excess.
pub fn judge(law: Law, measures: Vec<Measure>) -> LawVerdict {
    let held = measures.iter().all(Measure::held);
    let worst = measures
        .into_iter()
        .max_by(|a, b| a.excess().total_cmp(&b.excess()))
        .unwrap_or_else(|| Measure::new(0.0, 0.0, "nothing to measure", None));
    LawVerdict { law, held, worst }
}

/// A law that holds because a stage refused with a named reason (the
/// Prime Directive: refused is a legal result).
pub fn refused_held(law: Law, e: &Refusal) -> LawVerdict {
    judge(law, vec![Measure::new(0.0, 0.0, format!("refused by {:?}: {}", e.stage(), e.reason()), e.place())])
}

/// The five stage planners of one kind.
pub struct Planners {
    pub guide: Box<dyn GuidePlanner>,
    pub order: Box<dyn OrderPlanner>,
    pub load: Box<dyn LoadPlanner>,
    pub link: Box<dyn LinkProof>,
    pub entry: Box<dyn EntryPlanner>,
}

/// One case of a kind: its region stage, a recipe and the stock.
pub struct Case {
    pub name: &'static str,
    /// The region of the kind for a material side (K1 supplies the real one).
    pub region: fn(MaterialSide) -> Result<KernelRegion, Refusal>,
    pub material: MaterialSide,
    pub recipe: Recipe,
    /// The stock footprint at the start of the level: the check grid reads it.
    pub stock: Region,
    /// The planner's own stock reader, given to the load stage.
    pub reader: Box<dyn StockRead>,
    /// The fixture has no split ring: each link must stay at depth.
    pub no_split: bool,
    /// The recorded old output (L7): the cut curves in order.
    pub recorded: Option<Vec<Curve>>,
}

/// The outputs of the stages of one run, for one start side.
pub struct Run {
    pub side: StartSide,
    pub region: KernelRegion,
    pub guide: Verdict<GuideSet>,
    pub ordered: Verdict<OrderedSet>,
    pub controlled: Verdict<Controlled>,
    pub plan: Plan,
    pub tally: LinkTally,
    /// The verdict of each entry, in plan order: L4 reads its fallbacks.
    pub entries: Vec<Verdict<Entry>>,
}

fn live<T: Clone>(v: Verdict<T>) -> Result<(Verdict<T>, T), Refusal> {
    match v {
        Verdict::Refused(r) => Err(r),
        v => {
            let out = v.out().cloned().ok_or_else(|| Refusal::new(Stage::Recipe, NO_OUTPUT))?;
            Ok((v, out))
        }
    }
}

pub fn planar(r: &KernelRegion) -> Result<&PlanarRegion, Refusal> {
    r.planar().ok_or_else(|| Refusal::new(Stage::Region, NOT_PLANAR))
}

/// Runs the stages of `p` on `c` for the start side `side`: region, guide,
/// order, load, then one entry, the cuts with a link between each two (a
/// lift needs a new entry), and one exit.
pub fn run(p: &Planners, c: &Case, side: StartSide) -> Result<Run, Refusal> {
    let region = (c.region)(c.material)?;
    let tool_r = planar(&region)?.tool().radius();
    let (guide, gs) = live(p.guide.guide(&region, &c.recipe.guide(), GUIDE_TOL)?)?;
    let oc = OrderChoice { start: side, direction: c.recipe.order().direction };
    let (ordered, os) = live(p.order.order(gs, &region, &oc, None)?)?;
    let (controlled, cs) = live(p.load.control(os, &region, c.reader.as_ref(), &c.recipe.load(), tool_r)?)?;
    let cuts = cs.set().curves().iter().map(|cv| Cut::new(cv.clone(), None)).collect::<Result<Vec<_>, _>>()?;
    let (mut steps, mut tally, cleared) = (Vec::new(), LinkTally::default(), Region::empty());
    let mut paths: Vec<Vec<Point2>> = Vec::new();
    let mut entries = Vec::new();
    for (i, cut) in cuts.iter().enumerate() {
        let lifted = match i.checked_sub(1).and_then(|j| cuts.get(j)) {
            None => true,
            Some(prev) => {
                let level = Level::new(cuts.get(..i).unwrap_or_default(), cuts.get(i..).unwrap_or_default()).with_links(&paths);
                let hop = p.link.link(prev, cut, &region, level, c.recipe.link(), &mut tally);
                if let Hop::Stay(path) = &hop {
                    paths.push(path.points());
                }
                let lift = matches!(hop, Hop::Lift(_));
                steps.push(Step::Link(hop));
                lift
            }
        };
        if lifted {
            let (v, e) = live(p.entry.entry(cut, &region, &cleared, c.recipe.entry())?)?;
            steps.push(Step::Entry(e));
            entries.push(v);
        }
        steps.push(Step::Cut(cut.clone()));
    }
    if let Some(last) = cuts.last() {
        steps.push(Step::Exit(p.entry.exit(last, &region, &cleared)?));
    }
    let plan = Plan::new(steps)?;
    Ok(Run { side, region, guide, ordered, controlled, plan, tally, entries })
}

/// The region `p` rebuilt with another material side and other part walls.
pub fn rebuild(p: &PlanarRegion, material: MaterialSide, center: Region, walls: Vec<Loop>) -> Result<KernelRegion, Refusal> {
    PlanarRegion::new(p.plane(), center, material, walls, p.stock_edge().to_vec(), p.air().clone(), p.keep_out().clone(), p.tool())
        .map(KernelRegion::Planar)
}

/// The same region given another way: the loops in reverse order, each
/// started at its next vertex (L1's permutation).
pub fn permuted(p: &PlanarRegion) -> Result<KernelRegion, Refusal> {
    let turn = |ls: &[Loop]| -> Result<Vec<Loop>, Refusal> {
        ls.iter()
            .rev()
            .map(|l| {
                let mut pts = l.points().to_vec();
                pts.rotate_left(1);
                Loop::new(pts).map_err(|_| Refusal::new(Stage::Region, NOT_PLANAR))
            })
            .collect()
    };
    let center = Region::new(turn(p.center().loops())?).map_err(|_| Refusal::new(Stage::Region, NOT_PLANAR))?;
    rebuild(p, p.material(), center, turn(p.part_walls())?)
}

/// The cut curves of a plan, in order.
pub fn plan_cuts(plan: &Plan) -> Vec<&Curve> {
    plan.steps().iter().filter_map(|s| if let Step::Cut(c) = s { Some(c.curve()) } else { None }).collect()
}

/// The points of a curve as a polyline (a closed curve repeats its start).
pub fn poly(c: &Curve) -> Vec<Point2> {
    let mut v: Vec<Point2> = c.pts().iter().map(|p| p.at()).collect();
    if let Some(&p) = v.first().filter(|_| c.closed()) {
        v.push(p);
    }
    v
}

pub fn dist(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

pub fn seg_dist(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    dist(p, Point2::new(a.x + t * dx, a.y + t * dy))
}

pub fn poly_dist(p: Point2, line: &[Point2]) -> f64 {
    match line {
        [only] => dist(p, *only),
        _ => line.windows(2).map(|w| seg_dist(p, w[0], w[1])).fold(f64::INFINITY, f64::min),
    }
}

/// The distance of `p` outside `area` (0 inside).
pub fn outside(area: &Region, p: Point2) -> f64 {
    if area.contains(p) {
        return 0.0;
    }
    let ring = |l: &Loop| {
        let mut v = l.points().to_vec();
        v.extend(l.points().first().copied());
        poly_dist(p, &v)
    };
    area.loops().iter().map(ring).fold(f64::INFINITY, f64::min)
}

/// The Hausdorff distance of two curves in the plane, read at the vertices.
pub fn curve_gap(a: &Curve, b: &Curve) -> f64 {
    let (pa, pb) = (poly(a), poly(b));
    let one = |from: &[Point2], to: &[Point2]| from.iter().map(|p| poly_dist(*p, to)).fold(0.0, f64::max);
    one(&pa, &pb).max(one(&pb, &pa))
}

/// The gap of two curve multisets (order and direction free): each curve of
/// `a` matched to its nearest unused curve of `b`. A count that differs is
/// an infinite gap.
pub fn set_gap(a: &[&Curve], b: &[&Curve]) -> (f64, String) {
    if a.len() != b.len() {
        return (f64::INFINITY, format!("{} curves against {}", a.len(), b.len()));
    }
    let (mut used, mut worst) = (vec![false; b.len()], (0.0, String::from("each curve")));
    for (i, ca) in a.iter().enumerate() {
        let best = b.iter().enumerate().filter(|(j, _)| !used[*j]).map(|(j, cb)| (curve_gap(ca, cb), j)).min_by(|x, y| x.0.total_cmp(&y.0));
        let Some((g, j)) = best else {
            return (f64::INFINITY, format!("curve {i} has no partner"));
        };
        used[j] = true;
        if g > worst.0 {
            worst = (g, format!("curve {i} (k {})", ca.k()));
        }
    }
    worst
}

/// The gap of two curve sequences, point by point (order and direction
/// count). A count that differs is an infinite gap.
pub fn seq_gap(a: &[&Curve], b: &[&Curve]) -> (f64, String) {
    if a.len() != b.len() {
        return (f64::INFINITY, format!("{} cuts against {}", a.len(), b.len()));
    }
    let mut worst = (0.0, String::from("each cut"));
    for (i, (ca, cb)) in a.iter().zip(b).enumerate() {
        let g = if ca.pts().len() == cb.pts().len() {
            let d = |(p, q): (&camcraft_pathkernel::PlanePoint, &camcraft_pathkernel::PlanePoint)| {
                dist(p.at(), q.at()).max((p.depth() - q.depth()).abs())
            };
            ca.pts().iter().zip(cb.pts()).map(d).fold(0.0, f64::max)
        } else {
            f64::INFINITY
        };
        if g > worst.0 {
            worst = (g, format!("cut {i} (k {})", ca.k()));
        }
    }
    worst
}

pub fn assert_held(v: &LawVerdict) {
    eprintln!("{v}");
    assert!(v.held, "{v}");
}

/// Asserts that the check broke `law` at a place whose text holds `place`.
pub fn assert_broken(v: &LawVerdict, law: Law, place: &str) {
    eprintln!("{v}");
    assert!(!v.held && v.law == law && v.worst.place.contains(place), "expected {law:?} broken at '{place}', got: {v}");
}
