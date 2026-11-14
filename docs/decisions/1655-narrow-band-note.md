## #1655 - A band of the stock around the part narrower than the tool is named, not cut

**What was decided.** A pocket-family op (2D Pocket, 2D Adaptive, 2D
Offset) that clears the stock around the part (a whole-model source, #12,
or the stock source with parts, #1554) now carries a note for each
connected piece of its area that erodes to nothing by the tool radius plus
the stock to leave. The note gives the kind, the tool diameter, the stock
to leave and the XY box of each piece. The cut itself does not change. The
piece finder (`machining2d::unreached_pieces` / `unreached_note`) is pure
and lives in `camcraft-motion`; the app calls it once after generation
(`keep_law::narrow_noted`).

**Why.** The issue allows either outcome. To cut the band, the region's
outer boundary must grow past the stock outline by the tool radius, so
that the tool can hang over the stock edge. That moves every cutter
position of every whole-model and stock-source pocket (also the round
stock corners), so goldens would change, and the region is also the
preview and the verify bound. A note changes nothing for a region where
every piece is reached, so no golden moves.

**Considered and dropped.**
- Cut the band (tool over the stock edge): the better result for the user,
  but a behaviour change to every around-part pocket. It is a candidate
  for a follow-up issue.
- Put the note in `pocket_entered` for every pocket region: picked pockets
  with narrow slivers would get new CL notes, and goldens could change
  (the issue's own risk line).
- Refuse the op: the cut of the reachable area is correct and useful; a
  refusal would block it.

**Noise.** A piece narrower than 0.01 mm (it erodes to nothing by
0.005 mm) or smaller than 1e-4 mm^2 is a sliver where a stock face meets a
part face. It gets no note.

**Cost to change later.** Low: one call site in `generate.rs` and one pure
module. A change to cut the band would remove the note for those cases.
