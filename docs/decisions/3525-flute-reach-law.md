## #3525 - A cut deeper than the flute length is named by its cause

Taken unattended, as the recommended option under the owner's standing
word (Autonomous Work Law). The coordinator recorded the decision on #3525:

> Maintainer decision (owner's standing word 2026-11-22: choose the
> recommended option while he sleeps). Options: (1) `FluteShort` is always
> advisory and is listed first; the existing shank and holder findings
> hold the post when the body does meet material; (2) keep the block for a
> wider shank and accept that the program of `t31` does not post; (3)
> block only when a shank finding exists also (the same result as (1), with
> more code). Taken: **(1)**. Reason: the diagnostic exists to name the
> cause; the collision checks decide if the post is held. The case of
> #3319 stays blocked by its shank findings, and it now names the cause
> first.

**What was decided.**

- A new law, `camcraft_sim::verify::flute_reach`, reads the lowest cutting
  tip of a path against the stock top. When that depth is larger than the
  tool's flute length, it gives `Warning::FluteShort` (MCP kind
  `flute_short`) at the first cutting move below the flute's reach. It is
  always advisory: `blocks_post()` is false. The shank and holder findings
  decide if the post is held.
- The operation layer (`crates/app/src/operation/flute_law.rs`) runs it at
  generation for 2D Pocket, 2D Adaptive, 2D Offset and 2D Contour:
  - always a note, FIRST in the notes, with the tool's name;
  - the warning, FIRST in the warning list, only when the shank or holder
    laws convicted the body on that path (`ShankHit`, `HolderHit`,
    `SweepHit`). Then it explains the findings after it.
- The texts state the facts (flute length, cut depth below the stock top,
  the shank or holder above the flutes goes into the cut), what MAY follow
  ("can rub or collide"; "the findings after this one name the moves where
  it meets material"), and what to do (a longer flute, or a smaller
  depth). They do not say whether the shank is wider than the cutter: the
  stated `ToolGeom` shank is widened to a holder profile in some tools (the
  `t31` tool reads a 17.5 mm shank radius over a 6.35 mm shank), so that
  claim cannot be made from it.
- The verifier's findings, the collision and rapid checks, and the text of
  `RapidInStock` (#3520) do not change.

**The first form, and why it was dropped.** The first form blocked the
post when the shank was wider than the cutter, and called that contact
certain ("is in the cut and meets the walls that the flutes leave"). The
quick gate disproved it: `tool_body_beside_shank::
t31_verify_judges_shank_at_6_35_below_38_1_and_catches_real_holder_hit` has
a 2D Contour with a 14.478 mm flute, a 25 mm cut, and a shank that passes
beside the wall; the verifier finds no collision. With the block, that
program no longer posted.

A second form (always advisory, always a warning) still broke `t31`
without a change to that test: it asserts that the clean program has no
finding at all. So the warning rides only beside a body finding, and a
clean path gets the note alone. This is option (1) of the decision for the
post, with the condition of option (3) for the warning list.

**Considered and dropped.**

- A refusal at generation (`GenError`), like the drill reach law: the user
  then sees no path and no findings to compare.
- Only a note in all cases: a note is not in the MCP findings, so an agent
  that reads the #3319 findings would not see the cause.
- A law inside `verify_bodies` for every kind: it would reach the 3D,
  turning, drilling and rotary kinds, whose depth is not "levels below the
  stock top".
- The tool's name in the warning text: `Warning` is `Copy` and a file
  format; the name rides in the note.

**Limits.** The depth is read against the stock's highest point. For a
stock with steps, the depth at one place can be smaller than the one that
the finding names. The collision laws still give the exact moves.

**Cost to change later.** The kind name `flute_short` is part of the
project file and of the MCP output. To judge more kinds, extend `judged` in
`flute_law.rs`.
