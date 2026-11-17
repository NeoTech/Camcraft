# 2025: theme.json.example

Issue: #2025. Stacked on #2027 (the ribbon keys).

## Chosen

- The example file holds each setting at its dark default, with base `dark`.
  A user who copies it gets the dark theme and changes what they want.
- JSON has no comments, and the loader reports an unknown key. So the
  per-setting documentation (meaning, unit, range) is a table in
  `docs/user-guide/themes.md`, not keys in the file.
- The test reads the key set from `COLOR_NAMES` and `GEOM_NAMES` of
  `theme_files`. The overlay builds its fields from the same names, so there
  is no second list. A second test loads the file and compares it to
  `Tokens::DARK`, so a wrong default also fails.
- The test file is `crates/ui/tests/theme_example.rs`, because
  `theme_files.rs` is at the 400-line limit.

## Dropped

- Comment keys inside the file: the loader would add a note for each.
- A `light` example: one file is enough; the user sets `base`.
