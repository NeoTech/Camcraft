## #4241 - Path kernel K4b: a link can run outside the stock where the ground is proved

Follow-up of K4 (#4177, `docs/decisions/4177-kernel-order-link.md`,
decision 8). Sub-PR of `side/path-kernel`. No old planner changes and no
planner calls the new code yet: with the default switch the output is
identical.

**What was decided**

1. **The edge band is open ground for the part, not for the stock.** A
   link sample can now stand in the cutter-centre area, in the air, or in
   the edge band. The edge band is the band outside the stock within one
   tool radius of a stock edge. The link stage reads it from the legal
   room of the entry stage (`entry::legal_room`, K5): the centre, the air
   and the band at the stock edge, less the keep-out grown by the tool
   radius. There is one definition, not a second one. A sample in the
   band must also:
   - not be inside the part zone (see 2), so the disc meets no part;
   - be more than a tool radius from each keep-out area (else
     `HITS_FIXTURE`).
   The part of the disc inside the stock is stock as any other. The swath
   test of K4 stays as it is: a sample that is not in the air must be
   within 0.9 tool radius of a cut already made (else `UNCUT_STOCK`). The
   swath test reads only the centre, so it applies to a centre in the
   band as to any other centre.
   Dropped: the old ring links' `AIR` meaning ("outside the stock by
   0.1 r counts as air"). Such a centre cuts 0.9 r into stock that no cut
   has cleared. Also dropped: a test of the distance to the part wall
   loops. It is not exact (a wall loop is not the part), and the part zone
   is exact.
2. **The region keeps the part zone.** `PlanarRegion` has a new field: the
   part grown by the tool radius plus the stock to leave. The region
   builder sets it for an `Outside` region (`with_part_zone`, crate
   private); it is empty else. Why: the old region did not know the part
   outside the stock. A part can touch the stock edge or reach out of it,
   or a second chain of the selection can lie outside the stock. Then a
   centre in the band or in the air can be nearer to the part than the
   tool radius plus the leave, and no value of the region showed it.
   A region that `PlanarRegion::new` makes (not the builder) has no part
   zone, and so no edge band: the proof refuses there as before.
3. **The air leaves out the part zone.** The air was the band between the
   stock grown by one and by two tool radii, less the keep-out. A part that
   reaches out of the stock was in that air, and a link in the air could
   run across it (the old proof accepted each sample in the air). The air
   is now also less the part zone. The pin
   `a_link_beside_a_part_that_reaches_out_of_the_stock_is_refused` holds a
   link in the air across such a part. Where the part is inside the stock
   (all the present fixtures), the air does not change. The entry stage
   reads the same air: its room gets smaller only beside such a part.
4. **The edge route can walk along the air.** The third try of the link
   (a walk along a loop) now also takes the loops of the air. An end
   joins an air loop when it stands within the edge band of it (one tool
   radius plus the room tolerance: an end on the stock edge is one tool
   radius from the air). The walk goes out through the band, along the
   air, and back in. Each chord is proved as before. Centre walks are
   shorter and come first.

**Readings** (measured again, no constant changed). The law suite
(`order_link::k4_link_rule_holds_and_each_lift_is_named`; Auto, Inside,
Outside) and the motion pin
(`kernel_order_and_link_round_the_disc_and_the_dog_bone`; lifts and
crossing feeds):

| Fixture | Before | After |
|---|---|---|
| disc pocket (law suite) | 0 / 0 / 0 | 0 / 0 / 0 |
| disc band (law suite) | 13 / 0 / 13 | 13 / 0 / 13 |
| dog-bone band (law suite) | 15 / 0 / 15 | 15 / 0 / 15 |
| pocket with an island (law suite) | 7 / 7 / 5 | 7 / 7 / 5 |
| disc, Inside (motion) | 0 lifts, 0 crossings | 0 lifts, 0 crossings |
| disc, Outside (motion) | 13 lifts, 0 crossings | 13 lifts, 0 crossings |
| dog bone, Inside (motion) | 0 lifts, 0 crossings | 0 lifts, 0 crossings |
| dog bone, Outside (motion) | 15 lifts, 0 crossings | 15 lifts, 0 crossings |

The record's target of 0 lifts on the disc is not met. The band does not
remove a lift on these fixtures. Each lift on the disc band (Outside) is
at the start of a piece that no cut has touched yet, and the way in to
that start runs over uncut stock:

- 4 `UNCUT_STOCK`: in each corner, from the end of piece k on the top or
  bottom edge to the start of piece k - 1 on the side edge. The next start
  is 4.4 mm and more from each cut made; the swath is 3.6 mm.
- 4 `CUTS_PART`: from the end of piece k (k = 7, 6) on one long edge to
  the start of piece k - 1 on the other long edge. The straight link
  crosses the part. Back along the cut, the next start is 3.98 mm and
  5.47 mm from the cut made, more than the swath.
- 5 `CROSSES_CUT`: from the last piece of a group to the first piece of
  the next group (the next corner, or the first long piece), and from the
  last long piece on the left to the top right corner. The straight link
  crosses cuts made or still planned. The way round through the air is
  now proved out to the air and along it, but the way back in to the
  first piece of the new group is not within the swath of a cut made.

Each reason is the reason of the straight link, as in K4.

**What can remove these lifts** (not done here; the brief keeps the
`UNCUT_STOCK` refusal where the stock inside is not cleared): an
approach from the air into the start of the next cut cuts the same
material as the lead-in of an air entry, which the entry stage proves
without a cut made beside it. A rule that proves that approach as a lead-in
(not as a link over cleared stock) would let these links stay. That is a
change of the link rule, so it needs its own decision.

**Cost of a change later.** The band test is one function
(`link/proof.rs`, `in_band`). The air walk is one list in
`link/route.rs`. The part zone is one field of `PlanarRegion`, set in
`region_build.rs`.
