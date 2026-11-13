# #661 tool icon wiring (slice 3)

Decision taken unattended by the issue worker.

- **Chosen:** the Tool library catalog rows (`pages.rs`, `tool_page`) and the
  operation page's tool picker dropdown (`op_page.rs`, now
  `tool_icons::tool_combo`) show each kind's existing `tool-*` glyph at 16 px,
  before the unchanged label. The picker's selected-tool glyph beside the combo
  stays.
- **Why a new combo helper:** `combo_owned` takes plain strings, so the glyph
  per dropdown item could not go through it. `tool_combo` is the same
  `ComboBox` with a glyph in front of each `selectable_value`; the label text
  is unchanged, so accessible names do not move.
- **Pure choices pinned:** `catalog_row_icon(&Tool)` and
  `picker_item_icon(Option<&Tool>)` hold the mapping; the draw helpers only paint
  what they return. The structural test drives every kind from `every_tool_kind`.
- **Considered, dropped:** a new icon or a changed icon (out of scope; the
  table is shared with the browser rows); a headless world fixture to pin
  `named_glyph` end to end (the world lookup is two existing calls, and no
  harness in tests/ builds a tool world cheaply).
- **Not in this slice:** section 5 workspace tabs (waits on #659) and the
  redraw pass after the maintainer has seen the set in the app.
