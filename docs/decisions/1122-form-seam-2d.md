## #1122 - The 2D and 3D page families draw through the form seam

**What was decided.**
- The 2D family is the 2D milling pages and the cutting pages, with the
  editors that they share: `mill2d_page.rs`, `mill2d_more_page.rs`,
  `trochoid_page.rs`, `vcarve_page.rs`, `probe_page.rs`, `plasma_page.rs`,
  `waterjet_page.rs`, `wire_page.rs`, `sinker_page.rs`,
  `sinker_pick_rows.rs`, `deburr_page.rs`, `pockets_page.rs`,
  `approach_page.rs`, `predrilled_page.rs`, `rest_pocket_page.rs`,
  `start_exit_rows.rs`, `pass_start_row.rs`, `load_field.rs`,
  `lead_page.rs` and `kind_editors.rs`. The 3D family is `mill3d_page.rs`
  and `undercut_page.rs`.
- The 3D pages had no raw call on `main`, so the 3D family is one guard
  row in this PR, not a PR of its own.
- The common operation tabs (`heights_page.rs`, `feeds_page.rs`,
  `op_page.rs`, `pages.rs`) are not in a family of #655's list. They stay
  outside the guard for a later slice.
- New seam helpers, each a thin wrapper that draws what the raw call
  drew:
  - `layout::button_enabled` and `small_button_enabled` (a greyed-out
    button), `checkbox` (a box that is not a property row),
    `list_row_truncated` and `note` (a dimmed line that wraps);
  - `layout::drag_row` and `drag_sized_row`: a configured `DragValue` in a
    `prop_row`, at its own width or at `CTRL_W`. The field joins the
    pane's Tab / Enter ring (#2516) and is named by its row (#1119).
    `fields::int_range_row` now draws through `drag_row`.
- The sinker page's EDM condition field uses `text_row`.
- The guard joins a line that starts with `.` to the line before it, so a
  call that rustfmt splits after `ui` counts (`deburr_page.rs` had one).

**Why.** One seam for rows and fields lets a look restyle every page at
once (#655). The seam is also where the accessible name and the focus ring
are applied, so seven number and text fields get them: Tab count, Taper
angle, Vectors, Turns, EDM condition, Number of passes and Optimal load.

**Considered and dropped.**
- One generic `add(ui, widget)` helper. It hides which control a page
  draws, so a look cannot restyle by control.
- Use `num_unit_row` for the taper angle. It has no range and puts a space
  before "°"; `drag_row` keeps the ±45° range and the suffix.
- A separate PR for the 3D family. It has nothing to change.

**Cost to change later.** Small: each helper is a few lines, and each
call site is one line.
