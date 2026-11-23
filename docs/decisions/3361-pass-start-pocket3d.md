## #3361 - "Pass starts at" on 3D Pocket

**What was decided.**

- 3D Pocket gets the shared field `pass_start: machining2d::PassStart`
  (#3316) in `OpParams::Pocket3D` and in the engine's
  `Pocket3DParams`. There is no new type. The project key is the shared
  `pass_start` (`PassStartDto`): `auto` is never written, a file without
  the key reads Auto, and there is no version step. The digest writes
  the value only off Auto (`digest::write_pass_start`), so each earlier
  operation keeps its cache stamp. MCP uses the generic `edit_operation`
  (no kind-specific code). The page shows the shared row
  `Pass starts at` on the Passes tab.
- Inside and Outside force the ring order for the Waterline and Offset
  rings strategies. The 3D level calls the 2D ring law through the new
  `machining2d::level_ring_plan` (a thin door over the same
  `level_pieces` / `order` / completeness guard as 2D Offset). The 3D
  level's cutter-centre area is its own outline (the ladder erodes the
  target once), so the start side is read off the edges of `center`
  (offset 0). The 2D `ring_plan` keeps its exact arguments.
- Auto keeps today's order byte for byte: the Auto path of
  `level_ring_plan` is the same `plan_passes` call as before. No golden
  moves (`pocket3d_trochoidal_pocket.cl` is Adaptive and never reaches
  the ring law).
- Zigzag, and Auto (which cuts rows), have no ring order. The row stays
  enabled and keeps its value. A line under it says that the value is
  not used (`POCKET3D_NO_RING_ORDER`), and the toolpath carries the
  shared `PASS_START_ZIGZAG_NOTE` once. No move changes and nothing is
  refused.
- Adaptive (the HSM spiral) has no ring order either. It says so with
  `PASS_START_ADAPTIVE_NOTE` (once per path), and no move changes.
  Under Adaptive with Offset shells the level is cut by Offset rings,
  and those rings follow the parameter (the same law as Offset rings).
- The rest-closing ladder keeps its own order (`PASS_START_REST_NOTE`,
  once). The rotary wrap (a level with a seam) keeps Auto: its
  developed area ends at the seam, not at a wall. Rotary Offset and
  Multi-Axis Rough always pass Auto.
- Notes: the planner's warnings (`Pass start ...`) go on the path once
  each. A completeness-guard fallback is said for each level, with its
  Z, and carries `RINGS_FALLBACK` (an advisory strategy fallback).
- Room for #1448: the engine field has a doc line that says the start
  and exit points go beside it. No code for them.

**Why.** The maintainer asked for the option on each operation whose
passes or rings have an order (#3316). Waterline and Offset rings are
the 3D Pocket strategies with rings, and they use the same ring law as
2D Offset, so the same law and the same proofs apply.

**What was dropped.**

- A `plan_level` argument or a `ladder` argument for the side: dropped,
  because the side must reach each ladder entry (plain, rest, offset,
  avoid, open, trochoidal). The engine field reaches all of them.
- The start side for the Adaptive named fallbacks to rings
  (`rings_named`): each such fallback is on the rotary wrap, where the
  side stays Auto. The HSM planner's own internal fallback to rings
  (inside `adaptive_passes`) does not follow the parameter. That is more
  than a few lines, so it is the follow-up issue #3616 (P7).
- A disabled row for Zigzag and Adaptive: dropped, because the strategy
  can change after the value is set; a kept value with a visible line is
  clearer than a value that the page resets.
- An edit of the `camcraft-mcp` skill row: dropped from this PR, because
  the skill is a process file. The generic `edit_operation` already
  carries the key.
