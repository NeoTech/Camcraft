# 1440 - Machine page follows the WCS page's layout

- Chose: three titled groups over `layout::group` - Which machine (the own / document-machine choice and its hint), Machine file (a one-line helper, the Library combo as a label | control row, then the overview grid or the no-machine message) and Change machine (Load other / Edit / Clear as equal-width `even_row` buttons; the one "Load machine (.mch)" button with none). The page body moved into `machine_page.rs` (`machine_section` left `pages.rs`, which is far past the 400-line law); `ownership` and `Mount` in `setup_pages.rs` became `pub(super)` so the page can group it, and its trailing separator moved to the Post page, its other caller.
- Chose: the bare "Machine" caption above the Library row is gone - the group title says it - and the Library row sits in a `prop_grid`.
- Kept: every label, command and message (Library, the seven overview rows, Source, "No machine loaded - the checker states, it cannot judge.", all button labels); the inactive-setup activate prompt and the raised-rotary rows (own heading, drawn last) are untouched.
- Dropped: a group of its own for the rotary rows - they are `setup_rotary_page`'s, drawn only for a work-turning rotary, and another PR's files overlap there (#821 lineage).
- Scope: the properties-pane page only. The Machine editor window (`machine_editor.rs`, `machine_edm_editor.rs`) is a separate floating window with its own layout; it would need its own slice, and open PR #1218 edits `machine_editor.rs`.
- Not touched: Setup (Settings) page, one PR each.
