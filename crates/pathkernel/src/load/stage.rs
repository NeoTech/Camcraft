//! The load stage (K3, #4026): one planner for the three load choices.
//! It changes guide curves only (record "Stage 3": never an entry, a link
//! or a lift). `None` keeps the curves and names each cut above the limit;
//! constant engagement inserts passes where a cut reads above the load
//! (`rings.rs`); trochoidal replaces each full-width stretch by loops
//! (`trochoid.rs`). Each mode reads with the one engagement law
//! (`law.rs`), and each cut that stays above the limit is named on the
//! status line (the maintainer's answer 4 of the K0 record).
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 3: Load control" and `docs/decisions/4026-kernel-load-control.md`.

use super::law::{Read, Stock, read};
use super::{Controlled, Engagement, LoadControl, LoadPlanner, OverLimit, PLAN_TOL, Reach, Reading, StockRead, rings, trochoid};
use crate::frame::Positive;
use crate::guide::{Curve, GuideSet, Mark, Span};
use crate::order::OrderedSet;
use crate::region::KernelRegion;
use crate::verdict::{Note, Refusal, Stage, Verdict};
use crate::{KERNEL_TOL, Point2, near, seg_dist};

/// The limit (ae/D) that the stage states when the load choice has none
/// (`None` and `Trochoidal`): a contact angle of 90 degrees, half the
/// diameter.
pub const DEFAULT_LIMIT: f64 = 0.5;

/// Refusal reason: a region that is not planar.
pub const LOAD_NOT_PLANAR: &str = "the load stage reads planar regions only";
/// Refusal reason: the stock reach of constant engagement (Adaptive).
pub const STOCK_REACH_ELSEWHERE: &str =
    "constant engagement on the in-process stock (Adaptive) is planned by the HSM planner behind the load trait, not by this stage";
/// Refusal reason: the controlled curves and their marks do not make an
/// ordered set again (a mark off its curve, a start off the first curve).
pub const SET_NOT_KEPT: &str = "the load stage cannot keep the controlled curves, their marks and their start as an ordered set";
/// Over-limit reason of `None`.
pub const NO_CONTROL: &str = "no load control: this cut is above the load limit";
/// Over-limit reason: no earlier pass is beside the cut.
pub const FRESH_CUT: &str = "a full-width cut: no earlier pass is beside it";

/// The load stage, with the limit that it states for a choice without one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoadStage {
    limit: Engagement,
}

impl Default for LoadStage {
    fn default() -> Self {
        Self { limit: Engagement(DEFAULT_LIMIT) }
    }
}

impl LoadStage {
    /// The stage with the stated limit `limit` (ae/D above 0, at most 1).
    pub fn new(limit: f64) -> Result<Self, Refusal> {
        Ok(Self { limit: Engagement::limit(Stage::Load, limit)? })
    }
    #[must_use]
    pub fn limit(&self) -> f64 {
        self.limit.get()
    }
}

impl LoadPlanner for LoadStage {
    fn control(&self, s: OrderedSet, r: &KernelRegion, stock: &dyn StockRead, lc: &LoadControl, tool_r: f64) -> Result<Verdict<Controlled>, Refusal> {
        let p = r.planar().ok_or(Refusal::new(Stage::Load, LOAD_NOT_PLANAR))?;
        let tool_r = Positive::new(Stage::Load, tool_r)?.get();
        let base = Stock::new(p.center(), stock, tool_r)?;
        match lc {
            LoadControl::None => {
                let reads = read(&base, s.curves(), tool_r);
                let out = Outcome { curves: None, marks: s.marks().to_vec(), reads };
                finish(&s, out, self.limit.get(), "no load control", |_| NO_CONTROL)
            }
            LoadControl::ConstantEngagement(c) if c.reach() == Reach::Stock => Err(Refusal::new(Stage::Load, STOCK_REACH_ELSEWHERE)),
            LoadControl::ConstantEngagement(c) => rings::control(&s, p, &base, c.load(), tool_r),
            LoadControl::Trochoidal(t) => trochoid::control(&s, p, &base, t, self.limit.get(), tool_r),
        }
    }
}

/// What a mode gives to [`finish`]: the new curves in cut order (`None`:
/// unchanged), their marks on the new indices, and the reads of the curves.
pub(crate) struct Outcome {
    pub curves: Option<Vec<Curve>>,
    pub marks: Vec<Mark>,
    pub reads: Vec<Read>,
}

/// True when the place `p` of the marked curve is on the span of `m`
/// (within 1 um).
pub(crate) fn on_span(m: &Mark, p: Point2) -> bool {
    match m.span() {
        Span::Whole => true,
        Span::Part(pts) => pts.windows(2).any(|w| seg_dist(p, w[0].at(), w[1].at()) <= 1e-3),
    }
}

/// The distance from `p` to curve `c` (its closing segment included).
pub(crate) fn poly_dist(p: Point2, c: &Curve) -> f64 {
    let pts = c.pts();
    let close = c.closed().then(|| pts.last().zip(pts.first())).flatten();
    let segs = pts.windows(2).map(|w| (w[0].at(), w[1].at())).chain(close.map(|(a, b)| (a.at(), b.at())));
    let d = segs.map(|(a, b)| seg_dist(p, a, b)).fold(f64::INFINITY, f64::min);
    if pts.len() == 1 { pts.first().map_or(d, |q| (q.at().x - p.x).hypot(q.at().y - p.y)) } else { d }
}

/// The ordered set with `curves` in this order and `marks` on their
/// indices; the start side of `s` stays.
fn rebuild(s: &OrderedSet, curves: Vec<Curve>, marks: Vec<Mark>) -> Result<OrderedSet, Refusal> {
    // A refusal of the guide set or of the ordered set is this stage's own
    // failure: it names the load stage, not the constructor (#4176).
    let kept = |_| Refusal::new(Stage::Load, SET_NOT_KEPT);
    let gs = GuideSet::with_marks(curves.clone(), marks).map_err(kept)?;
    let start = curves.first().and_then(|c| c.pts().first()).map_or(s.start(), |q| q.at());
    OrderedSet::new(&gs, curves, start, s.resolved()).map_err(kept)
}

/// The controlled set: the readings of `out`, each read above
/// `limit x (1 + PLAN_TOL)` named with `why`, and one status line note
/// for each reason. A second read at the same place and load gets no
/// second record: it is the same cut place (a curve that ends where the
/// next starts), and the one record names both.
pub(crate) fn finish(s: &OrderedSet, out: Outcome, limit: f64, mode: &str, why: impl Fn(&Read) -> &'static str) -> Result<Verdict<Controlled>, Refusal> {
    let cap = limit * (1.0 + PLAN_TOL);
    let set = match out.curves {
        None => s.clone(),
        Some(c) => rebuild(s, c, out.marks)?,
    };
    let (mut readings, mut over, mut hot): (Vec<Reading>, Vec<OverLimit>, Vec<Read>) = (Vec::new(), Vec::new(), Vec::new());
    for rd in &out.reads {
        readings.push(Reading::new(rd.at, rd.load)?);
        if rd.load > cap && !hot.iter().any(|h| near(h.at, rd.at) && (h.load - rd.load).abs() <= KERNEL_TOL) {
            hot.push(*rd);
            over.push(OverLimit::new(rd.at, rd.load, why(rd))?);
        }
    }
    let mut notes = Vec::new();
    let mut reasons: Vec<&'static str> = over.iter().map(OverLimit::reason).collect();
    reasons.sort_unstable();
    reasons.dedup();
    for reason in reasons {
        let mine: Vec<&OverLimit> = over.iter().filter(|o| o.reason() == reason).collect();
        if let Some(w) = mine.iter().max_by(|a, b| a.load().total_cmp(&b.load())) {
            let (n, at) = (mine.len(), w.at());
            let text = format!(
                "{mode}: {n} cut place(s) above the load limit ae/D {limit:.3}, the worst {:.3} at ({:.3}, {:.3}): {reason}",
                w.load(),
                at.x,
                at.y
            );
            notes.push(Note::new(Stage::Load, text)?);
        }
    }
    let ctl = Controlled::new(set, readings, over, limit)?;
    Ok(Verdict::Done { out: ctl, fallbacks: Vec::new(), notes })
}

/// True when an earlier curve than `i` (in `curves`) lies within `reach` of `p`.
pub(crate) fn beside(curves: &[Curve], i: usize, p: Point2, reach: f64) -> bool {
    curves.iter().take(i).any(|c| poly_dist(p, c) <= reach)
}
