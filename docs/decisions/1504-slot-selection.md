## #1504 - a plain Slot picked on the part stays inside its picks

**What was decided.** A Slot whose source is a selection takes its room
from the picks: two opposite straight edges give the strip between them
over their shared length; one face (or one closed edge ring) gives its
region, framed by its minimum-area rectangle. Picking seeds the slot that
fills the room (full width, round ends tangent to the room's ends) into
the op's endpoints and width, inside the pick's own command
(`SetOperationGeometry`, and `CreateOperation` when an op is created on
picks), so one undo takes back the pick and the seed - the same pattern
the Probe's pick seeding uses. The typed numbers stay editable: a
narrower or shorter slot inside the picks is cut as typed; one too wide,
too long or placed past them is refused by name with its numbers at
generation, and the preview draws nothing for it. The Geometry tab
echoes the room (or the refusal). A picked Slot is also judged against
the part with the pocket family's depth law (`DepthLaw::Section`), so a
slot cut below the picked floor is convicted as a gouge and holds the
post. With nothing picked (stock or whole-model source) nothing changes:
the typed endpoints rule and no part law applies, as before.

**Why.** The maintainer's report: the slot with two edges selected
"should stay within the selection". The seed answers "two edges give the
slot between them"; the confine law answers "a slot that does not fit is
refused by name, never previewed across a wall"; the depth law closes
the gap the confine law cannot see (a slot inside the picks in XY but
deeper than the feature). The fit/confine law lives in
`machining2d::slot_room` (`SlotBound`), world-free, so the Trochoidal slot
(#1503) can call the same `confine(a, b, width)`.

**Considered and dropped.**
- Ignoring the typed numbers once picks exist (the Circular op's law,
  "picks author the circle"): it would make "too long / too wide" moot
  and hide a user's narrower slot; the seed plus the confine law gives
  the derived slot by default and still lets a smaller one be typed.
- Clipping a typed slot to the picks: a silent change of the authored
  geometry - refused instead (Prime Directive).
- A separate "Fit to selection" button: the pick already seeds the fit,
  and re-picking re-seeds it; one more control was not needed.
- Stacking on #1513 (M5's `operation::slot_fit::refuse_unfit_slot`,
  the Trochoidal slot's region-capsule law) and flipping Slot in
  `selection_drives_region`: that walk refuses open runs for every kind
  but the contour, so two picked edges - the reported case - would be
  refused before any region exists, and the region-only law cannot read
  the strip two edges bound. This PR keeps its own `SlotBound` (edges and
  faces, in motion, world-free) and touches none of #1513's files or
  hunks, so the two merge in either order; unifying the two laws is a
  follow-up.
- Reading the picks with the pocket's selection walk
  (`selection_geometry`): it refuses open runs for every kind but the
  contour, and a slot is exactly two open runs.

**Tolerances.** `FIT_TOL` = 0.01 mm (the gouge law's own), so a picked
face's tessellated round end (chords a sag inside the true arc) is no
wall; `STRAIGHT_TOL` = 0.01 mm for an edge's straightness and the second
edge's parallelism.

**Cost to change later.** The seed lives in two commands and one app
function (`slot_from_picks`); the confine call sits in the job collector,
the Slot arm and the preview's `region_for`. Dropping the seed (or
making picks author the slot outright) touches only `slot_pick.rs` and
those call sites. The messages are in mm regardless of the document's
display unit, like the other slot generator refusals.
