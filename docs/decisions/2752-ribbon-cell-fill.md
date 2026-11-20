## #2752 - ribbon cell fill holds its whole label

**Decided:** the fill and the click target of a ribbon cell extend down to
the bottom of its label. The extension stops 2 px above the group caption
row. On a filled cell, the label ink is the on-fill ink of the fill, the
same rule that the glyph uses. The cell code moves from `painted.rs` to
`ribbon_cell.rs`, because `painted.rs` was 468 lines.

**Why:** the label sits at `glyph_px + 2` below the cell top, but the fill
was `cell_h` tall. The shipped themes set `glyph_px` to 22 and keep
`cell_h` at 30. The label then ran from 24 to about 38 px, and the fill
covered only its top half. The built-in tables overflow by about 2 px. The
light themes showed it most, because a pale fill shows its edge.

**Dropped:**

- Raise `cell_h` from the glyph and the font (`ribbon_h` and
  `ribbon_cells` follow). That moves every ribbon row and needs an estimate
  of the line height in a `const fn`. The ribbon layout already permits
  the label below `cell_h`: `ribbon_rows` pins
  `cell_h + ribbon_caption_gap >= label_bottom + 2`.
- Change the shipped theme files to a larger `cell_h`. A user theme file
  with a large glyph has the same defect.

**Cost to change later:** low. `ribbon_cell::cell_box` is the one place
that sets the box.
