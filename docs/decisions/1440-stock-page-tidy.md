# 1440 - Stock page follows the WCS page's layout

- Chose: titled groups (Shape, Size, Position and rotation, Fit to models) over `layout::group` / `prop_grid`, and the three actions as `wcs_page::even_row` equal buttons. Moved the page into `stock_page.rs` (pages.rs was far past the 400-line law).
- Kept: the "Per side" checkbox as a plain checkbox (its accessible name is the caption; `check_row` would not take an accessibility click), every label and command as before.
- Dropped: the separate "Shape" radio label (the group title says it); a collapsed-by-default Fit group (the fit margin is a common edit, so it stays open).
- Not touched: the other four pages (Machine, Fixtures, Tool list, Setups), one PR each.
