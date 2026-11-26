## #1120 - Viewport handle and pick inks become theme tokens (first family)

**What was decided.** Three colour tokens join `Tokens`, with the same
names as theme-file keys (`colors.<name>`):
- `handle_shade`: the shadow under a move or turn handle and under its
  readout (was `Color32::from_black_alpha(170)`);
- `handle_text`: the readout text of a handle, and the fallback ink of an
  axis (was `Color32::WHITE`);
- `pick_rim`: the rim of a picked point's disc (was
  `Color32::from_rgb(245, 240, 250)`).

Both built-in columns carry the old values, so no pixel changes. The draw
sites in `move_handles.rs`, `turn_handles.rs` and `align_pick_viz.rs` read
`theme::current()`, and their rows leave the colour-literal guard.
`geom_legal` and its two asserts move unchanged from `theme.rs` (425 lines)
to `theme/legal.rs` for the 400-line law.

**Why.** Flat field names follow the table as it is (`ribbon_ink`, #2021).
If #659 later maps our tokens onto another theme crate, a flat name maps one
to one. The light column keeps the dark values because the handles and the
picks draw over the 3D view, and a shadowed white readout reads on a dark or
a pale ground.

**Considered and dropped.**
- A nested `ViewportInks` struct inside `Tokens`. It reads better, but the
  theme format and both exhaustive destructures are flat; a nested struct
  would change three files for no user gain, and #659 may rename it all.
- Deriving the inks from existing tokens (`text`, `viewport`). The readout
  must read on the 3D view, not on the chrome, so `text` is wrong in the
  light column.
- Moving `pick_ink.rs`'s halo. It is `from_black_alpha(HALO_ALPHA)` with a
  named alpha constant; the alpha is a geometry choice pinned by its own
  test, and the guard does not count it.

**Cost to change later.** Small: three fields, three keys, three rows of the
user guide's table.
