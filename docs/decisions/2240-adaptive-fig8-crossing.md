## #2240 - Hold the crossing law on all arc-fitted pocket strategies, including OffsetRings and Waterline

**Decided.** In `crates/motion/src/machining2d/pocket_emit.rs` (`emit_pocket_passes`),
hold the posted-path crossing law (`post_crossing_free`) whenever
`hold_crossings` is set or the resolved strategy fits pass arcs
(`PocketStrategy::Adaptive`, `PocketStrategy::OffsetRings`,
`PocketStrategy::Waterline`), and record fitted arc counts in `proof.arcs`
from the `fit_arcs` branch so `ProofTally::note()` reports accurate totals
when a crossing arc returns to chords.

**Why.** On `fig8.step`, 2D Adaptive in tight stock or stock excluding the
top face refuses the HSM planner and falls back to `PocketStrategy::OffsetRings`.
Clipper2 integer quantization (`1e-6 mm` grid) on STEP-faceted circles can
produce micron-scale vertex steps that split `fit_arcs` across `sag_bound`
into `[Arc, 1 um Line, Arc]`. Because the arc fit and verifier tolerances
exceed the micron gap, adjacent arc chords properly cross in XY at shared
depth and block posting. `emit_pocket_passes` previously guarded only
`PocketStrategy::Adaptive || hold_crossings`, leaving `OffsetRings`
(direct or Adaptive fallback) and `Waterline` unguarded unless invoked from
`rest2d`. Running `post_crossing_free` preserves non-crossing programs and
refuses only true crossing arcs back to their plan chords.

**Considered and dropped.**
- Altering `arcs::fit_arcs` tolerances or adding aggressive collinear/short-edge
  vertex pruning in `geo2d`: risks disturbing geometry and byte-identity on
  clean programs, whereas `post_crossing_free` intervenes only when a posted
  arc actually crosses another feed.
- Restricting the guard to Adaptive fallback or changing `pocket_entered` in
  `pocket.rs`: direct `OffsetRings` and `Waterline` pockets share the same
  `fit_arcs` path over quantized loops, and touching `pocket.rs` creates
  unnecessary churn beside open PRs while `hold_crossings` remains active for
  `rest2d`'s blended `Zigzag` passes.

**Cost to change.** One `matches!` arm and two tally lines in
`emit_pocket_passes` (`crates/motion/src/machining2d/pocket_emit.rs`).

## #2240 - Preserve the opening rapid during crossing retries

**Decided.** Make `PathBuilder` cloneable. Use a shared
`crossing_free_from` factory to copy the input builder for each
`post_crossing_free` reconstruction. The 3D `crossing_free` caller keeps
its existing `PathBuilder::new(start)` contract.

**Why.** Reconstructing from the cursor alone loses the pending opening
rapid. At entry X0 Y0, that loss lets the path start with a plunge. Copying
the builder retains its approach state, notes and other recorded state
on the first attempt and each retry.

**Considered and dropped.** An unconditional `PathBuilder::approached`
would add a rapid to callers that supply a builder without that contract.
Disabling the crossing guard would remove the safety check required by
this fix.

**Cost to change.** Remove the `Clone` derive and replace the private
factory with another way to preserve the input builder state.
