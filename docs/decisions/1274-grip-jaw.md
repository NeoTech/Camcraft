## #1274 - Self-centering vise grip jaw (a step the stock sits on)

**Decided.** `vise::SelfCentering` gains `grip_depth` (ledge below the jaw
top) and `grip_width` (ledge reach in from the clamping face), both 0 =
no step, so every existing vise and project is unchanged. With a step the
jaw profile walks down the face to the ledge, in along it, and down to
the floor; the vise frame's z = 0 stays "the floor the part sits on" -
the ledge - and the whole profile drops by `jaw_height - grip_depth`.

- **Frame origin = seat, not floor.** Fit already puts the frame origin
  at the stock's bottom, and `SetVise` keeps the pose. Keeping z = 0 at
  the seat means a fit, and an edit that turns the step on or off on a
  placed vise, both keep the stock seated on the ledge with the vise body
  dropping under it. Dropped: keeping z = 0 at the floor and lifting the
  pose in Fit - an in-place edit would then put the ledge inside the
  stock unless `SetVise` also moved the pose (a second undo half).
- **Dovetail or grip, never both.** The issue allowed combining only if
  the code already did; it did not, so both on one vise is refused by
  name ("a dovetail or a grip jaw, not both"). No `jaw form` enum was
  added: the forms stay figures (0 = off) like the existing lip, so the
  MCP `dimensions` object and the project key stay additive.
- **Law** (refused by name at command creation): both figures or neither;
  non-negative, finite; depth < jaw height; width < jaw thickness; the
  opening more than two widths (else the ledges meet under the stock).
  A stock too wide is the existing clamping-range refusal.
- **Project file.** Two optional keys on the `self_centering` object,
  written only when non-zero; an older file reads as no step and a
  stepless vise saves byte for byte as before, so the project version
  does not move. An older build reading a grip vise ignores the keys and
  keeps the saved (stepped) facets; only a re-edit in that older build
  would drop the step.
- **Setup sheet.** Not touched: it does not state vise jaw forms today.

**Cost to change later.** A jaw-form enum, or combining dovetail and grip,
is a widening of `check_grip` and the profile walk; the wire keys stay.
