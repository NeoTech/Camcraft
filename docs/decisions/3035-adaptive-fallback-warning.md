## #3035 - An Adaptive fallback to Offset rings is a toolpath warning

**What was decided.**
- A new `Warning::StrategyFallback { at, areas }` (`camcraft_sim::verify`).
  It is advisory: `blocks_post` is false. `at` is the first cutting move.
  `areas` is the number of areas (a 2D pocket, a 3D level) that the Offset
  rings cut in place of the Adaptive plan.
- Generate (`operation/generate.rs`) counts the fallback notes with
  `machining2d::rings_fallbacks` and adds the warning when the count is
  more than zero. The op then reports `warnings` > 0 in MCP and in the UI.
- The note tail `" - Offset rings cut "` is now one constant,
  `machining2d::RINGS_FALLBACK`. The 2D door (`pocket_hsm`) and the 3D
  ladder (`machining3d::hsm::rings_tag`) both use it, so the count cannot
  drift from the note text.
- The rest pass and the offset shells are not counted. Their notes say
  "cuts Offset rings at the load step" by design, not as a fallback.
- The warning locates no single move (`warned_moves` gives `[None, None]`,
  like `RestUnjudged`). The UI folds it on its own key, 225.
- This PR is partial work on #3035 (`Refs #3035`). The run time of the
  three operations and the MCP server that serves nothing during a long
  generate stay open on the issue.

**Why.**
- The issue: six levels hit the budget and eight more crossed themselves.
  All of them went to Offset rings, but the op said `warnings=0`. A
  strategy that the user did not ask for cut most of the part. That must be
  visible in the warning count, not only in the notes.
- Advisory, not blocking: the rings are verified by the same laws as any
  other path. The output is safe. Only the strategy is different.
- One count for 2D and 3D: the two doors write the same note and have the
  same problem.

**Considered and dropped.**
- A blocking warning: it would hold the post of a safe program.
- A field on `ClPath` that the planner sets: it touches the `cl` crate and
  each generator. The notes already carry each fallback by name.
- A count per level or a list of Z values in the warning: the notes name
  each level and why. The warning gives the count and points to them.
- Split `verify/types.rs` (now past 400 lines): out of scope. The new
  variant adds about 12 lines, as `DrillUndersize` did.

**Cost to change later.** Low. One enum variant (serde name
`strategy_fallback`, additive in the project file), one count function, and
one call in generate.
