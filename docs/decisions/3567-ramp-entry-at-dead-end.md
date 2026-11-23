## #3567 - A later Outside pass enters by a ramp along the pass at a dead end

**The maintainer decision** (issue #3567, 2026-11-22, given in the
question dialog):

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
> - The workaround "Pass starts at: Auto" stays until the fix merges.

**The coordinator's decision by the owner's standing word** (issue #3567,
2026-11-22T23:24Z):

> Maintainer decision (owner's standing word 2026-11-22: choose recommended
> while he sleeps). Options: (1) accept the full-width ramp at the ramp angle
> and state it in the decision file, the PR and the changelog; (2) stop the
> work until the maintainer confirms; (3) find an entry that never cuts at
> the full width (it needs the reversed walk that the maintainer dropped).
> Taken: **(1)**. Reason: it is what "a ramp along the pass" gives by
> geometry, the angle limits the load, and the alternatives were dropped in
> the decision. If the maintainer wants a lower load, the ramp angle of the
> operation is the control.

**What the ramp is, in plain words.** A dead end has the part wall behind
the tool and uncut stock in front of it. The ramp is an entry INTO that
uncut stock: the tool goes down along its own pass at the ramp angle
(3 degrees when the operation sets none), out and back along the start of
the pass (the existing zig-zag of the ramp fallback). Where the ring cut
before does not reach the dead end, the ramp cuts a slot at nearly the full
tool width, at the ramp angle. After the ramp, the pass cuts at one
stepover. The ramp angle is the control of the load: a smaller angle gives
a longer, lighter ramp. The late-entry pin reads 0.00 mm2 because it does
not count ramp moves, not because the ramp cuts no stock.

Readings on the band of the pins (stock 100 x 40, part R 15, tool D8,
stepover 4, depth 3, no ramp angle set):

- 8 dead ends. Each ramp runs at 3.000 degrees from Z 2.0 (the top plus
  the 2 mm plunge height) to Z -3.0, 95.41 mm along the pass (one out and
  back leg of 47.7 mm). The pass then cuts that stretch again at the floor.
- Where each ramp starts, about 25 percent of the tool disc (about
  12.6 mm2 of 50.3 mm2) is uncut stock on the 2-D grid of the pins. While
  the ramp travels, the width of uncut stock on the leading half of the rim
  reads 4.780 mm at most (the stepover band). The coordinator's comment
  gives about 83 percent (41.6 mm2 of 50.3 mm2): 41.6 mm2 is 332.88 mm2 / 8,
  the stock that each helix of main cut over its whole swept orbit, not the
  stock in the disc at the start. On this band the ring before reaches the
  dead end; the full-width slot is the case where it does not.
- The late-entry pin: 332.88 mm2 on main, 0.00 mm2 with the ramp.

**The slow pin is a test-speed problem, not a block.** The verification of
`fig8_pocket_outside_starts_each_level_at_the_stock_edge` is slow. This is a
test-speed problem, filed as #3587 (`P5`). The verifier speed is filed as
#3588 (fix PR #3595, draft). Neither issue blocks #3577.

Readings on head `d41c2b76a`:

- `pocket_pass_start`: 5 passed of 6. The sixth test,
  `fig8_pocket_outside_starts_each_level_at_the_stock_edge`, did not
  complete. It stopped at 29 min 23 s. The earlier run tripped the 15-min
  stall watchdog at 961 s.
- Every other suite is green:
  - motion `pass_start_engagement` 6/6 and `pass_start_levels` 7/7;
  - motion lib `machining2d` 267 passed and 10 ignored, `level_ramp_entry`
    4/4;
  - app `offset_pass_start` 4/4, `golden_cl` 33/33, `conformance` 7/7,
    `adaptive_pass_start` 4/4, `bore_pass_start` 3/3;
  - the catalogs 37/1/1;
  - clippy clean;
  - the light lane 2 twins green.

**What was decided by the workers, inside those decisions.**

1. **The plan marks the dead ends.** `level_order::order` marks each later
   pass of an Outside plan that no lead-in reaches (`LevelPlan::ramp_starts`).
   The ladder enters only a marked start by the ramp
   (`level_ramp_entry::dead_end_ramp`, called first in
   `ladder_links::pass_link`). The first entry of each level and the ring 0
   pieces keep the entry of `main`: a helix or a plunge into solid stock
   there is the normal first entry and is outside this issue. The Auto-order
   plan after a completeness-guard fallback has no marks, so it stays pure
   Auto (pinned byte for byte). The first head of the PR read the dead end
   from `plunge_law::into_solid` at each pass link; that changed ring 0
   pieces and the guard-fallback plan too, and scanned all earlier moves at
   each pass link.
2. **The ramp is the existing ramp fallback.** `descent::ramp_fallback`
   plans the zig-zag along the first cut at the entry's ramp angle, 3 degrees
   (`links::DEFAULT_RAMP_ANGLE`) when the operation sets none, with the
   entry's maximum ramp length as the leg limit. No second ramp law.
3. **No silent plunge.** The plan checks each dead end before it is kept
   (`level_ramp_entry::too_short_at`, at the deepest descent of any cut
   level). A dead end too short for the ramp sends the area to the Auto
   order with `StartSide::ramp_fallback_note`, which carries
   `RINGS_FALLBACK`, so the op raises `Warning::StrategyFallback`. If the
   ladder still cannot plan the ramp, it names that on the path.
4. **A replaced entry is named.** When the ramp stands in for the user's
   Plunge, Helix or Arc entry, the path says so once, in the #2265 stand-in
   form (`level_ramp_entry::stand_in_note`).
5. **The scope.** 2D Offset and 2D Pocket with "Pass starts at: Outside" and
   the Waterline or Offset rings strategy (the main pocket path and the
   "Do not touch" path). Auto, Inside, Adaptive, Zigzag and rest machining
   mark no dead end.
6. **The pin is narrowed, not removed.** `late_entry_cut` counts each entry
   move that is not a ramp along the pass. A ramp is a feed no steeper than
   the ramp angle plus 0.05 degrees whose centre stays within 0.02 mm of a
   floor feed all along. A plunge, a helix and a steep straight feed count.
   The pin also asserts that a ramp ran.

**Dropped.**
- A planar lead chord across one stepover (the held option A on
  `issue/3567-outside-later-entry`, commit `12c0fd125`): it feeds into the
  crescent at the full tool width and breaks the shape-mask law.
- The Auto order at each dead end: the Auto order slots the band.
- A reversed walk at a dead end: it breaks climb milling.
- The dead end read from `into_solid` at every pass link (the first head of
  the PR): see item 1.

**Cost to change later.** Low. The marks are one list on the plan and on
`Motion`; the entry is one function in `machining2d/level_ramp_entry.rs`.
To make the ramp the entry for Inside too, mark Inside dead ends in
`level_order::order` and run the Inside pins.
