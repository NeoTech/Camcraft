## #1120 - The verdict palettes move into the token table, values unchanged

**What was decided.** The six colour-vision palettes (`Semantic::OFF`,
`RED_GREEN`, `TRITAN` and their `_LIGHT_GROUND` sets) and the house text
inks (`TextInks::HOUSE`) move out of `colorvision.rs` and `ground_ink.rs`
into a new table file, `crates/ui/src/egui_app/theme/verdict.rs`. The
values do not change. The names do not change, so no call site changes:
they are associated consts of `Semantic` and `TextInks`. `colorvision.rs`
keeps the types, the CIEDE2000 gate and the simulation maths. The colour
literal guard drops the `colorvision.rs` row and gets a `keep:` row for
the new table. No theme-file key sets a verdict colour.

**Why.** #1120 says it in the maintainer's words: "The verdict colours
(gouge, rest, move kinds) and their colour-vision maths stay ours and stay
as they are - they move into the token table, they do not change value."
A theme key for them would let a file undo what the colour-vision gate
proves for each mode.

**Considered and dropped.**
- Theme keys for the house (Off) text inks only. The Off set is also the
  base that the gate checks against the body text, and the same three
  inks serve the dark and the light panel today. A key would need a
  per-ground pair and a gate run at theme load. That is a separate design
  question, not a move.
- The palettes in `theme/tables.rs`. That file holds the two theme
  columns (240 lines). A second table in it mixes a theme choice with a
  fixed palette and pushes it toward the 400-line law.
- Moving `interact::FEED_RGB`, `RAPID_RGB` and `PLUNGE_RGB` too. Other
  code reads them as plain numbers (shaders, overlay lines). The Off
  palette still names them, so they keep one source.

**Cost to change later.** Small: one file of consts. A theme key for a
verdict colour later needs only the gate run at load, not a move.
