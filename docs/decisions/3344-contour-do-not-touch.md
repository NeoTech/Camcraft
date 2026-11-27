## #3344 - "Do not touch" on 2D Contour operations

**What was decided.** 2D Contour takes protected faces and edges. Each
Contour mode has an avoid-aware twin in `camcraft-motion`
(`contour_avoid*`): closed rings, rings on a part, picked rings, the
whole-model contour, open chains, open runs (part side and stock side) and
DXF sketch loops and chains. With no protected item, each twin calls the
plain generator, so the toolpath is the same as before.

- A ring is clipped against the keep-out of the full tool (flutes, shank,
  holder) at each level. The kept pieces are cut as open pieces. A lead,
  a link or an entry that enters the keep-out is dropped or moved.
- A clipped piece ends a small step (`END_TRIM`, 0.001 mm) outside the
  keep-out boundary. A clip end lies on that boundary, and
  `Region::contains` counts the boundary as inside, so without the step
  each piece without a lead read as blocked and was dropped.
- On the bore side, the entry room of a piece is the centre region less the
  keep-out. It never falls back to the whole centre region. An entry point
  outside that room grazes at itself.
- A ring or a chain that is blocked on every level is skipped, and a note
  names the protected items (and the sketch chain). The operation is
  refused, with the items named, when no cut remains. Control
  compensation that a clip would break is refused by name.
- Each result goes through the full-tool verify before it leaves the
  generator. The app verify and the post gate read Contour moves as they
  read the other kinds (`verify_avoid`).

**Why.** The issue asks for a refusal or an "incomplete" mark when the
cut cannot keep clear. The app has no "incomplete" status for an
operation. Bore skips a hole that it cannot reach with a named note and
refuses only when no hole remains; the Pocket and Trace protection refuse
only when no room remains. Contour follows that precedent.

**The resumed branch.** The held work (1993c0cf1) was never built. The
merge of main needed these ports into the avoid emitter: the ring entry
(#3830, #3917), the pecked dive (#3678), the picked seam under
compensation (#1473), the tab level start (#3453) and the picked ladder.
The pins now live in the `do_not_touch` suite (#3187). Their tool has a
25 mm flute and a 2 mm shank: main's flute law (#3525) blocks the old
15 mm flute under the fixture's 24 mm stock with or without protection.

**Dropped.** A new operation status "incomplete": it is a project-file and
UI change outside this issue. A separate test binary for the Contour pins:
the rule of #3187.

**Cost to change later.** A formal "incomplete" status would replace the
skip notes in `contour_avoid*` and `generate.rs` / `contour_sketch.rs`.
