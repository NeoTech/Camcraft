## #2677 - Whole-model contour: fence the lead room at a shared level

**Decided.** The whole-model 2D Contour plans all its bands first and emits
them at the end (`contour_whole_fence.rs`). Before a band is emitted, its lead
room (`PreparedBand::room`) is its center region plus a thin fence (0.05 mm on
each side) around each ring of another band that is cut on a level this band
cuts too. The lead fitter (`lead_fit.rs`) then shortens, reseats or refuses a
lead that would cross that ring. The levels do not change. An error that
carries the partial path (ghost) emits the planned bands first, so the ghost
is the same as before.

**Why.** With the stock top one stepdown or more over Top Z, the grown ring
must take Top Z as its first level: a lower first level bites more than one
stepdown. So the grown ring shares the Top Z plane with the band-0 ring, and
a lead longer than the gap between the two rings crosses the other ring at
the same Z. The verifier's `Crossing` law then blocks the post. The same
happens with no stock over Top Z, at the flat levels under Top Z. The only
fix that keeps the law and the stepdown is in the lead room.

**Dropped.**
- A level a little off Top Z for the grown ring: that hides the law (two
  passes 1 um apart cross in fact) and was refused by the issue.
- A level under Top Z: the first bite is then more than one stepdown.
- Emit order changes: the `Crossing` law does not depend on the move order.

**Costs and limits.**
- The fence is per band, not per level: a lead of a band on a level that no
  other band shares can also shrink at a fenced ring. This is conservative.
- The fence leaves a 0.1 mm gap around the band's own rings, so a ring of
  another band that runs on one of them (a touch, not a crossing) does not
  close the lead's own start.
- Lead notes of a band now come after the planning notes of the later bands,
  and a planning error of a later band now comes before an emit error of an
  earlier band.
- To change it later: give `room` per level, or make `upper_levels` move the
  other ring's levels instead.
