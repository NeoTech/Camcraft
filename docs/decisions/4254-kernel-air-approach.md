## #4254 - Path kernel K4c: an Outside piece starts at the stock edge with a proved approach from the air

Follow-up of K4b (#4241, `docs/decisions/4241-kernel-link-edge-band.md`,
"What can remove these lifts") and of K4 (#4177). Sub-PR of
`side/path-kernel`. No old planner changes and no planner calls the new
code yet: with the default switch the output is identical.

**What was decided**

1. **The approach is the last stretch of a stay-down link, not a new
   step.** A link from the end of one piece to the start of the next can
   end with an approach from the air: a straight stretch from a place of
   the air to the start of the next piece. The hop is `Hop::Stay`, so the
   plan has no new step kind and the transition table of `Plan::new`
   stays the one gate (cut end, stay link, ready, cut). The link stage is
   the only stage that makes a stay hop, and it makes one only when each
   stretch of the path is proved: the stretches before the approach by
   the chord proof of K4 (#4177, #4241), the approach by the proof below.
   A move that has neither proof is a lift with the named reason of the
   straight link, as before.
   Dropped: a lift-free re-entry step in the plan (`Step::Approach` or an
   entry without a lift). It needs a new row in the transition table and
   a second gate for the same move, and the entry stage would then prove
   a move that starts at depth in the cut plane, which is the link
   stage's work.
2. **The proof of the approach** (`link/approach.rs`). The next cut must
   be an open piece whose start is on a stock edge loop of the region
   (within the room tolerance). The approach starts at the place of an
   air loop nearest to the start, within the edge band of it (a tool
   radius and the room tolerance). The approach holds when:
   - each sample is in the air or in the edge band (out of the part
     zone), more than a tool radius from each keep-out area, and not
     inside the cutter-centre area: the tool comes from outside the stock;
   - the stretch holds in the legal room of the entry stage
     (`entry::legal_room`), as the lead of an air entry does;
   - it crosses no cut made and no stay link before it, and touches no
     planned cut away from the ends of the link (`CROSSES_CUT` test of
     K4);
   - **the disc swept along it cuts no stock off the planned curve**: the
     swept disc, less the swath of the next piece (the disc swept along
     that piece), lies within one tool radius of the air. The air starts
     one tool radius outside the stock, so no point inside the stock is
     that near to it: the rest is outside the stock. So each piece of
     stock that the approach cuts is cut by the next piece too, and the
     tool begins to cut along the planned curve.
   The area test is exact polygon geometry (`Region::sweep`,
   `difference`). Its only tolerance is the polygon flattening: both
   covers are grown by twice `ROOM_TOL` (10 um), because the swept discs
   and the grown areas are each flattened within `ROOM_TOL`. No strip of
   uncut stock is open by a constant (the defect that the review of
   PR #4234 found).
3. **The way to the air is a link as any other.** The path to the door
   of the approach uses the three tries of K4 (`proof::tries`, the code
   of `stay_path` made reusable): the straight chord, back along the cut
   just made, a walk along a loop of the centre area or of the air. The
   approach is the fourth try of the link, after those three to the start
   itself fail. Shorter, proved centre walks still come first.
4. **The order stage needs no new rule.** The start of an open piece is
   fixed by its direction (climb or conventional, the start side's
   sense): the order may not reverse a piece to reach another end
   (law 3 and the direction choice). A ring cut open at the stock edge
   has its two ends on the stock outline, so the start that the
   direction gives is an end that the air reaches. The walk already asks
   the link stage for a proved path to each of its first 16 starts
   (`link::proved_path`), and that path now includes the approach. So the
   walk takes the piece whose approach is proved, in its present order
   of preference.
5. **A piece whose start is not on the stock edge keeps the present
   rule.** A closed ring, a piece that starts inside the stock or in the
   edge band, and a start on a loop that is a part wall (a part that
   touches or leaves the stock) get no approach. Where no proof holds the
   lift stays with the reason of the straight link.

**Readings** (measured again; no constant changed or added for the
counts). The law suite (`order_link::k4_link_rule_holds_and_each_lift_is_named`;
Auto, Inside, Outside) and the motion pin
(`kernel_order_and_link_round_the_disc_and_the_dog_bone`; lifts and
crossing feeds):

| Fixture | Before | After |
|---|---|---|
| disc pocket (law suite) | 0 / 0 / 0 | 0 / 0 / 0 |
| disc band (law suite) | 13 / 0 / 13 | 0 / 0 / 0 |
| dog-bone band (law suite) | 15 / 0 / 15 | 0 / 0 / 0 |
| pocket with an island (law suite) | 7 / 7 / 5 | 7 / 7 / 5 |
| disc, Inside (motion) | 0 lifts, 0 crossings | 0 lifts, 0 crossings |
| disc, Outside (motion) | 13 lifts, 0 crossings | 0 lifts, 0 crossings |
| dog bone, Inside (motion) | 0 lifts, 0 crossings | 0 lifts, 0 crossings |
| dog bone, Outside (motion) | 15 lifts, 0 crossings | 0 lifts, 0 crossings |

The record's target of 0 lifts on the disc is met. The pocket with an
island has no stock edge: its lifts and reasons do not change.

The engagement readings of the motion pin (Outside; the test turns each
lift into a plunge, so "before" has 14 and 16 plunges):

| Fixture | Before | After |
|---|---|---|
| disc, Outside | over 402, worst 145, full 0 | over 415, worst 145, full 0 |
| dog bone, Outside | over 724, worst 167, full 0 | over 739, worst 180, full 1 |

On the dog bone one approach arrives at full width: at (-2.40, 25.00) on
the top stock edge. There the approach runs straight in to the start of a
piece on stock that no cut has touched, and for its last tool radius the
disc cuts as a slot (180 degrees). The material is in the swath of the
piece (the proof above), so it is not a gouge and not stock off the
curve, but it is load that no stage reads yet: the load stage (K3) reads
the cuts, not the links. Follow-up: the load stage reads the approach of
a link (#4272).

The K3 pin (`k3_load::kernel_load_control_round_the_disc_and_the_dog_bone`,
merged to the side branch in PR #4265) reads the same Outside plans. Its
rows (moves above the limit, worst degrees, moves above the limit with no
named place) are pinned again as measured; Inside does not change:

| Fixture, load control | Before | After |
|---|---|---|
| disc, Outside, none | 402, 145, 13 | 415, 145, 19 |
| disc, Outside, constant 25 % | 334, 118, 74 | 344, 122, 81 |
| dog bone, Outside, none | 724, 167, 3 | 739, 180, 13 |
| dog bone, Outside, constant 25 % | 678, 143, 66 | 683, 143, 69 |

The new unnamed moves are the approach stretches: a link, which the load
stage does not read (#4272). With constant engagement the dog bone,
Outside, lifts 7 times (each named): the load stage changes the curves,
and that pin does not pin lifts.

**Dropped.**
- A tangent approach (along the start direction of the piece). Where
  the piece meets the stock edge at a slant, the extension sweeps stock
  behind its start that the piece does not cut: the area test refuses it.
  The straight way in from the nearest place of the air is the one that
  the area test proves on any straight stock edge.
- An arc lead as the air entry uses. It runs in the centre area over
  stock off the curve.
- A 4th try in the edge route only (the last chord of an air walk). The
  approach as its own try also takes a straight chord or the way back
  along the cut out to the air.

**Cost of a change later.** The approach is one function
(`link/approach.rs`, `approach`) and one try (`from_air`, called from
`stay_path`). Its area tolerance is one constant (`AREA_TOL`).
