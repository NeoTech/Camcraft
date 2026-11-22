## #3355 - "Pass starts at" on 2D Adaptive

**What was decided.**

- 2D Adaptive takes the shared `pass_start` field of #3316: the same
  type (`machining2d::PassStart`), the same project key (`pass_start`,
  `auto` never written), the same Passes-tab row and the same MCP door
  (`edit_operation`). No new type. The digest writes the value only off
  Auto, so every earlier operation keeps its cache stamp.
- The law for the Spiral method (`pocket_hsm_start::adaptive_passes_from`):
  - **Auto**: the plan every earlier file cut. The stock outline that
    the generator now passes is not read. All goldens stay
    byte-identical. A pin compares the two paths.
  - **Outside**: each piece of the area is cut from the air past the
    stock edge inward, with the #1950 air plan that the 3-D ladder
    already uses. A piece that the air plan cannot start refuses the HSM
    plan (`hsm_passes` with `air_only`), because the growth spiral would
    start it in its core. Then the Offset rings cut the area in the
    Outside order, at the load step. The note names the switch.
  - **Inside**: in an area without holes, the growth plan is the Inside
    order already. It starts at its bore in the core and grows out to
    the wall, so the plan does not change. An area with holes (islands,
    or the part of an Outside band) cannot start on a hole wall without
    a full-width slot. The Offset rings cut it from the holes out. The
    note names the switch.
  - The trochoid fallback is tried only under Auto, because its loops
    have no ring order. Every ring fallback takes the asked order.
- The Trochoidal method refuses Inside and Outside by name: at the
  command door (`named_refusal`), in the generator, and as a consequence
  line under the row. This follows the pattern of corner relief and arc
  turns under Trochoidal.
- The stock outline goes to the planner only for an untilted operation,
  because the outline is in world XY. A tilted Outside operation says
  "no stock outline" and uses the Outside rings.

**Why.** The maintainer asked for one parameter on every kind with a
pass order (#3316, 2026-11-22). An adaptive plan keeps the tool load
constant. It can start only in open air or in its own bore, never on a
wall. So where the plan cannot honour the order, the rings honour it,
and the note says so. The order is never dropped without a word.

**Finding.** With an Outside containment, the 2D band stops where the
tool's disc touches the stock (centre at stock edge + R). The air plan
puts its entry ramp 0.005 mm (`AIR_GAP`) further out, outside the band.
So on fig8 (the #2849 pin 3 case), Outside uses the Outside rings. The
air plan runs where the boundary reaches farther past the stock (a
sketch boundary bigger than the stock). A motion pin covers that path.

**Considered and dropped.**

- A stock outline made smaller by the air gap, so that the fig8 band has
  room for the air entry. The first plunge would then touch the stock.
  That breaks the air plan's guarantee.
- A band made wider past the stock for Outside. That changes the
  containment law of every kind, and it is outside this issue.
- A refusal of Outside where the air plan cannot start. The rings cut
  the asked order with one stepover of engagement, so a refusal would
  only take a working option from the user.
- A "no pass order" row for 2D Adaptive. The Spiral method has an order
  (growth from the bore, or air inward). Only Trochoidal has none, and
  its row says so by name.

**Room for #1448.** The start point and the exit point go beside
`pass_start` as their own fields, as in the #3316 decision. The air plan
and the rings each take an entry point. The growth plan already chooses
its own bore (`choose_entry`).

**Cost to change later.** Low. The law is one dispatch function. A
wider Outside band would let the air plan cut fig8 with no change here.

**Merge resolution, 2026-11-22 (class DELTA).** The parent #3353 merged
first (`e313aa663`) and changed the same functions. Its `StartSide`
already carries `{ start, stock, around_part }`, and its
`pocket_started_on` and `level_rings::ring_plan` replaced the branch's
`pocket_started_in` and `plan_passes_from`.

- Main's API wins. `pocket_started_in` is dropped and every caller uses
  `pocket_started_on` with a `StartSide`.
- `adaptive_passes_from` takes the region and the `StartSide`. Every
  ring fallback is `ring_plan`, so it keeps #3316's order and its
  completeness guard; the older `plan_passes_from` had no guard.
- `adaptive_ordered` takes the region and `around_part`, so the fallback
  rings name the same start wall as the Offset path.
- The 2D Adaptive generator builds one stock source,
  `pass_start_stock::footprint`, and one API. The branch's `air_stock`
  (world XY only) is dropped.
- Auto is unchanged: it reads no stock and plans with `plan_passes`.
- One pin changed. `fig8_inside_adaptive_starts_on_the_part_wall_and_
  ends_in_air` becomes `..._and_cuts_out_to_the_edge`. Main's ring law
  cuts each connected piece of the area on its own, with that piece's
  walls last. On the fig8 band the level therefore ends on the last
  piece's last cut, not always on the stock edge. The new pin asserts
  what the feature gives: the first cut is on the part wall, and the
  level cuts the stock edge in air on both sides (no material left).

Considered and dropped: restoring `plan_passes_from` and the
`waterline_passes(start)` order beside main's planner. It would keep the
old pin, but it is the design that #3316 replaced with `level_rings`, and
it would fork the ring law for one path.

**Cost to change later.** Low. One dispatch function and one stock
source; a later change touches `pocket_hsm_start.rs` only.
