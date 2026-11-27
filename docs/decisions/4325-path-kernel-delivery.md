## #4325 - Path kernel delivery: merge to main first, then switch the kinds one at a time

Part of #4023. This record changes how the path kernel gets to `main`. It
changes no stage, no law and no code. The plan comment on #4023 is
https://github.com/NeoTech/Camcraft/issues/4023#issuecomment-7215353049.

### Plain summary

This part has no code names. It says what a machinist sees.

- **The first merge.** The new path planner goes into the program that you
  use, but it is off. Each operation keeps the old planner. Nothing
  changes at the machine: each program is the same as before.
- **Each switch.** Then the operations move to the new planner one at a
  time, 2D Pocket first. When one operation moves, only the paths of that
  operation change: the new planner makes them. The paths of all other
  operations stay the same.
- **Before the first switch.** The maintainer gets a question before the
  first operation uses the new planner for users. It says what changes for
  users and who approves the switch.

### What was decided (2026-11-27)

The maintainer's words: "well then we should preemptive bring it up to
main, and merge it without implementing a switch over, making sure it
doesnt break and then start switching over.." (2026-11-27).

His answers in the question dialog of the same day:
- the operations switch one at a time, each in its own PR on `main`;
- the first merge into `main` does not wait for 2D Pocket;
- the coordinator session merges the one PR from `side/path-kernel` to
  `main` when an independent review is clean and the gates are green.

The new delivery:

a. **The kernel merges to `main` with no switch-over.** `side/path-kernel`
   is brought up to `main` (a sync PR). Then one PR takes it to `main`.
   The planner switch stays, and its default is the old planner for each
   kind. With `CAMCRAFT_PLANNER=new`, a kind that is not moved is refused
   by name. It never falls back to the old planner without a note.
   - The PR from `side/path-kernel` to `main` uses `Refs #N`, not
     `Fixes #N`, so that no unfinished issue closes.
   - It merges with a merge commit and without `--delete-branch`.
   - The coordinator session merges it when an independent review is
     clean on its head, and the batch gate, the heavy `motion` and `app`
     tests and the example catalog suites are green.
   - The Pocket recipe (#3752) is in it only if it is reviewed by then.
b. **Each kind switches in its own PR on `main`**, when the kind passes its
   rule tests (its K7 law checks). 2D Pocket is the first. The old planner
   stays the default of a kind until the switch PR of that kind.
c. **The old planner of a kind is removed after its switch**, on `main`.
d. **#4260, #4272, #4300 and #4236 must be closed before the switch of each
   kind that they touch.** Before this change they had to close before the
   one switch-over PR.
e. **Before the first kind becomes the default for users, the maintainer
   is asked.** The question says what changes for users, which of the four
   issues of (d) the switch needs, and who approves and merges it.

An issue of the kernel closes with the switch PR of its kind, or with its
own work when that work is complete on `main`. Until then it keeps
`in-review` after its sub-PR merges into the side branch.

### Why

- The side branch moves away from `main` with each week. Each sync costs a
  review and a batch gate, and the risk of a conflict grows. A merge with
  the default at the old planner puts the kernel on `main` at no risk to
  posted programs.
- One switch for 16 kinds puts all the changes of all the programs in one
  PR. A switch for each kind gives a small list of changed programs that a
  reviewer can read, and a user sees one change at a time.
- A kind that is ready does not wait for the slowest kind.

### What it replaces in the K0 record

The K0 record is `## #4024 - Path kernel K0: ...` in `RECOMMENDED.md`
(gathered there at the v0.27.0 cut). This record replaces these parts of
it. The cut does not edit `RECOMMENDED.md` again; read the K0 record with
this record next to it.

- **Decision 5 (the maintainer's): replace the whole planner in one go.**
  Replaced by (a) to (c). "No change at the machine until the switch-over.
  Then all the fixes arrive at once" becomes: no change at the first
  merge; then each kind changes at its own switch.
- **Plain summary**, the paragraph "Then one change switches all of 2D
  milling and 3D Pocket to the new planner at once." Replaced by the plain
  summary above.
- **Migration order (one-go build on the side branch)**: "Nothing reaches
  `main` before the switch-over PR", and the steps P, SW and RM. New
  steps: the sync PR, the PR from the side branch to `main` with the
  default at old, then for each kind its proof, its switch PR and its
  removal PR. The K6 recipes can merge into the side branch before the
  first merge, or go to `main` after it.
- **Side-branch plan**, "Sub-PRs": "The switch-over PR lists `Fixes #N`"
  and "keeps `in-review` until the switch-over PR merges ... closes at the
  switch-over". Replaced by `Refs #N` on the PR to `main`, and the close
  rule above.
- **The one switch**: "Default: `Old` on the side branch until the
  switch-over PR, which sets `New`." Replaced by: the default is `Old` for
  each kind on `main`, until the switch PR of that kind sets `New` for
  that kind only.
- **The switch-over proof**: one proof for all 16 kinds on the side branch
  head. Replaced by one proof for each kind on `main`, in its switch PR
  (the parts that stay are listed below).
- **The switch-over PR** ("one PR from `side/path-kernel` to `main` ...
  sets the default to `New`. The maintainer approves it himself ... It
  merges at a minor cut. After it, a second PR on `main` removes the old
  planners and the switch"). Replaced by (a), (b), (c) and (e).
- **Defects in the old planners meanwhile**: "Its issue stays open and
  closes with the switch-over PR." Replaced by: it closes with the switch
  PR of its kind.
- **Answered by the maintainer**, revision 2, items 6 and 7. Item 6 (the
  heavy `motion` and `app` steps run one time before the switch-over) now
  applies to the PR from the side branch to `main`. Item 7 (the maintainer
  approves the switch-over PR himself) is replaced by (a) for the merge
  and by (e) for the first switch.
- The earlier switch-over conditions on #4023 (comments of 2026-11-27,
  01:59 and 06:11 UTC): #4260, #4272, #4300 and #4236 now hold for the
  switch of each kind that they touch, not for the merge into `main`.

### What is dropped

- The replacement "in one go".
- One switch for all 16 kinds.
- One switch-over PR, and one removal PR for all old planners.
- The maintainer's own approval of the PR from the side branch to `main`
  (the coordinator session merges it by his answer of 2026-11-27).

### What stays

- The stages K1 to K5, the recipes (K6), the law suite (K7) and the laws
  of #4023.
- The crate `camcraft-pathkernel`, and the planner switch with
  `CAMCRAFT_PLANNER`.
- The side branch `side/path-kernel` for the kernel work, with its rule
  **Side branch** in `AGENTS.md`, its sub-PRs and its syncs.
- The scope of phase 1 (16 kinds) and the later phases.
- The proof for each kind, now in its switch PR:
  - its law checks (K7) are green;
  - the readings on the test parts, old against new (the disc of #3934,
    the dog bone of #3741 and `testdata/models/fig8.step`, with Inside,
    Outside and Auto);
  - each changed program of the example catalog has a one-line reason;
  - the verifier verdicts, the gouge checks and the collision checks are
    clean for each new program.

### Cost to change later

Low. No code depends on this record. A return to one switch for all
kinds needs only a change of the default in one place. A kind that is
switched can go back to the old planner while its old planner is not
removed.
