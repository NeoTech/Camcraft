## #3556 - Targeted tests at review, the full heavy lanes at the cut

**Decided.** The maintainer took the option "Targeted tests, heavy at the
cut" (question dialog, 2026-11-22; the comments on #3556 and #2568). The
process files follow these six points:

1. At review a PR needs only its own pinning tests, the touched test
   binaries (the quick plan), the example catalog suites by name where a
   post or a toolpath changed, and the light lane (`cargo xtask gate light`).
   A red light run is only a note on the PR. It is not a blocker by itself.
   The full heavy steps of the red twins run in the cut gate. They do not
   run locally before the cut.
2. No full heavy lane for each PR. A diff that reaches a heavy lane is not
   "not ready" for that reason. No PR is "owed heavy".
3. The full heavy lanes run only at the cut, in the cut gate on the cut
   runner.
4. A `P1` PR under the safety floor (wrong output) gets one targeted heavy
   step: the named heavy tests of the changed code. It never gets the whole
   lane. `P2` and lower get none.
5. The heavy lock stays for the targeted step. The lock is a local rule of
   this machine, not a process file text, so this PR adds no text for it.
6. `cargo xtask gate heavy` and its `--covers` wave form stay as commands.
   They are not a condition for "ready" or for a merge.

The maintainer's words on #3556 named the post-merge runner. The text now
names the cut gate only, because `.github/workflows/post-merge.yml` runs
only the batch tier (`cargo xtask gate batch`), and the batch tier has no
release-mode heavy lanes.

The texts changed: `AGENTS.md` ("Heavy lanes at review", "The light lane",
the quick pass, "When PRs merge"), the `gate`, `work-issue`, `review-prs`
and `merge-ready` skills, and `.claude/agents/PR-reviewer.md`. The rule
name **Owed heavy result** stays in the files as a retired rule, so that
its citations still resolve.

**Why.** The maintainer's words: "Why are we doing heavy builds locally at
all? should be small targeted shallow tests". On 2026-11-22 the team merged
7 PRs in 3 hours, and one heavy run takes 50 to 140 minutes. A heavy run for
each PR, or a covering wave run before each merge batch, held the merges
for hours on a shared machine. The cut gate already runs the full heavy
lanes on the merged tree.

**Dropped.**
- A heavy run for each PR (#2568, the PR form): too slow for the merge rate.
- The covering wave run as a merge condition (#3460, and the owed result of
  #3467): it still held each merge batch for one heavy run. The command
  stays as an optional tool.

**Cost to change later.** Low for the text: the rule is in one `AGENTS.md`
block and five process files. To make heavy results a merge condition again,
restore the #2568 / #3460 / #3467 texts from the git history. The xtask unit
tests that pin the process text (`xtask/src/heavy_plan_tests.rs`) must follow
the text in each direction.
