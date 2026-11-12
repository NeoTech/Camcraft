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
- **Defaults are 0 (off), not 3 / 1.5.** The issue asks for parameters
  with defaults; the step is opt-in so existing vises stay unchanged, and
  the dialog tooltips name the typical 3 mm deep / 1.5 mm wide figures.
- **Width is measured against `jaw_thickness`.** `check_grip` caps
  `grip_width` below the jaw plate's thickness. The ledge is added inward
  of the clamping face (into the opening), not cut from the jaw body
  (`jaw_body_depth`), so the model is never thinner than the real steel.
- **No parallels under a grip vise** (review of f0e63454). Parallels fit
  from the vise floor; a grip vise seats the stock on its ledges, so a
  fit would stand the stock clear of the jaws. "Parallels under the
  stock" in a grip-jaw vise is refused by name. Dropped: a seat-aware
  `fit_parallels` - parallels under a stepped jaw make little physical
  sense.
- **Setup sheet.** Not touched: it does not state vise jaw forms today.

**Cost to change later.** A jaw-form enum, or combining dovetail and grip,
is a widening of `check_grip` and the profile walk; the wire keys stay.
