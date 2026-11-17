## #1490 - Zigzag rest passes and separate stock-proof work

**What was decided.** Enable Zigzag for 2D rest machining. Scan rows over
the rest cutter-center area, inset by `min(0.01 mm, stepover / 4)`, then
close each original boundary. Keep each boundary as a separate pass.
The existing level planner proves the links and performs the transitions.

**Why.** Rows alone leave wall cusps. Closing passes cover that material.
The inset keeps row endpoints off the closing boundary, where numeric
rounding can produce crossings. It does not shrink the rest estimate or
move the closing boundary. The quarter-stepover cap preserves interior
rows in small rest areas. Analytic coverage pins check the cutter sweep.

**What was considered and dropped.** Removing the closing passes missed
287 residue grid points in the first fixture run. Rows that touched the
boundary produced six verifier crossings. A fixed 0.01 mm inset erased
the rows in a 0.02 x 0.015 mm pocket with a 0.001 mm tool, a 0.03 mm
previous tool, and a 0.0005 mm stepover. Its center-coverage pin failed.
The capped inset passed; restoring the fixed inset made it fail again.
Replacing Zigzag with Waterline would ignore the selected strategy.
Changing the plain-pocket scan would change unrelated paths. Neither is needed.

**Keep the existing trust contract.** Manual mode still trusts the entered
previous diameter, corner radius, allowance, and wall stock. Its path note
still states that trust. This change does not infer actual cleared space
or authorize new rapids through assumed cleared stock. Adaptive remains refused.

**Separate the remaining work.** The completion rule for #1490 permits
separate issues with recorded decisions. Three items need additional proofs:

- Adaptive, #2031: initialize engagement from conservative remaining stock
  at each depth. Prove the entry. Keep unknown cells occupied.
- Entry from cleared space, #2032: prove the full cutter footprint and
  descent volume clear. A clear tool axis does not prove a clear rim.
- Previous-operation stock, #2033: read current earlier cuts in order.
  Preserve actual depth, tool shape, allowances, and dependency invalidation.
  Refuse unknown or stale stock. Keep backend migration in #586/#637.

Each follow-up uses `P7`, the Issue Law default, and `effort:max` for new
geometry algorithms. Each also uses `enhancement`, `area:2d`, and
`source:agent`; entry and carried-stock work also use `area:sim`.

**Code placement.** Move the existing `zigzag_passes` algorithm unchanged
to `pocket_zigzag.rs`, with provenance. This keeps `pocket.rs` at 398 lines.
No new crate dependency is needed.

**Branch and overlaps.** Stack on #1817, which changes `rest_pocket`,
exports `rest_strategy`, and adds the operation parameter door. Merge it
first. PRs #2013/#1954 touch different declarations in `machining2d/mod.rs`.
PR #1738 touches another MCP skill section. Whichever merges second refreshes.

**Mutation baselines.** Restore old pocket code from `upstream/main` for
its whole-file mutation. Use `base-1817` for the old rest strategy:
`upstream/main` lacks the `rest_strategy` export the stack requires.
That upstream file would test a missing symbol, not the strategy refusal.
Separate closing-pass, zero-inset, and fixed-inset mutations test each
geometric decision. Each mutation failed; each restored version passed.
Final pins are `zigzag_rest_covers_corner_residue_without_gouges_or_unsafe_rapids`
(both directions and 1.5/4 mm steps) and
`tiny_rest_pocket_keeps_interior_rows_when_previous_tool_fits_nowhere`.

**Cost to change later.** Rows and closing passes can change behind the
same strategy without a stored-parameter change. Coverage, verifier, and
rapid-motion pins must still pass. A different inset needs proof against
boundary crossings and loss of interior rows. Adaptive and cleared-space
entry need their separate stock-proof contracts before trust can change.

**Literature.** Held's [Zigzag Pocket Machining](https://www.cosy.sbg.ac.at/~held/projects/pocket_zig/pocket.html)
describes parallel rows and boundary connections. His
[pocket geometry monograph](https://link.springer.com/book/10.1007/3-540-54103-9)
(LNCS 500, 1991) provides the geometry context. Yao, Gupta and Nau's
[cutter-selection paper](https://www.cs.umd.edu/~nau/papers/yao2003algorithms.pdf)
(Computer-Aided Design 35, 2003) provides the coverage context.
The capped inset, closing passes, and stock-proof contracts are our decisions.
No other CAM product's code or output was used.

**PR #2037 review refresh (2026-11-16).** Merge `upstream/main` at
`a49322b9`, preserving its move of the level-links choices tests and all
of its safety tests. Carry `rest: None` into the moved Pocket fixture and
main's new `first_router_part` fixture. Extend
`choices::links_round_trip_and_the_defaults_write_no_key` to assert that
both default and picked links still load a non-rest Pocket. These fixture
changes are compatible and keep the checks from both branches. Restoring
the old inline tests would duplicate main's moved tests; changing unrelated
product code is outside this review. Both alternatives were dropped.
Parent #1817 remains open and must merge first. Verification is recorded
separately in the PR comment after the checks complete.

**Shared-emitter conflict resolution (2026-11-16).** Keep both
`pocket_zigzag` and main's `pocket_emit` modules. Select the Zigzag rest
passes before shared emission on both rest paths. The selection and
emission changes are compatible; keeping either conflict side alone would
lose the selected strategy or main's entry and link safety checks. Keep
main's emitter unchanged rather than duplicating its emission logic.
