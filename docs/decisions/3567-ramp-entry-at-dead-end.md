## #3567 - 2D Offset Outside: a dead-end later pass enters by a ramp along the pass

**Maintainer decision** (issue #3567, 2026-11-22, given in the question
dialog):

> At a dead end of a later pass of 2D Offset with "Pass starts at: Outside",
> the tool enters with **a ramp along the pass**. Option 1 of the worker's
> comment.
>
> - The ramp lowers the tool while it travels along the pass. The cut
>   engages in a gradual way.
> - The late-entry pin is narrowed to count only plunges and helices into
>   stock. A ramp along the pass is not one of them.
> - Climb milling stays. The full-width rule (no flat feed wider than one
>   stepover) stays.

**What was decided (by the worker, inside that decision).**

- A dead end is a pass of the Outside start-side plan after ring 0 for
  which `lead_in` finds no landing on a cut walk (`level_order::order`).
  The plan marks its start (`LevelPlan::ramp_starts`). Only Outside marks
  starts; Inside, Auto, Adaptive, Zigzag and rest machining mark none.
- The ladder's pass link (`ladder_links::pass_link`) enters a marked start
  below the material top by the existing ramp fallback of the 2D strategies: the zig-zag along the
  first cut (`links::plan_zig_capped` / `push_zig`, `descent::zig_from`).
  No second ramp law: the angle is the entry's ramp angle, and 3 deg
  (`links::DEFAULT_RAMP_ANGLE`) when the operation sets none; the leg
  limit is the entry's maximum ramp length. The ramp starts at the top of
  the material at that level plus the plunge height, as the existing
  restart does.
- The zig-zag lands on the pass start at depth, and the pass then walks
  the ramped stretch again at the floor. So the floor under the ramp is
  cut, in the climb direction.
- The ramp comes before the user's helix or plunge restart (#2265) at a
  marked start. The ladder does not scan the cut moves again to prove the
  column solid (`plunge_law::into_solid`): the mark is the plan's proof
  that no cut of the level reaches the start, and the scan of every
  earlier move at each mark made the fig8 Outside pocket test run for
  more than 10 minutes in debug. A ramp over a column that is cut after
  all is harmless.
- The fallback law: the plan checks each dead end before it is kept
  (`level_ramp_entry::too_short_at`), at the deepest descent of any cut
  level (the material top of that level plus the plunge height, down to
  the level; levels that cannot be read count as too short). A dead end too
  short for the ramp sends the area to the Auto order with the named note
  `StartSide::ramp_fallback_note`, which carries `RINGS_FALLBACK`, so the
  op raises `Warning::StrategyFallback`. If the ladder still cannot plan
  the ramp, it names that on the path (`entry: the dead-end pass ... is
  too short for a ramp along it`) and posts the ordinary pass link.
- The pin `late_entry_cut` counts the stock cut by every move that is not
  a floor feed and not a ramp along the pass. A ramp is a feed no steeper
  than the ramp angle plus 0.05 deg whose centre stays within 0.02 mm of a
  floor feed all along. A plunge (90 deg) or a helix (its orbit leaves the
  pass) is counted. Three pins guard the ramp itself: its angle, its width
  of cut (the engagement law of `pass_start_levels`, one stepover plus a
  quarter radius), and the floor under it.

**Readings (band of #3316: D8, stepover 4, depth 3, no ramp angle set).**

- The late pin: 332.88 mm2 on main, 0.00 mm2 with the fix.
- 8 dead ends, each ramped at 3.000 deg from Z 2.0 (top 0 plus the 2 mm
  plunge height) to Z -3.0: 95.41 mm of travel along the pass, one out and
  back leg.
- The widest cut during the ramps is 4.780 mm, under the 5.375 mm limit.

**Considered and dropped.**

- Option A, a planar lead across one stepover to the nearest cut walk: it
  feeds 8.0 mm wide into the stock crescent and breaks the full-width
  rule (three `pass_start_levels` pins red). Held on
  `issue/3567-outside-later-entry`, not for merge.
- Option B, the Auto order at each dead end: the Auto order slots the
  band at the full tool width (same pins red), and the pin is refused.
- A reversed walk at a dead end: it breaks climb milling, which the
  decision keeps.
- A one-way ramp in the climb direction only, then a separate closing
  pass over the ramped stretch: a second ramp law beside the zig-zag. The
  zig-zag is the existing law, and the pass itself closes the floor.
- A new ramp parameter for this entry: the entry's ramp angle and maximum
  ramp length already reach the ladder.

**Cost to change later.** Small. The marks are one list on the plan and
on `Motion`; the entry is one function (`level_ramp_entry::dead_end_ramp`).
A different entry (for example a one-way ramp) replaces that function. The
Auto path, Inside and Adaptive do not read the marks.
