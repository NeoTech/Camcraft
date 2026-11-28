//! The law kit of the K7 suite (#4027): the verdict of a law check, the
//! planner set of a kind (its five stage planners), one case (a region, a
//! recipe of choices and a stock), and the run of the stages into a plan.
//! A kind that moves to the kernel registers its planner set and its cases
//! in `registry.rs`; each law check reads only what this kit gives it.
//! A refused or unread leg is a distinct outcome, never held (#4406).
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

pub use super::kit_geo::{dist, outside, poly, poly_dist, seq_gap, set_gap};

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

/// One measured value of a check, the limit that it is held to, and its
/// place. A measure that could not be read (a stage refused, or the check
/// had nothing to read) carries the reason in `refused`: it is never held
/// and never broken (#4406).
#[derive(Clone, Debug, PartialEq)]
pub struct Measure {
    pub value: f64,
    pub limit: f64,
    pub place: String,
    pub at: Option<Point2>,
    pub refused: Option<String>,
}

impl Measure {
    pub fn new(value: f64, limit: f64, place: impl Into<String>, at: Option<Point2>) -> Self {
        Self { value, limit, place: place.into(), at, refused: None }
    }
    /// A count of events that must not occur (limit 0).
    pub fn count(n: usize, place: impl Into<String>, at: Option<Point2>) -> Self {
        Self::new(n as f64, 0.0, place, at)
    }
    /// A leg of a check that a stage refused: not measured.
    pub fn refusal(place: impl Into<String>, e: &Refusal) -> Self {
        let place = format!("{}: refused by {:?}: {}", place.into(), e.stage(), e.reason());
        Self { value: f64::NAN, limit: f64::NAN, place, at: e.place(), refused: Some(e.reason().to_string()) }
    }
    /// A leg of a check with nothing to read (no curve, no entry, no cut).
    pub fn unread(place: impl Into<String>, why: &str) -> Self {
        let place = format!("{}: not measured: {why}", place.into());
        Self { value: f64::NAN, limit: f64::NAN, place, at: None, refused: Some(why.to_string()) }
    }
    /// At or below the limit (false for a NaN value or limit).
    fn within(&self) -> bool {
        self.value <= self.limit
    }
    fn held(&self) -> bool {
        self.refused.is_none() && self.within()
    }
    fn broken(&self) -> bool {
        self.refused.is_none() && !self.within()
    }
    fn excess(&self) -> f64 {
        if self.value.is_nan() || self.limit.is_nan() { f64::INFINITY } else { self.value - self.limit }
    }
}

/// The outcome of one law on one case (#4406). Only `Held` is a pass: a
/// refused or unread law is not held.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// Each measure is at or below its limit, and no leg was refused.
    Held,
    /// One or more measures are above their limits.
    Broken,
    /// No measure broke, and one or more legs were refused or had nothing
    /// to read: the law is not measured. The reason of the first such leg.
    Refused(String),
}

/// The verdict of one law on one case: the law, its outcome, and the worst
/// measure (its place and value).
#[derive(Clone, Debug, PartialEq)]
pub struct LawVerdict {
    pub law: Law,
    pub outcome: Outcome,
    pub worst: Measure,
}

impl LawVerdict {
    pub fn held(&self) -> bool {
        self.outcome == Outcome::Held
    }
    /// The reason of a refused verdict.
    pub fn refused(&self) -> Option<&str> {
        if let Outcome::Refused(r) = &self.outcome { Some(r) } else { None }
    }
}

impl fmt::Display for LawVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = match self.outcome {
            Outcome::Held => "held",
            Outcome::Broken => "BROKEN",
            Outcome::Refused(_) => "REFUSED (not measured)",
        };
        let w = &self.worst;
        write!(f, "{:?} {state} ({}): at {}", self.law, self.law.text(), w.place)?;
        if let Some(p) = w.at {
            write!(f, " ({:.4}, {:.4})", p.x, p.y)?;
        }
        write!(f, ", measured {:.6} against the limit {:.6}", w.value, w.limit)
    }
}

/// Refusal reason of the kit: a check with no measure at all.
pub const NOTHING_TO_MEASURE: &str = "the check had nothing to measure";

/// The verdict of `law` over its measures (#4406). Broken when a measure
/// that was read is above its limit (the worst is the one with the largest
/// excess). Else refused when a leg was refused or unread, or when there
/// is no measure. Held only when each leg was read and is at or below its
/// limit.
pub fn judge(law: Law, measures: Vec<Measure>) -> LawVerdict {
    if measures.is_empty() {
        return LawVerdict { law, outcome: Outcome::Refused(NOTHING_TO_MEASURE.into()), worst: Measure::unread("the check", NOTHING_TO_MEASURE) };
    }
    if measures.iter().any(Measure::broken) {
        let worst = measures.into_iter().filter(Measure::broken).max_by(|a, b| a.excess().total_cmp(&b.excess()));
        let worst = worst.unwrap_or_else(|| Measure::new(f64::INFINITY, 0.0, "a broken measure", None));
        return LawVerdict { law, outcome: Outcome::Broken, worst };
    }
    if let Some(r) = measures.iter().find(|m| m.refused.is_some()) {
        let reason = r.refused.clone().unwrap_or_default();
        return LawVerdict { law, outcome: Outcome::Refused(reason), worst: r.clone() };
    }
    debug_assert!(measures.iter().all(Measure::held));
    let worst = measures.into_iter().max_by(|a, b| a.excess().total_cmp(&b.excess()));
    let worst = worst.unwrap_or_else(|| Measure::unread("the check", NOTHING_TO_MEASURE));
    LawVerdict { law, outcome: Outcome::Held, worst }
}

/// The verdict of a law whose run a stage refused: refused with the
/// reason, never held (#4406; before, a refusal read as held at 0 against 0).
pub fn refused(law: Law, e: &Refusal) -> LawVerdict {
    judge(law, vec![Measure::refusal("the run", e)])
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

pub fn assert_held(v: &LawVerdict) {
    eprintln!("{v}");
    assert!(v.held(), "{v}");
}

/// Asserts that the check broke `law` at a place whose text holds `place`.
pub fn assert_broken(v: &LawVerdict, law: Law, place: &str) {
    eprintln!("{v}");
    assert!(v.outcome == Outcome::Broken && v.law == law && v.worst.place.contains(place), "expected {law:?} broken at '{place}', got: {v}");
}

/// Asserts that `law` is refused (not measured, never held) with a reason
/// that holds `reason`.
pub fn assert_refused(v: &LawVerdict, law: Law, reason: &str) {
    eprintln!("{v}");
    assert!(v.law == law && v.refused().is_some_and(|r| r.contains(reason)), "expected {law:?} refused for '{reason}', got: {v}");
}
