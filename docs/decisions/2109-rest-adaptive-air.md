## #2109 - rest Adaptive: hold air feed and retracts to a budget

**What was decided**

1. **Back along the last pass instead of round the walls.** A new way
   of reaching a run (`leads.rs`, `retrace`): the tool goes back along
   the pass it has just cut. It follows the very chords, reversed, so
   the verifier's exact-sign crossing law sees a collinear retrace and
   no crossing. It stops at the new run's start where that pass went
   over it; otherwise it goes on to that pass's own wall landing and
   along the wall to the new start. Every way of reaching a run is now
   offered stay-down first, least air first; the retracts keep their
   order. The old stretch (ride in from the area's first landing, out
   to its first exit, back on one chord) is still offered, and it is
   what remains when a retrace fails its checks. A retrace is at most
   8 tool diameters long. Like every other lead, it is asked the
   centre-area law, the crossing law and the load walk before it is
   kept. It is a feed at the cut depth, over chords this depth has just
   cut, and never a plunge, so the entry guard is untouched.
2. **An area is planned in cells at most 4 tool diameters wide**
   (`peel.rs`, `TILE_PER_D`). Before this, the neck fixture's one area
   (both mouths of the neck and the material between them) ran its
   phases over the whole area. Each phase then cut one mouth, then the
   other, with a retract between. Now each cell runs all its phases
   before the next, nearest, cell. Corner areas (under 16 mm) stay one
   cell.
3. **A travel budget of its own, refused by name** (`budget.rs`).
   - Feed out of cut is at most 3.0x the feed in cut, per depth.
   - Retract hops are at most one for each rest area after the first,
     plus the plain Adaptive's `HOP_BUDGET` (8).
   - Past either limit: "rest Adaptive at Z... would feed X mm out of
     cut for Y mm in cut (over 3.0x)" or "... would reach N passes by
     a retract (over M ...)", followed by "- choose Waterline or Offset
     rings".
   - The yardstick is what the passes remove. A feed chord is in cut
     where it clears, on average, at least a graze (0.5% of D) of the
     rest material planned to be cut (the film excluded) on the
     conservative raster. Everything else is air.
   - The figures go into each depth's note: "... mm in cut, ... mm out
     of cut, N retract hops".

**Why**

- **Why not the plain Adaptive's 1.5x.** Under the no-crossing law a
  stay-down rest pass can only come back over chords it has already
  cut. In a corner, each pass's return therefore costs about one pass
  of air.
- **Why not the plain Adaptive's yardstick (`lift::travel`, contact
  over the bite).** It reads a ride along a wall as cut, because the
  wall stands as material in its contact reading. The plan before
  this change read 402 mm in cut against 194 out of cut there, inside
  1.5x, while it fed 4x air. A budget on that reading would bind
  nothing.
- **Why one hop per area.** Each rest area is cut off from the others
  by material, so it takes an entry of its own.

**Readings** (debug build, 4 mm tool after a 12 mm tool, two depths;
"test" = the independent column-stock measure of
`tests/rest_adaptive`, cut / air feed / rapid in mm for both depths;
"planner" = the budget's yardstick, one depth)

| Fixture | Before (test) | After (test) | Plunges a depth | Planner in / out of cut, hops | Moves |
|---|---|---|---|---|---|
| 60 x 40, 3 mm corners | 224 / 967 / 360 | 328 / 562 / 359 | 4 -> 4 | 117 / 479, 3 -> 117 / 329 (2.8x), 3 | 3216 -> 4064 |
| 70 x 50, island, 2 pillars | 250 / 955 / 450 | 388 / 485 / 452 | 4 -> 4 | 121 / 482 -> 121 / 316 (2.6x), 3 | 3288 -> 4160 |
| two bays, 10 mm neck | 954 / 3445 / 1337 | 1438 / 1979 / 545 | 37 -> 10 | 474 / 1726, 36 -> 467 / 1241 (2.7x), 9 | 14850 -> 17840 |

- **Cut length.** The test's cut figure rises because a retrace runs
  along the wall the last pass left, and that wall contact reads as
  cut. The planner's yardstick, which counts material removed, does
  not move.
- **Air time.** The neck's rapid travel falls with its retracts.
- **Planning time.** About the same: neck 34 s, corners 8 s.
- **Load.** The neck's worst measured load is 15.38% of D, the
  planner's own bound (optimal x 1.05, `PLAN_TOL`). Before it was
  15.05%. The bound is unchanged: one pass at a cell boundary now
  reads the full margin.

Pinned by:

- `four_corner_residues_...`: air feed at most 2x the cut;
- `an_island_...`: air feed at most 1.5x the cut;
- `a_neck_the_earlier_tool_...`: at most 24 plunges, air feed at most
  1.6x the cut;
- `the_travel_budget_refuses_by_name_past_each_limit` (unit).

Checked by mutation:

- Without the retrace, the corner and neck plans are refused by the
  budget (479 and 1553 mm out of cut) and the island test fails its
  air assertion.
- Without the cells, the neck is refused for 22 hops against 16.

**Considered and dropped**

- **Lifted returns (`adaptive/lift.rs`).** A lifted return comes back
  down where this operation has cut. That is a descent the entry law
  does not credit, and it would add plunges, which this issue counts
  down.
- **A straight return from the run's end to the next run's start, or
  a shortcut chord across part of the retrace.** In a corner the
  phases fall by at most 15%, so the arc before always stands within
  about one step. A chord between the two walls crosses it, which the
  crossing law refuses.
- **Trimming the wall margin a run carries past its bite (0.5 r, ridden
  out and back).** It is part of what the run was planned and load-read
  with. Left as is.
- **Arc fitting.** Not started. The posted moves are still the measured
  chords (#2031 point 9).

**Cost to change later**

- The budget is two constants in `budget.rs` and one yardstick. A
  planner that travels less can tighten them.
- A shape that travels more is refused by name. It never gets a longer
  path planned.
