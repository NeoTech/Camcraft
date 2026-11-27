## #1120 - Viewport overlay inks become theme tokens (third family)

**What was decided.** Ten colour tokens join `Tokens`, each also a
theme-file key (`colors.<name>`):
- the generate progress panel: `overlay_panel`, `overlay_edge`,
  `overlay_text`, `overlay_track`;
- the file drop hint: `drop_veil` (its frame and text read `overlay_text`);
- the empty 3D view: `empty_hint`;
- the dark halo under a picked edge and under the Geometry-list row hover:
  `pick_halo`;
- the fixture plate marks: `plate_hole`, `plate_dowel`;
- the move of the selected G-code line: `gcode_line`.

Both tables hold the values that the draw sites wrote before, so no pixel
changes. The draw sites did not follow the viewport ground, so the ground
rule of the view cube inks (`docs/decisions/1120-ground-inks.md`) does not
apply to these keys: a key that a file omits keeps its base value.

The outline of the origin triads (`shell.rs`, black at alpha 170) reads the
existing `handle_shade`: it is the same value and the same role, a dark
outline under a line in the 3D view. Its user-guide row now names the triads.

`pick_ink.rs` reads `pick_halo` too. `HALO_ALPHA` stays as the documented
alpha; the test pins the token against it.

The colour-literal guard loses the rows of `gen_overlay.rs`, `file_drop.rs`
and `shell.rs`. The two `Color32::WHITE` in `gen_overlay.rs` that only
measure the text width now read `overlay_text`, the ink that the text is
drawn in.

**Why.** Flat names, as in the first two families. One key for each role:
the drop hint's frame and text are the same white as the panel text, and a
theme that changes one wants the others to follow.

**Considered and dropped.**
- A separate key for the origin triad outline. It would duplicate
  `handle_shade` with the same value and the same role.
- A light-column set for these overlays. The panel and the drop hint draw a
  dark veil first, so white text reads on any ground; the plate and G-code
  inks are a separate question for a later issue, and #1120 moves values,
  it does not change them.

**Cost to change later.** Small: ten fields, ten keys.
