## #3460 - One heavy run for a merge wave, on the trial tree of the wave

**The maintainer's words** (issue #3460, 2026-11-22, given in the
question dialog):

> A wave of PRs that the planner groups together gets ONE heavy run on a
> trial tree of the wave (current `main` plus each PR of the wave). The
> result is recorded on each PR of the wave (tree hash, lanes, counts). A
> red step is attributed to a PR, that PR leaves the wave, and the run
> starts again for the rest.
>
> Rules that stay:
> - "Not run" is never a pass.
> - A PR that merges alone keeps the rule of today.
> - The release tier at the cut stays the final check.
>
> [...] Safety-floor PRs: the wave run covers them only if the trial tree
> contains their final head.

**Decided.** AGENTS.md "Heavy lanes at review" has two forms. The PR
form is the rule of #2568: the author runs `cargo xtask gate heavy` on
the final head. The wave form is new: the merging session runs it one
time on the trial tree of wave 1, with `--covers <PR>:<head sha>` for
each PR. The tool refuses a sha that is not a commit, a head that is not
an ancestor of `HEAD`, and a working tree with changes. It prints
`covers: #<PR> <sha>` and `tree: <HEAD^{tree}>` in the plan header and
under the summary, and writes `covers.txt` (header, steps, counts, wall
time, verdict) to the log directory. The `merge-ready` skill (step 4a)
makes the trial tree, posts one `<!-- heavy-wave: <tree hash> -->`
comment on each PR of the wave, and drops a PR with a red step. A
reviewer accepts a wave record only if it names the current head of the
PR. A wave-2 PR merges alone and keeps the PR form.

**Why.** One run of the motion and app lanes takes 60 to 70 minutes, and
one heavy build runs at a time on a machine. A wave of 9 PRs took about
12 hours with one run for each PR. The trial tree contains each diff of
the wave, so one run covers each PR. The ancestor check makes the record
true: a tree that lacks a PR's final head cannot name it. The clean-tree
check makes the tree hash the tree that the run tested.

**Taken unattended.**
- The rule text says "does not merge while not run on its final head"
  in place of "not ready while not run", because the wave run comes at
  merge time. The PR form keeps "before the PR is flagged ready".
- The reviewer keeps its duty of #2568 for a `P1` or `P2` PR
  (PR-reviewer step 5, pinned by
  `review_requires_targeted_heavy_tests_on_both_paths`). A wave
  record that names the current head takes the place of the author
  result. The maintainer decided the `P3` and lower case in #3467 (the
  entry below).
- `--covers` refuses `--only` and `--failed`, and the record has the
  planned step count beside the run count. A planned step that did not
  run gives `INCOMPLETE`, and a plan with no step gives `NO LANE` ("the
  wave reaches no heavy lane"). Only `PASS` covers a head. Reason: "not
  run" is never a pass, also for one lane of a wave.
- The record names the base (the merge base of `HEAD` and `--base`). If
  `main` moves before the merges, the planner runs again, and a PR whose
  trial merge is no longer clean leaves the wave.
- A wave-2 PR keeps its heavy result through a `SAME` or `MECHANICAL`
  refresh, as its review does ("A clean merge of main keeps its
  review"). A `DELTA` or `NEW CODE` refresh needs a new result.
- The attribution of a red step: the PR whose diff the red test reads;
  when that is not clear, the last such PR in the planner's order.
- The tool does not make the trial tree and does not post the record.
  The merging session does, by the skill. This keeps `xtask` free of
  `gh` writes and of worktree side effects.
- `gate_args.rs` holds the moved `parse_gate`, so that `main.rs` stays
  under 400 lines.

**Dropped.**
- Heavy runs for each PR only (the rule of #2568 alone). It is correct,
  but it costs about 12 hours for a wave of 9 PRs.
- No heavy run before the merge, with the cut gate as the only check. A
  red heavy test would then be found only at the cut, after the merge,
  and "not run" would act as a pass.
- A trial tree built from `git merge-tree` objects only (as
  `merge-plan` does). The heavy tier diffs the working tree, so the run
  needs a checkout.

**Cost to change later.** Small. The flag is additive; without it,
`gate heavy` works as before. The comment form is a skill text.

## #3467 - A review can owe the heavy result to the wave run (P3 and lower)

**The maintainer's words** (issue #3467, 2026-11-22, given in the
question dialog):

> Maintainer decision, 2026-11-22, given in the question dialog: **allowed for P3 and lower only.**
>
> - A review of a P3 or lower PR may set `merge-ready` while the heavy result says "not run - owed to the wave run before merge". The comment names the owed lanes.
> - The merge (not the review) waits for a green run that covers the final head, with the `--covers <PR>:<sha>` record of #3460.
> - A P1 or P2 PR keeps the strict rule: the heavy result on the head comes before the label `merge-ready`.
> - The coordinator refuses to merge a PR with an owed heavy result until the covering record names the head.

**Decided.** AGENTS.md "Heavy lanes at review" gets the stable rule
**Owed heavy result**. PR-reviewer step 5 has one bullet for each
priority. A `P3` or lower PR can get `merge-ready` with the fixed text
"not run - owed to the wave run before merge" and the list of owed
lanes. A `P1` or `P2` PR, or a PR with no priority label, keeps the
strict rule. The `merge-ready` skill (step 4a, item 11) does not merge an
owed PR until a green record names its final head. The `review-prs`
skill (2e) says the same. The test
`an_owed_heavy_result_is_allowed_for_p3_and_lower_only` pins both halves
in PR-reviewer step 5, AGENTS.md and the `merge-ready` skill.

**Why.** The review comes before the planner makes a wave. Without this
rule each reviewer runs the lanes, and the wave run of #3460 saves time
only for heads that change at merge time.

**Taken unattended.** A PR with no priority label is read as strict
(like `P1`/`P2`): an untriaged PR does not get the lenient rule.

**Dropped.** The lenient rule for each priority: the maintainer kept the
strict rule for `P1` and `P2`.

**Cost to change later.** Small: rule text and one wording test.
