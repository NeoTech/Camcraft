# 1440 - Tool list page follows the WCS page's layout

- Chose: three titled groups (Tools in this setup, Time and travel, Export) over `layout::group`; the per-tool readout, which was space-padded text lines, is now a real grid with a header row (Tool, Ops, Changes, Cut, Rapid, Est (min)) and a "Setup total" line. The page moved into `tool_list_page.rs` (with `tool_table`, from `setup_pages.rs`, and `tool_list_section`, from `pages.rs`, which is far past the 400-line law).
- Kept: both empty-state messages verbatim, in the same places ("No operation in this setup uses a tool yet." / "Tool list: no generated operation in this setup yet."), and every command and export as before; pinned by a structural test.
- Dropped: equal-width `even_row` for the two export buttons. Open PR #1438 routes both through `browser_files::button`, which carries the browser build's refusal and cannot sit in an `even_row`; reshaping them now would either drop that refusal or conflict. The two existing rows are reused unchanged under an "Export" group; revisit once #1438 lands.
- Dropped: a "Setup sheet" group of its own - the sheet export is one row.
- Not touched: Machine, Fixtures and Setups pages, one PR each.
