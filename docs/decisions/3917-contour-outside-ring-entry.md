## #3917 - Every wall-side ring takes the entry

**What was decided.** An outside (wall-side), untabbed ring after another ring at the same level takes
the entry when the entry (or the level transition below the first level) is Ramp or Helix. It goes
through `engage_level`, so it takes the wall-side ramp of #3833 (`contour_wall_ramp`). The helix stands
down to that ramp with its one note, as on the first ring. The change extends `contour_link::ring_entry`
of #3834 (bore side) to the wall side. The traverse between the parts rides the pass link plane, by the
same `Step::traverse` rule as the bore side.

**Why.** The pass link lifted, crossed and plunged straight down at the plunge feed, with no note. Two
parts at 3 levels posted 3 axial dives and `notes: []`.

**Kept.**
- The wall-side Arc entry keeps its own path (`wall_arc_hop`, `wall_arc`). `ring_entry` returns `None`
  for it, so its graze notes stay as they are.
- Tabbed levels and the Plunge entry keep the pass link's plunge.
- A pass link `KeepDown` that holds its proof still feeds straight across.
- A ring too short for the ramp angle still grazes, with the existing "too short" note.

**Dropped.** A note on the old plunge instead of the ramp (the second option of the issue). The ramp is
the safer motion, and the wall-side ramp needs no new certification: it rides the lead-in and the ring.

**Cost to change later.** Low. The rule is one condition in `ring_entry`.
