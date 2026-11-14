## #1605 - The default-cap STEP pin gets its own test binary; the product is unchanged

**Decided.**
- The ~50 s default-cap pin `a_face_that_never_converges_is_refused_by_name_within_the_budget`
  moves from `crates/app/tests/import_step_guard.rs` to a new binary,
  `crates/app/tests/import_step_guard_default_cap.rs`. No test is removed or weakened.
- The earlier branch commit stays: the two evaluation-capped pins keep a
  120 s hang barrier. It is a generous barrier, not the fix.
- No product code changed. The refusal depends only on the input and the
  evaluation cap; no wall time or shared counter takes part.

**Why.**
- truck tessellates on rayon's one global pool. The default-cap import holds
  pool workers for about 50 s. Cheap tests in the same process queue behind
  it. Measured: with the two tests alone in one process, the may-jump pin
  (0.02 s when run alone) finished after 79 s; under extra CPU load, three
  pins passed 60 s only after the heavy one ended.
- Cargo runs test binaries one after another, so the cheap pins no longer
  share a pool with the heavy one. 30 parallel runs of the cheap binary,
  with the heavy binary running at the same time, passed (0.9 s each).

**Considered and dropped.**
- Only raising the barrier: hides the queueing, and the wait stays.
- A private rayon pool in the importer: a product change for a test problem.
- `#[ignore]` on the heavy pin: it would drop a pin from the batch gate.

**Not reproduced.** One earlier report of an assertion failure (not the
barrier) on the may-jump pin was not seen in 30+ runs here.
