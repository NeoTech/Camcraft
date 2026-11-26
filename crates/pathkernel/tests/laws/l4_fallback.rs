//! L4 (#4027): a fallback is visible; the status line names it. The check
//! reads what each stage used against what the recipe asked: a guide whose
//! curves are of another family than the asked guide, or an entry into
//! material of another kind than the asked one, must carry a named
//! fallback of its stage. The entry rule is `entry::needs_fallback` (#4178):
//! an entry into air, a pre-drilled hole or a cleared column where an entry
//! into material was asked is the safer entry of the record's order of
//! tries, not a fallback. The entry verdicts of the run carry the records.

use camcraft_pathkernel::entry::{Entry, EntryChoice, EntryKind, EntryPlanner, Exit, named as named_entry, needs_fallback};
use camcraft_pathkernel::guide::{Family, GuideChoice, GuidePlanner, GuideSet, RowsChoice};
use camcraft_pathkernel::link::LinkPolicy;
use camcraft_pathkernel::plan::Cut;
use camcraft_pathkernel::region::KernelRegion;
use camcraft_pathkernel::{Fallback, Recipe, Refusal, Region, Stage, Verdict};

use super::kit::{Case, Law, LawVerdict, Measure, Planners, Run, assert_broken, assert_held, judge, planar, run};
use super::standins::{ProvedEntry, STEP, good, helix, recipe, rings, rings_choice, square_case, start_of};

fn family(g: &GuideChoice) -> Option<Family> {
    match g {
        GuideChoice::Rings(_) => Some(Family::Ring),
        GuideChoice::Rows(_) => Some(Family::Row),
        GuideChoice::Medial => Some(Family::Medial),
        GuideChoice::Trace(_) => Some(Family::Trace),
        _ => None,
    }
}

/// The fallbacks and notes of one stage in a verdict.
fn named<T>(v: &Verdict<T>, stage: Stage) -> usize {
    match v {
        Verdict::Done { fallbacks, .. } => fallbacks.iter().filter(|f| f.stage() == stage).count(),
        Verdict::Incomplete { notes, .. } => notes.iter().filter(|n| n.stage() == stage).count(),
        Verdict::Refused(_) => 0,
    }
}

/// L4 over one run of the recipe `recipe`.
pub fn check_l4(run: &Run, recipe: &Recipe) -> LawVerdict {
    let mut m = Vec::new();
    let asked = family(&recipe.guide());
    let rows_ring = |f: Family| matches!(recipe.guide(), GuideChoice::Rows(r) if r.closing_ring()) && f == Family::Ring;
    let used: Vec<Family> = run.guide.out().map(|g| g.curves().iter().map(|c| c.family()).collect()).unwrap_or_default();
    let other = used.iter().filter(|&&f| Some(f) != asked && !rows_ring(f)).count();
    let names = named(&run.guide, Stage::Guide);
    let silent = if other > 0 && names == 0 { other } else { 0 };
    m.push(Measure::count(silent, format!("guide: {other} curves of another family than asked ({asked:?}), {names} named"), None));
    let asked = recipe.entry().asked();
    for (i, v) in run.entries.iter().enumerate() {
        let Some(e) = v.out() else { continue };
        let n = named(v, Stage::Entry);
        let wrong = needs_fallback(asked, e.kind()) && n == 0;
        let at = e.descent().first().map(|p| p.at());
        let place = format!("entry {i}: used {:?}, asked {asked:?}, {n} fallback records", e.kind());
        m.push(Measure::count(usize::from(wrong), place, at));
    }
    judge(Law::L4, m)
}

/// Plans rings where rows were asked (a narrow neck); `named` says so.
struct RowsAsRings {
    named: bool,
}

impl GuidePlanner for RowsAsRings {
    fn guide(&self, r: &KernelRegion, _g: &GuideChoice, _tol: f64) -> Result<Verdict<GuideSet>, Refusal> {
        let v = Verdict::done(GuideSet::new(rings(planar(r)?.center(), 0.0, STEP, None)?));
        if !self.named {
            return Ok(v);
        }
        Ok(v.with_fallback(Fallback::new(Stage::Guide, "rows", "rings", "the rows do not fit the narrow neck")?))
    }
}

/// A ramp where a helix was asked: with a fallback record when `named`,
/// else silent (breaks L4).
struct RampInstead {
    named: bool,
}

impl EntryPlanner for RampInstead {
    fn entry(&self, first: &Cut, r: &KernelRegion, _cleared: &Region, e: &EntryChoice) -> Result<Verdict<Entry>, Refusal> {
        let ramp = helix(start_of(first), r, e, EntryKind::Ramp)?;
        if self.named { named_entry(ramp, e, "the helix does not fit", vec![]) } else { Ok(Verdict::done(ramp)) }
    }
    fn exit(&self, last: &Cut, r: &KernelRegion, cleared: &Region) -> Result<Exit, Refusal> {
        ProvedEntry.exit(last, r, cleared)
    }
}

fn rows_case() -> Case {
    let rows = GuideChoice::Rows(RowsChoice::new(0.0, STEP, false).unwrap());
    Case { recipe: recipe(rows, EntryKind::Helix, LinkPolicy::StayWhereProved).unwrap(), ..square_case().unwrap() }
}

fn l4(p: &Planners, c: &Case) -> LawVerdict {
    check_l4(&run(p, c, Default::default()).unwrap(), &c.recipe)
}

#[test]
fn l4_holds_for_the_asked_stages_and_for_a_named_fallback() {
    let c = square_case().unwrap();
    assert_eq!(c.recipe.guide(), rings_choice().unwrap());
    assert_held(&l4(&good(), &c));
    assert_held(&l4(&Planners { guide: Box::new(RowsAsRings { named: true }), ..good() }, &rows_case()));
}

#[test]
fn l4_breaks_for_a_silent_guide_fallback() {
    let v = l4(&Planners { guide: Box::new(RowsAsRings { named: false }), ..good() }, &rows_case());
    assert_broken(&v, Law::L4, "guide:");
    assert!(v.worst.value >= 1.0, "{v}");
}

#[test]
fn l4_breaks_for_a_silent_entry_fallback() {
    let v = l4(&Planners { entry: Box::new(RampInstead { named: false }), ..good() }, &square_case().unwrap());
    assert_broken(&v, Law::L4, "entry 0: used Ramp, asked Helix");
}

#[test]
fn l4_holds_for_a_named_entry_fallback() {
    assert_held(&l4(&Planners { entry: Box::new(RampInstead { named: true }), ..good() }, &square_case().unwrap()));
}

/// #4178: the entry and exit stage is registered in the suite and passes L4
/// and L5 on the stand-in cases.
#[test]
fn the_entry_stage_passes_l4_and_l5() {
    let p = || Planners { entry: Box::new(super::entry_stage::stage()), ..good() };
    for c in [square_case().unwrap(), super::standins::island_case().unwrap()] {
        let r = run(&p(), &c, Default::default());
        assert!(r.is_ok(), "{}: {:?}", c.name, r.as_ref().err());
        assert_held(&super::l5_entry::check_l5(&r, &c));
        let r = r.unwrap();
        assert!(r.entries.iter().all(|v| v.out().is_some_and(|e| e.kind() == EntryKind::Helix)), "{}", c.name);
        assert_held(&check_l4(&r, &c.recipe));
    }
}
