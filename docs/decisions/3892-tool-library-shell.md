## #3892 - Tool library dialog: U1 dialog shell and tools list

**Decided.** The tool library is a non-modal in-app `egui::Window` with the
id `tool-library` and the title "Tool library"
(`crates/ui/src/egui_app/tool_library_dialog/`). Its open state is
`State::tool_library: Option<ToolLibraryView>` (view state, no command).
All doors (the ribbon's Tools cell, the View menu's "Tool library…", the
ribbon gate route for "no tool" and "no fitting tool") go through
`actions::target` with the empty tool name, which now opens the window and
keeps `ui_target`. `LayoutRequest::ToolLibrary` is removed. The `Tab::Tools`
variant stays because a saved panel setup can name it.

**Why.**
- The design of #3663 (section 6, decision 1) asks for a non-modal window,
  like the tool shape window.
- A saved panel setup that holds the old tab must still load. The dock is
  serde data, so the variant must stay. `layout::setups::decode` drops the
  tab with `DockState::retain_tabs` before the permanence check. A float
  that held only that tab goes with it. The drop is at load, not in
  `settle`, because a load is the only path that can bring the tab back.
- The Tool type and Name filters stay in the existing fields
  `State::tool_catalog_kind` and `tool_catalog_name`, not in
  `ToolLibraryView`. This keeps the filters between two openings and keeps
  the change small. `ToolLibraryView` holds the source only.
- A new window opens on the Document source when the document has tools,
  else on the Catalog. An open window keeps its source when a door is used
  again (a second click does not close or reset it).
- The Tool type control stays a combo box above the list. The plan named
  "categories"; a left category column waits for the rail of U2, where the
  window gets its two-column form.
- The Document list has the #3663 columns T, Tool, Kind, Ø, Flute length,
  Flutes, Holder and Recipes. The gauge length column waits for U4, which
  owns the assembly (holder plus stick-out). "Edit" shows the tool in
  Properties (view state), as the setup's tool list does, until U2 puts the
  editor in the window.
- The Catalog rows keep their label form "`<name>  Ø<d> <kind>`" and the
  "Add" button (the `AddTool` command), so that the catalog tests and the
  flows that find a row by label do not change.

**Dropped.** A filter state in `ToolLibraryView` (moves the existing
fields for no gain now). Removing the `Tab::Tools` variant (breaks the load
of a saved setup). A drop of the tab in `settle` on each frame (no path
adds the tab after a load).

**Cost to change.** Small: the window and its state are in two new files.
U2 replaces "Edit" with the editor in the window.
