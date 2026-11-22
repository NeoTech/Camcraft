## #3535 - post-merge.yml: step limit of 75 minutes

**Decided.** The step "Batch gate tier" of `.github/workflows/post-merge.yml`
has `timeout-minutes: 75` (was 45). The check in "Gate summary" reads 74
minutes or more (was 44) as a step timeout. The job limit of 90 minutes, the
15-minute watchdog of `xtask` and all triggers stay. The pin in
`xtask/tests/post_merge_stall.rs`, `AGENTS.md` ("Runner stops") and the
`runners` skill use the new number.

Maintainer decision: step limit 75 min (owner, question dialog 2026-11-22)

**Why.** PR #3363 added the dependency `wgpu` and the crate `camcraft-gpu`.
Each runner must build `app`, `gpu` and `ui` from a cold state one time. That
build did not finish in 45 minutes (runs 45045638947 and 45046106061), so no
test ran and the issue #3522 opened with no `FAIL` row.

**Dropped.**
- Keep 45 minutes: each cold runner would time out again.
- Warm each runner by hand: it needs work on every runner after each new
  dependency, and a new runner starts cold again.

**Cost to change later.** Change the same four places: the step limit, the
summary check, the xtask pin and the two process texts.
