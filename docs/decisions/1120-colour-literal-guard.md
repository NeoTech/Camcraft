## #1120 - Colour literals: a guard with exact counts first, then one PR for each family

**What was decided.** The first #1120 PR adds only the guard,
`crates/ui/tests/theme_files/colour_literals.rs`, with its scanner in
`colour_scan.rs` beside it. Both are modules of the existing `theme_files`
test binary (#3187: no new binary), whose gate map names `ui::egui_app`. It walks `crates/ui/src`, counts the
colour literals in each file that is not a test file, and compares each
count with a row of `ALLOWED`. A literal is a named constant
(`Color32::WHITE`) or a constructor whose arguments are all numbers written
in place; a constructor over variables converts a colour chosen elsewhere
and does not count. The count must be equal: more fails ("read a
theme token instead"), fewer also fails ("lower the row"), so the list only
shrinks. Each row says why: `keep:` for a colour that is not a choice at a
draw site, `move (#1120 <family>)` for a site that a later PR moves to
tokens. On `main` at f287a370c the list has 24 files and 142 literals.

The families that later PRs move, each with its tokens and its theme-file
keys (`theme_files::COLOR_NAMES`):
- viewport overlays (pick, avoid and handle inks, ground ink, view cube,
  viewport bar, generate overlay, drop hint, shell overlays);
- tool drawings (tool, holder and profile previews, tool library views);
- verdict palettes (`colorvision.rs`), moved into the token table with their
  values unchanged, as the issue says;
- G-code syntax colours, window chrome, status inks.

The `keep:` rows are the token tables, the contrast maths, and a colour
that only measures text.

The guard does not see colour data kept as plain numbers: the G-code syntax
table in `gcode_highlight.rs` (tuples), and `[u8; 3]` consts such as
`PICK_RGB8`. The family PRs move those by name, and the last PR checks them
by hand.

**Why.** On `main` there are about 105 literal sites to move, not the 51 of
the 2026-11-12 estimate. One PR for all of them would touch about 30 files
and conflict with most open UI PRs. The guard stops the count from growing
from the first merge, and each family PR is small and reviewable.

**Considered and dropped.**
- A guard that only refuses a file with no row. It lets the count inside a
  listed file grow without notice.
- Counting every constructor. A first version did: it counted 162, and
  about 20 of them only convert a palette colour (`from_rgb(r, g, b)`,
  `from_rgba_unmultiplied(r, g, b, ALPHA)`). Moving those would change
  nothing a theme can reach.
- A regex or a parse of the Rust source. A substring search with a small
  argument scanner is enough for the forms that occur, and the guard stays
  in `std`. The scanner first removes comments (`//` to the line end and
  nested `/* */`) and keeps the text of string and char literals: the
  #3990 review showed that skipping only whole comment lines lets a
  comment after code hide a literal or add one.
- A new top-level test binary for the guard. Each `camcraft-ui` test
  binary costs a link and disk space; the guard is a theme test, so it
  joins `theme_files`.
- Moving the colour-vision maths and the contrast maths. They compute
  colours; they do not choose them.

**Cost to change later.** Small. The guard is two test modules. A new literal
form is one line in `PATTERNS`, and the counts are printed by the failing
test, ready to paste.
