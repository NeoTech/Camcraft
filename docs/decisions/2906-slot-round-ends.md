## #2906 - A wide Slot cuts its round ends: one wall lap after the rows

**What was decided.** When the Slot is wider than the tool (`h > 0`), each
level runs one lap of the tool centre stadium after the rows. The lap
starts where the last row ends (a tangent point of the +n side). It goes
around the full stadium and stops at its start. Each half circle is a
chain of arcs (G02/G03), split at every row end on it and at its tip. The
rows and the lap compute a row end with one function (`cap_point`), so
each row join meets the lap at the same point to the bit. The lap goes counterclockwise
(air on the left) when `climb` is set, the same sense as the rows, and
clockwise when it is not. A full-width slot (`h = 0`) is unchanged.

**Why.** A row touches a round end only at its end point. The tool did not
follow the arc between two row ends, and left up to 0.86 mm on a 10 mm
slot with a 6 mm tool. The issue offered two fixes: cut the full outline,
or warn. Cutting is the better answer: the operation then makes what it
says. The lap stays on the boundary of the cutter centre stadium, so it
cannot gouge. True arcs post short, exact G-code, and the CL already
carries arcs for other operations.

**Dropped.**
- A warning only: the user still gets a slot with material in its ends.
- Polyline caps: more blocks and a small chord leftover.
- Caps only (no second straight side): the two caps are not connected
  without a pass along one side. A full lap also finishes the two straight
  walls in one climb pass.
- One 180 degree arc for each cap: some controls read a half circle in R
  form as ambiguous. With the split at the tip, no arc is more than 90
  degrees.
- Two quarter arcs for each cap, not split at the row ends: the verifier's
  crossing law (a) flattens an arc into chords inside the circle. A row
  join is a chord that ends exactly on the circle, so it crosses those
  flattened chords properly, and the post is held. This was the first
  version; the gate found it on 8 app suites. Split at each row end, the
  join and the lap share their vertices, and a shared vertex is a touch,
  not a crossing.
- A change to the crossing law: the law is a safety check under the
  safety floor. The slot can obey it, so the law stays as it is.

**Side effect.** `clearance_in_stock`: the slot's lateral traverse between
its levels was convicted only because the uncut crescents of the round
ends stood in its path. With the ends cut, the traverse rides over the
slot's own channel and is acquitted. The plane is still convicted by the
first traverse and by the rapid down into the stock, and the post is held.

**Cost to change later.** Small. The lap is in `machining2d/slot_lap.rs`.
The golden `slot_wide_channel_rows` (CL and each NC dialect) and the catalog
case `2d/slots` pin its moves.
