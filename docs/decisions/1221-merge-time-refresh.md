## #1221 - refresh a conflicting PR only at merge time; clean merges keep their review; only the owner's reviews count

**What was decided.** Five process points, as the issue lists them. (1) A
conflicting PR is refreshed only at merge time, in the order the planner
gives; start-of-session contributor refreshes stop. (2) `cargo xtask
merge-plan` ports the maintainer's merge-ready planner: wave 1 merges as one
batch with no refresh, wave 2 one refresh at a time. (3) A clean merge of
main (empty remerge-diff) keeps its review and its ready flag; `--delta`
classifies it SAME or MECHANICAL, with no delta review. (4) Only markers and
`merge-ready` set by the owner's account count. (5) One batch-tier gate per
merge batch, not per merge.

**Why.** Every merge moves main, so an early refresh is undone by the next
merge and costs a delta review. The maintainer's words, 2026-11-12: "we spend
a shit ton of tokens on uncessary deltas", and the question-dialog answer
"Refresh only at merge time". Owner-only counting is #1207 ("No, only ours
count").

**Considered and dropped.**
- Porting the planner as a Python script under `.claude/skills/`: dropped.
  The repo's tooling is Rust in `xtask` (one implementation on Windows, macOS
  and Linux, #748), and the reference script stays the maintainer's local file.
- Porting the planner with its `--json` output: dropped for now; the skills
  read the text. Easy to add later.
- Stacking a fix for the planner's wave-1 parent check on top of the port:
  kept, as a deliberate difference. The reference merged a child whose
  parent it had just deferred, which would silently pull in the parent's
  changes; the port defers the child too (`its parent #N is not in wave 1`)
  and a unit test pins it (`three_way_clash_defers_the_merge_and_its_child`).
- Rewriting the draft rule to allow any merge of main on a ready PR: dropped.
  Only a clean merge (empty remerge-diff) keeps the ready flag; anything else
  still goes back to draft.

**Cost to change later.** Low. The ordering is pure (`merge_plan.rs`, fake
merge oracle in tests); the git side is `merge_plan_git.rs`. The process
texts name the commands, so renaming the subcommand touches the skills and
AGENTS.md only.

**Not done here.** The announcement on #681 ("PROCESS_VERSION 31") is made
when this PR merges, by the merging session, per AGENTS.md "Process changes".
