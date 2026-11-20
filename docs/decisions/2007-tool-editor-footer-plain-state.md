## #2007 - Tool editor fixed footer, plain-English state lines and label-left shape rows

**What was decided.**

- The Properties tool page (`crates/ui/src/egui_app/tool_settings.rs`) now
  owns its own vertical scroll area (`tool_page_scroll`) and keeps its
  footer fixed below it: `Apply` (or `Add tool` on a new tool) beside
  `Close`, with any name-block message or `ErrorScope::Tool` refusal
  drawn directly above that button row. `pages.rs` delegates
  `UiTarget::Tool` directly before its shared scroll area, matching
  `op_page`.
- The Catalog group's `Tool number` row (`tool_settings_grid.rs`) drops
  the raw `"(0 = next free)"` label suffix and moves that detail into the
  row's hover tooltip (`prop_row_hint`), matching the `Material group` and
  `Holder` rows.
- The Body tab (`tool_body_page.rs`, `tool_body_lathe_page.rs`) aligns
  `Tool axis`, `Tip offset`, and `Body frame` as label-left property rows
  and replaces raw/jargon state strings (`"body: ..."`, `"draft: ... (not
  applied)"`, `"none - the holder is/cylinder is judged"`, `"placed: foot
  ..."`, `"placement: ..."`) with plain-English lines matching the
  Profile tab (`"In use: ..."`, `"Ready to apply: ... (not applied yet)"`,
  `"Placed from ..."` / `"Placed: ..."` / `"Placement error: ..."`).
- The shape window's holder library picker (`tool_holder_picker.rs`)
  puts `Library holder`, `Gauge length`, and `Projection` in label-left
  `prop_row` / `prop_row_hint` rows using unit-suffixed `num_row` fields,
  `tool_profile_page::editor` syncs `tool_profile_ui` before drawing the
  picker so an existing library holder shows immediately on the Body tab,
  and `pages::tool_shape_editor_window` renders `ErrorScope::Tool` above
  its fixed `Close` button.

**Why.**

After PRs #2663 (`#2014`) and #2764 (`#2397`) merged, #2007 still had
two unowned UI items from the maintainer's 2026-11-18 report and the
2026-11-19 status audit: `Apply` / `Add tool` scrolled off the bottom of
Properties at 720 px height, and several state lines and labels still
used raw fragments (`"(0 = next free)"`, `"body: ..."`, `"judged"`,
stacked `field(...)` labels in the shape editor).

**What was dropped.**

- Removing the typed `holder_d` and `holder_len` fields when no library
  holder is selected. Existing `#2016`, `#2009`, `#2014`, and `#93` pins
  require parametric cylinder dimensions as the fallback when no library
  entry or custom profile is set, and removing them would break project
  and tool-library formats without a separate migration decision.
- Removing the armed-pick 3D placement view in `tool_body_page`. It
  already stays hidden at rest (`BodyPickMode::Off`) and only renders
  while `Pick tip` / `Pick axis` / `Pick radial` / `Pick spindle` needs
  click targets on the mesh.
- Moving `Delete` and `Edit tool shape` out of the top of the tool page.
  Keeping them under the `Tool: <name>` heading preserves `layout_page_tool`
  and keeps them reachable without scrolling while `Apply` / `Add tool`
  and `Close` share the bottom bar.

**Cost to change.**

The tool Properties footer lives in `tool_settings::footer`, the shape
window error footer in `pages::tool_shape_editor_window`, and each
updated label/state line is pinned in `tool_properties.rs`,
`tool_body.rs`, `tool_body_lathe.rs`, `tool_holder.rs`, and
`tool_holder_reference.rs`.
