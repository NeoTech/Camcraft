## #1122 - The turning page family draws through the form seam

**What was decided.**
- The turning family is `turning_page.rs`, `turning_more_page.rs`,
  `turning_start_exit_rows.rs` and `thread_infeed_page.rs` (the infeed rows
  of the Threading page). The lathe tool pages (`tool_body_lathe_page.rs`,
  `tool_turning_row.rs`) are in the tools and machine family, so they are
  not in this slice.
- The three raw calls left in the family go through the helpers that the
  holes slice (#1724) added:
  - the Threading page's Hand row uses `radio_pair`, as the thread milling
    page does;
  - the Spring passes row uses `fields::int_range_row`, as the thread
    milling page does. The field now has the row's accessible name
    (`labelled_by`) and the house focus ring, the same as the other number
    rows.
- The raw-call guard (`layout/form_seam_tests.rs`) gets a `turning` family
  at 0. The test that pinned the guard to the holes family only now pins
  the list of migrated families.

**Why.** One seam for rows and fields lets a look restyle every page at
once, and it is where accessible names are set (#1119).

**Considered and dropped.**
- Keep the `CTRL_W` width of the Spring passes field. `int_range_row`
  sizes the field as egui does, the same as the thread milling page; a
  width argument on the helper is a form-seam change for one row.
- Put the lathe tool pages in this slice. They belong to the tools family,
  whose other pages are not migrated yet.

**Cost to change later.** Small: two call sites and one guard row.
