# 1440 - Fixtures page follows the WCS page's layout

- Chose: two titled groups over `layout::group` - "Workholding" (the quiet line that says what the checks see, each fixture's row, the machine-table line) and "Add workholding" (the adds as `wcs_page::even_row` equal buttons, two to a row: Import / Built-in vise, Accessory / Fixture plate). The page moved out of `pages.rs` into `fixtures_page.rs` (pages.rs re-exports `fixtures_section`, so the many tests that draw it are untouched).
- Chose: the "Mark X as fixture" button stays conditional (exactly one model selected) but sits on its own full-width row under the adds, so the rows above never jump when the selection changes. A separator now divides one fixture's row from the next.
- Kept: both state lines verbatim ("No workholding modeled: ..." / "Workholding in this setup's collision checks:", now the group's helper line), the machine-table line, every fixture-row control and its command, the draft retirement, and #1432's `labelled_by` names (pinned by a test that opens the Place and Place-on-plate forms and checks every input is named).
- Dropped: reshaping `fixture_rows` itself (name+position as text, Place / Locate / Unmark / Delete small buttons, vise / plate / accessory sub-forms) into a grid - those rows have their own pane-fit pins (#509) and a dozen tests, and a change that size is its own slice.
- Dropped: equal-width buttons for the per-row verbs, for the same reason.
- Not tested: clicking "Import fixture..." (it opens a native file dialog); its place in the layout is pinned, its command is unchanged.
- Not touched: Machine and Setups pages, one PR each.
