## #2528 - Drill point length follow-ups: a correction to the #2441 entry, and three file splits

**What was decided.**

1. *A correction to the #2441 entry in `RECOMMENDED.md`.* Item 3 of the
   issue names two wrong statements in that entry ("Project v68
   migration"). Issue branches never edit `RECOMMENDED.md`, and a PR edits
   only its own file here, so this file records the correction. A reader
   of the #2441 entry reads these two points in its place:
   - *What PR #2504 changed in the examples.* The entry says the migration
     keeps "existing saved projects and snapshot goldens
     (`examples/first-part.camproj`) byte-identical". That file did stay
     the same. But the PR rebuilt two other committed files:
     - the catalog case `examples/catalog/2d/first-part` (`.camproj`,
       `.nc`, `README.md`): its four through holes changed from
       `G73 ... Z-1. ... Q18.` to `G83 ... Z-2.803 ... Q6.`;
     - `examples/first-router-part.camproj`: saved again at project
       version 68 with `"break_through_from": "full_diameter"`, and its
       through-hole depth moved from -1.0 to -2.802582.
     The v68 migration keeps a *loaded* pre-v68 file on `Tip`. It does not
     keep a file that a PR builds again.
   - *What an MCP or script patch does.* The entry says "newly created
     operations and MCP/script patches default to `FullDiameter`". Only a
     new operation does: `add_operation` (and the script verb on it) starts
     from the kind's seed, which is `FullDiameter` (`kinds.rs`), and then
     applies the patch. `edit_operation` applies a JSON merge patch
     (RFC 7386, `mcp/edit.rs`) over the operation's stored parameters, and
     these always write `break_through_from`. A patch that omits the key
     keeps the stored value: an operation that a pre-v68 file migrated to
     `Tip` stays on `Tip`.

2. *Three file splits (item 4), structure only.* Each file is split
   along one domain seam, and the `#[test]` counts do not change (app lib
   322, motion lib 777):
   - `crates/app/src/stock_sim.rs` (469 lines): the program plan
     (`Planned`, `UndercutPlan`, `bull_corner`, `plan_program`,
     `plan_program_until_ungenerated`, `plan_ops`) moves to
     `stock_sim_plan.rs`. `stock_sim` re-exports the four names that other
     modules use, so no caller changes.
   - `crates/app/src/stock_play.rs` (409 lines): the move table and the
     gathered inputs (`PlayIndex`, `PlaySeed`) move to
     `stock_play_index.rs`, re-exported by `stock_play`. Their fields
     become `pub(super)`, because `FieldPlay` and `stock_play_head` read
     them.
   - `crates/motion/src/drilling/tests.rs` (508 lines): the five hole
     recognition tests and their `rect` and `plate` helpers move to
     `tests_recognize.rs`. The shared helpers (`drill_tool`, `circle`,
     `circle_cw`, `ring_walls`) stay in `tests.rs`, so the other test
     modules keep their imports.

**Why.** A decision record is what the next agent reads before it changes
the code. A wrong statement there leads to a wrong change: for example, a
"fix" that makes `edit_operation` reset `break_through_from`.

**Considered and dropped.**

- *Edit the #2441 entry in `RECOMMENDED.md`.* `docs/decisions/README.md`
  forbids it on an issue branch: each such edit conflicts with every other
  open PR.
- *Change `edit_operation` so that it agrees with the old text.* The code
  is correct: a patch that changes one field must not move another field.
  The text was wrong, not the code.

**Cost to change later.** None for the correction: it is text only. The
splits move code and change no behaviour; a later seam can move it again.
