//! Unit tests of the merge planner's ordering and the delta classes (merge_plan.rs, merge_delta.rs). No git: the
//! merge answers come from a fake `trial`. Gate map: xtask merge_plan, merge_delta.

use std::collections::BTreeMap;

use crate::merge_delta::{self, Commit, Verdict};
use crate::merge_plan::{self, Facts, Kind, Pr};

fn pr(n: u32, paths: &[&str]) -> Pr {
    Pr { number: n, title: format!("PR {n}"), head: format!("h{n}"), paths: paths.iter().map(|p| p.to_string()).collect() }
}

fn facts(prs: Vec<Pr>) -> Facts {
    Facts { base_name: "main".into(), base_sha: "base0000".into(), prs, ..Facts::default() }
}

/// A fake merge oracle: `base` conflicts are for a PR merged on the base alone, `pairs` for two PRs
/// (either order). A sequence conflicts when any member of it clashes with the PR.
#[derive(Default)]
struct Fake {
    base: BTreeMap<u32, Vec<String>>,
    pairs: BTreeMap<(u32, u32), Vec<String>>,
    /// Conflicts only when the sequence holds both of these (a three-way clash).
    triple: Option<(u32, u32, u32, Vec<String>)>,
}

impl Fake {
    fn trial(&self, onto: &[u32], pr: u32) -> Result<Option<Vec<String>>, String> {
        if onto.is_empty() {
            return Ok(self.base.get(&pr).cloned());
        }
        if let Some((x, y, z, files)) = &self.triple
            && pr == *z
            && onto.contains(x)
            && onto.contains(y)
        {
            return Ok(Some(files.clone()));
        }
        for o in onto {
            if let Some(f) = self.pairs.get(&(*o, pr)).or_else(|| self.pairs.get(&(pr, *o))) {
                return Ok(Some(f.clone()));
            }
        }
        Ok(None)
    }
}

fn run(f: &Facts, fake: &Fake) -> merge_plan::Plan {
    merge_plan::plan(f, &mut |onto: &[u32], n: u32| fake.trial(onto, n)).unwrap_or_default()
}

#[test]
fn mechanical_is_the_settled_files_and_snapshots() {
    assert!(merge_plan::mechanical(".claude/PROCESS_VERSION"));
    assert!(merge_plan::mechanical("CHANGELOG.md"));
    assert!(merge_plan::mechanical("crates/ui/tests/snapshots/x.png"));
    assert!(!merge_plan::mechanical("crates/app/src/lib.rs"));
}

#[test]
fn independent_prs_form_one_wave_docs_first_then_cool_then_hot() {
    let f = facts(vec![pr(1, &["src/a.rs"]), pr(2, &["docs/x.md"]), pr(3, &["src/lib.rs"])]);
    let p = run(&f, &Fake::default());
    assert_eq!(p.wave1, vec![2, 1, 3]);
    assert!(p.wave2.is_empty());
    assert!(p.stacked.is_empty());
}

#[test]
fn base_conflict_is_refreshed_after_wave_one_and_is_clean_then() {
    let f = facts(vec![pr(1, &["a.rs"]), pr(2, &["b.rs"])]);
    let fake = Fake { base: BTreeMap::from([(2, vec!["b.rs".to_string()])]), ..Fake::default() };
    let p = run(&f, &fake);
    assert_eq!(p.wave1, vec![1]);
    assert_eq!(p.wave2.len(), 1);
    assert_eq!(p.wave2[0].number, 2);
    assert_eq!(p.wave2[0].why, "conflicts with main now");
    assert_eq!(p.wave2[0].kind, Kind::CleanAfterWave1);
}

#[test]
fn pair_clash_on_mechanical_files_is_a_mechanical_refresh() {
    let f = facts(vec![pr(1, &["a.rs"]), pr(3, &["c.rs"])]);
    let fake = Fake { pairs: BTreeMap::from([((1, 3), vec!["Cargo.lock".to_string()])]), ..Fake::default() };
    let p = run(&f, &fake);
    assert_eq!(p.wave1, vec![1]);
    assert_eq!(p.wave2[0].number, 3);
    assert_eq!(p.wave2[0].why, "conflicts with #1");
    assert_eq!(p.wave2[0].kind, Kind::Mechanical);
}

#[test]
fn code_conflict_after_wave_one_is_by_hand() {
    let f = facts(vec![pr(1, &["a.rs"]), pr(2, &["b.rs"])]);
    let fake = Fake { pairs: BTreeMap::from([((1, 2), vec!["src/shared.rs".to_string()])]), ..Fake::default() };
    let p = run(&f, &fake);
    assert_eq!(p.wave2[0].kind, Kind::ByHand);
    assert_eq!(p.wave2[0].files, vec!["src/shared.rs".to_string()]);
}

#[test]
fn stacked_child_merges_after_its_parent_even_when_it_is_docs_only() {
    let mut f = facts(vec![pr(1, &["lib.rs"]), pr(2, &["docs/x.md"])]);
    f.before.insert(2, [1].into_iter().collect());
    let p = run(&f, &Fake::default());
    assert_eq!(p.wave1, vec![1, 2]);
    assert_eq!(p.stacked.get(&2), Some(&vec![1]));
}

#[test]
fn child_of_a_deferred_parent_is_deferred_too() {
    let mut f = facts(vec![pr(1, &["a.rs"]), pr(2, &["b.rs"])]);
    f.before.insert(2, [1].into_iter().collect());
    let fake = Fake { base: BTreeMap::from([(1, vec!["x.rs".to_string()])]), ..Fake::default() };
    let p = run(&f, &fake);
    assert!(p.wave1.is_empty());
    let numbers: Vec<u32> = p.wave2.iter().map(|w| w.number).collect();
    assert_eq!(numbers, vec![1, 2]);
    assert_eq!(p.wave2[0].kind, Kind::ByHand);
    assert_eq!(p.wave2[1].why, "its parent #1 is not in wave 1");
}

#[test]
fn shared_history_puts_the_side_with_fewer_own_commits_first() {
    let mut f = facts(vec![pr(5, &["a.rs"]), pr(6, &["b.rs"])]);
    f.shared.insert((5, 6), (3, 1));
    let p = run(&f, &Fake::default());
    assert_eq!(p.wave1, vec![6, 5]);
}

#[test]
fn three_way_clash_defers_the_merge_and_its_child() {
    let mut f = facts(vec![pr(1, &["a.rs"]), pr(2, &["b.rs"]), pr(3, &["c.rs"]), pr(4, &["d.rs"])]);
    f.before.insert(4, [3].into_iter().collect());
    let fake = Fake { triple: Some((1, 2, 3, vec!["m.rs".to_string()])), ..Fake::default() };
    let p = run(&f, &fake);
    assert_eq!(p.wave1, vec![1, 2]);
    let numbers: Vec<u32> = p.wave2.iter().map(|w| w.number).collect();
    assert_eq!(numbers, vec![3, 4]);
    assert_eq!(p.wave2[0].kind, Kind::ByHand);
    assert_eq!(p.wave2[1].kind, Kind::CleanAfterWave1);
    assert_eq!(p.wave2[1].why, "its parent #3 is not in wave 1");
}

#[test]
fn render_names_both_waves() {
    let f = facts(vec![pr(1, &["a.rs"]), pr(2, &["b.rs"])]);
    let fake = Fake { base: BTreeMap::from([(2, vec!["b.rs".to_string()])]), ..Fake::default() };
    let text = merge_plan::render(&run(&f, &fake));
    assert!(text.contains("WAVE 1"));
    assert!(text.contains("#1  PR 1"));
    assert!(text.contains("WAVE 2"));
    assert!(text.contains("[clean after wave 1] conflicts with main now"));
}

fn merge_of_main(sha: &str, files: &[&str]) -> Commit {
    Commit {
        sha: sha.into(),
        subject: "Merge main".into(),
        parents: vec!["p1".into(), "p2".into()],
        second_is_base: true,
        remerge_files: files.iter().map(|f| f.to_string()).collect(),
    }
}

#[test]
fn delta_same_head_and_rewritten_history() {
    assert_eq!(merge_delta::classify("h", "h", true, "main", &[]).verdict, Verdict::Unchanged);
    assert_eq!(merge_delta::classify("h", "r", false, "main", &[]).verdict, Verdict::Rewritten);
}

#[test]
fn delta_clean_merges_of_main_keep_the_review() {
    let d = merge_delta::classify("h", "r", true, "main", &[merge_of_main("c1", &[])]);
    assert_eq!(d.verdict, Verdict::Same);
    assert_eq!(d.notes, vec!["c1 merge of main, no hand edits".to_string()]);
}

#[test]
fn delta_hand_edits_in_settled_files_are_mechanical() {
    let d = merge_delta::classify("h", "r", true, "main", &[merge_of_main("c1", &["CHANGELOG.md"])]);
    assert_eq!(d.verdict, Verdict::Mechanical);
    assert_eq!(d.notes, vec!["c1 merge of main, hand edits only in: CHANGELOG.md".to_string()]);
}

#[test]
fn delta_hand_resolved_code_is_a_delta_review_and_new_code_beats_it() {
    let d = merge_delta::classify("h", "r", true, "main", &[merge_of_main("c1", &["src/a.rs"])]);
    assert_eq!(d.verdict, Verdict::Delta);
    let plain = Commit { sha: "c2".into(), subject: "fix".into(), parents: vec!["p1".into()], second_is_base: false, remerge_files: vec![] };
    let d = merge_delta::classify("h", "r", true, "main", &[merge_of_main("c1", &["src/a.rs"]), plain]);
    assert_eq!(d.verdict, Verdict::NewCode);
    assert_eq!(d.notes[1], "c2 commit: fix");
}

#[test]
fn delta_merge_of_another_branch_is_new_code() {
    let other = Commit { second_is_base: false, ..merge_of_main("c1", &[]) };
    let d = merge_delta::classify("h", "r", true, "main", &[other]);
    assert_eq!(d.verdict, Verdict::NewCode);
    assert!(d.notes[0].starts_with("c1 merges a branch that is not main (p2): "));
}

#[test]
fn delta_render_starts_with_the_label() {
    let d = merge_delta::classify("h", "h", true, "main", &[]);
    assert!(merge_delta::render(&d).starts_with("SAME: the head is the reviewed head"));
}
