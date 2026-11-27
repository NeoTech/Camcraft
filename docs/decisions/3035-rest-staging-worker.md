## #3035 - the rest grid's stock simulation runs in the generation job

**Decided.** `rest::prior_state` is split in two halves:

- `rest::stage_prior` does every world read. It reads the chain of
  operations ahead, the digest and the stock grid. It has the same order and
  the same refusals as before.
- `StagedPrior::simulate` cuts the cached moves ahead into the stock. It
  reads no world.

`collect_job` keeps the staged half in a new `GenJob::prior_staged` field.
`run_job` calls `GenJob::settle_prior` first, so the simulation runs on the
thread of the job. For an MCP `generate` and a GUI generation, that thread
is the worker. The session thread stays free to answer other calls.
`prior_state` keeps its signature and calls the two halves in sequence.

**Why.** The MCP worker of PR #3860 took the generator off the session
thread. But the staging of a rest operation (3D Pocket, Multi-Axis Rough,
and a 2D Pocket or 2D Adaptive whose rest reads the previous operations)
still simulated the program ahead on the session thread. On a large part
this simulation takes a long time, and no other call got an answer.

**Kept the same.** An `#319` distrusted grid still runs the simulation and
then drops the grid. A simulation refusal therefore still refuses the
operation, as before. The skip would be faster, but it would let an
operation pass that refuses today. A stored error surfaces at the same arm
statement as before. The corner-relief reader (`zones_for_op`) settles the
grid before the WCS yaw turn reads it, so its result does not change.

**Dropped.**

- A carried chain across setups (#318) stays eager. Its body comes from a
  per-thread cache (`carried_cache`). A move to the worker would make each
  job miss that cache.
- The stock grid (`first_held`) is still built at staging. It needs the
  world. It costs one allocation and no simulation.
- Verify's `PriorPlan` was already deferred (#96 review) and does not change.

**Cost to change later.** Low. `StagedPrior` is one type in
`crates/app/src/rest/staged.rs`; the job has one extra field.
