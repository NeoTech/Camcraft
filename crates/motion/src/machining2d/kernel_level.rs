//! One level of a path kernel recipe (#3752, K6 of #4023): the stages in
//! the record's order - guide, order, load, then the cuts with a proved
//! link between each two, an entry after each lift and before the first
//! cut, and the exit after the last cut. The caller gives the region of
//! the level, the recipe and the stage state of the level (its stock
//! reader and its cleared area). The stage verdicts are kept as text for
//! the status line: each fallback and each note of a stage. A gap of an
//! incomplete verdict is kept too: the caller refuses the level, because
//! only a done verdict can reach the post.
//! Provenance: our own design record `docs/decisions/4024-path-kernel-design.md`
//! ("Data flow", "Common result types") and the stage order of the law kit
//! (`crates/pathkernel/tests/laws/kit.rs`).

use crate::geo2d::{Point2, Region};
use camcraft_pathkernel::entry::{EntryPlanner, EntryStage};
use camcraft_pathkernel::guide::{GuidePlanner, Guides};
use camcraft_pathkernel::link::{Hop, Level, LinkProof, LinkStage, LinkTally};
use camcraft_pathkernel::load::{LoadPlanner, LoadStage, StockRead};
use camcraft_pathkernel::order::{OrderPlanner, OrderStage};
use camcraft_pathkernel::plan::{Cut, Plan, Step};
use camcraft_pathkernel::{Fallback, Gap, KernelRegion, Note, Recipe, Refusal, Stage, Verdict};

/// The tolerance (mm) that the level gives to the guide stage.
pub const GUIDE_TOL: f64 = 1e-3;

/// Refusal reason: a region that is not planar.
pub const LEVEL_NOT_PLANAR: &str = "the kernel level plans planar regions only";

/// The stage state of one level: the entry and exit planner with the
/// heights of the level, the load planner with its stated limit, the stock
/// that stands at the depth of the level, and the area where no stock
/// stands down to that depth.
pub struct LevelStages<'a> {
    pub entry: &'a EntryStage,
    pub load: &'a LoadStage,
    pub stock: &'a dyn StockRead,
    pub cleared: &'a Region,
}

/// The plan of one level, the status line texts of its stage verdicts and
/// the gaps that an incomplete verdict named.
#[derive(Clone, Debug)]
pub struct PlannedLevel {
    pub plan: Plan,
    pub tally: LinkTally,
    pub notes: Vec<String>,
    pub gaps: Vec<Gap>,
}

/// The status line text of a fallback: the word "instead" lets the law
/// measures and the reader see that a behaviour was substituted (law 4).
#[must_use]
pub fn fallback_text(f: &Fallback) -> String {
    format!("{}: {} instead of {} - {}", stage_name(f.stage()), f.used(), f.asked(), f.reason())
}

/// The name of a stage on the status line.
#[must_use]
pub fn stage_name(s: Stage) -> &'static str {
    match s {
        Stage::Recipe => "kernel recipe",
        Stage::Region => "kernel region",
        Stage::Guide => "kernel guide",
        Stage::Order => "kernel order",
        Stage::Load => "kernel load",
        Stage::Link => "kernel link",
        Stage::Entry => "kernel entry",
        Stage::Exit => "kernel exit",
        _ => "kernel",
    }
}

/// The status line texts of the verdict `v` go to `notes`, its gaps to
/// `gaps`; the output comes back. A refused verdict is the error.
fn take<T: Clone>(v: Verdict<T>, notes: &mut Vec<String>, gaps: &mut Vec<Gap>) -> Result<T, Refusal> {
    let push = |notes: &mut Vec<String>, n: &[Note]| {
        for t in n.iter().map(|n| n.text().to_string()) {
            if !notes.contains(&t) {
                notes.push(t);
            }
        }
    };
    match v {
        Verdict::Done { out, fallbacks, notes: n } => {
            for t in fallbacks.iter().map(fallback_text) {
                if !notes.contains(&t) {
                    notes.push(t);
                }
            }
            push(notes, &n);
            Ok(out)
        }
        Verdict::Incomplete { out, gaps: g, notes: n } => {
            push(notes, &n);
            gaps.extend(g.list().iter().copied());
            Ok(out)
        }
        Verdict::Refused(r) => Err(r),
    }
}

/// The plan of one level of `recipe` in `region` with the stage state `s`.
pub fn plan_level(region: &KernelRegion, recipe: &Recipe, s: &LevelStages<'_>) -> Result<PlannedLevel, Refusal> {
    let tool_r = region.planar().ok_or_else(|| Refusal::new(Stage::Region, LEVEL_NOT_PLANAR))?.tool().radius();
    let (mut notes, mut gaps) = (Vec::new(), Vec::new());
    let gs = take(Guides.guide(region, &recipe.guide(), GUIDE_TOL)?, &mut notes, &mut gaps)?;
    let os = take(OrderStage.order(gs, region, &recipe.order(), None)?, &mut notes, &mut gaps)?;
    let cs = take(s.load.control(os, region, s.stock, &recipe.load(), tool_r)?, &mut notes, &mut gaps)?;
    let cuts = cs.set().curves().iter().map(|c| Cut::new(c.clone(), None)).collect::<Result<Vec<_>, _>>()?;
    let mut tally = LinkTally::default();
    let mut links: Vec<Vec<Point2>> = Vec::new();
    let mut steps = Vec::with_capacity(cuts.len().saturating_mul(2).saturating_add(2));
    for (i, cut) in cuts.iter().enumerate() {
        let lifted = match i.checked_sub(1).and_then(|j| cuts.get(j)) {
            None => true,
            Some(prev) => {
                let level = Level::new(cuts.get(..i).unwrap_or_default(), cuts.get(i..).unwrap_or_default()).with_links(&links);
                let hop = LinkStage.link(prev, cut, region, level, recipe.link(), &mut tally);
                if let Hop::Stay(path) = &hop {
                    links.push(path.points());
                }
                let lift = matches!(hop, Hop::Lift(_));
                steps.push(Step::Link(hop));
                lift
            }
        };
        if lifted {
            let e = take(s.entry.entry(cut, region, s.cleared, recipe.entry())?, &mut notes, &mut gaps)?;
            steps.push(Step::Entry(e));
        }
        steps.push(Step::Cut(cut.clone()));
    }
    if let Some(last) = cuts.last() {
        steps.push(Step::Exit(s.entry.exit(last, region, s.cleared)?));
    }
    Ok(PlannedLevel { plan: Plan::new(steps)?, tally, notes, gaps })
}
