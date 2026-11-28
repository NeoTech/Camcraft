## #1120 - The window chrome inks are tokens; the guard keeps only permanent rows

**What was decided.**
- Two colour tokens join `Tokens`, each also a theme-file key: `close_hover`
  (the close button's hover fill, the desktop caption red `#C42B1C`) and
  `close_ink` (the white cross on it). Both built-in columns keep the old
  values. `window_chrome::CLOSE_HOVER` is gone; `caption` reads the tokens.
- The `error_text.rs` row of the colour-literal guard was not a draw site.
  Its one literal (`Color32::RED`) was in the file's inline test module,
  and every caller passes a theme or palette ink. The test module moves to
  `error_text_tests.rs`, which the guard skips as a test file.
- The guard now has only `keep:` rows (the token tables, the verdict
  palette table, the contrast maths and a colour that only measures text).
- A new test pins the colour vocabulary inside the 64-entry map cap of a
  theme file (61 keys now), as one already pins the geometry vocabulary.

**Why.** These were the last Color32 literals at draw sites in
`crates/ui/src`. A row that stays must have a reason that is not a theme
choice.

**Considered and dropped.**
- A light-column close red. The desktop caption red is the same on light
  and dark title bars; a new value is a design change, not a move.
- Teach the scanner to skip inline `#[cfg(test)]` modules. Only one file
  had the case, and the sibling test file is the house pattern.

**Cost to change later.** Small: two fields and keys. The map cap has three
free colour slots; a later family that needs more raises the cap.
