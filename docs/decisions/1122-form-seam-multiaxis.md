## #1122 - The multi-axis page family draws through the form seam

**What was decided.**
- The multi-axis family is `multiaxis_page.rs`, `rotary_page.rs` and
  `multiaxis_deburr_page.rs`. The setup rotary and axis pick pages
  (`setup_rotary_page.rs`, `setup_axis_pick.rs`, `stock_axis_rows.rs`) are
  in the setup and fixtures family. `machine_axis_chain.rs` is in the
  tools and machine family.
- The one raw call left in the family, the read-only "Always on" value of
  the multi-axis deburr page's "Shank and holder guard" row, uses
  `list_row`. It paints the same text.
- The raw-call guard (`layout/form_seam_tests.rs`) gets a `multi-axis`
  family at 0.
- The branch is stacked on the turning slice (PR #4218): the two slices
  change the same guard rows.

**Why.** One seam for rows and fields lets a look restyle every page at
once (#655).

**Considered and dropped.**
- Add a value-text helper (`value_row`) for read-only values. `list_row`
  already is the seam's read-only text, and one call site does not need a
  second helper.
- Put this family into the turning PR. The issue asks for one PR for each
  family.

**Cost to change later.** Small: one call site and one guard row.
