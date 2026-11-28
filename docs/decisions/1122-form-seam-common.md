## #1122 - The common operation tabs draw through the form seam; the page status moves to layout/status.rs

**What was decided.**
- The "operation tabs" family is the frame and the tabs that every
  operation page shares: `op_page.rs`, `op_page/header.rs`, `geo_page.rs`,
  `boundary.rs`, `heights_page.rs` and `feeds_page.rs`. The guard holds it
  at 0 raw calls.
- `pages.rs` is not in this family. It draws the post, setup and model
  pages, so it goes with the "setup and fixtures" family.
- The status banner and the action bar move unchanged from
  `layout/form.rs` (424 lines) to `layout/status.rs`. The UI plan #4245
  asks for this split before PR B, which adds its page frame (status card,
  action band) there. `layout` re-exports both modules, so no call site
  changes.
- Three new seam helpers, each a thin wrapper that draws what the raw call
  drew:
  - `layout::list_row_wrapped`: a text row that wraps (a reason line);
  - `layout::readout`: a text that the user cannot select (an echo of a
    derived value);
  - `layout::choice_button_enabled`: a selectable button that is greyed
    out while it is not enabled (a boundary role choice).
- An optional group's enable switch (`layout::group`) had no visible
  caption and no accessible name: AccessKit showed a check box with an
  empty label on the Heights tab. The seam now names it "Use {title}".
  The hover text stays "use {title}". Nothing visible changes.
- The new helper test is in its own file (`form_seam_ops_tests.rs`),
  because `form_seam_tests.rs` is near 400 lines.

**Why.** PR B of #4245 rebuilds exactly these files (header, status card,
height results, folded groups). When they draw through the seam, B can
change the look in the seam and not at each call site. The split keeps
`form.rs` under the 400-line law and gives B's new components a home.

**Considered and dropped.**
- Include `pages.rs` here. It is a different family, and it is 731 lines,
  so it needs its own split first.
- Put the new helpers in `fields.rs`. They are basic controls, so they go
  with the others in `controls.rs`.
- Split `boundary.rs` (440 lines on `main`). This PR does not grow it. A
  split is a separate change.

**Cost to change later.** Small: each helper is a few lines, each call
site is one line, and the module move is a file rename.
