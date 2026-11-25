## #3894 - The tool library window shows the tool as a 3D model and a to-scale side view

**Scope (decided by the user, 24 Nov 2026).** The maintainer said: "as
long as we keep the 3D preview bit, im aiming to develop on that down the
line" / "as running weirdly shape tools requires models" / "not just a
spun profile". The old U3 (a large 2D drawing with painted flute helix
lines) was replaced by a 3D model view (the default) and a to-scale side
view with dimension lines. The painted helix lines were dropped. Helical
flutes on the 3D model are the follow-up #3908.

**What was decided.**
- The renderer of the body placement view moved into its own module,
  `egui_app/tool_model_view.rs` (`View`, `project`, `gesture`, `apply`), and
  the placement view (Pick tip / Pick axis, `placement_view`) uses it
  unchanged. The orbit is one value, `state::Orbit`; the body draft keeps
  its four fields, so no older test changes.
- The preview stands at the top of the window's editor, above the Tool and
  Details tabs, so it shows on the two tabs. Its height is 40 % of the
  editor (140 to 320 px); the tabs' rows keep the rest.
- A tool with a landed body shows the steel revolved up to the body's foot
  and then the body's own facets (as `tool_body_view::assembly`). A tool
  without one shows the steel and the holder cylinder (`SpunProfile::
  holder_of`), both revolved. The steel is the landed drawn cutter carried
  onto the draft numbers (as the editor's Apply carries it), else
  `Tool::profile` up to the shank top.
- The window's preview reads the landed body and cutter from the document
  by the draft's name. It never calls `sync_tool_body_ui` or
  `sync_tool_cutter_ui`: those drafts belong to Properties and the shape
  window, and the window can show another tool.
- Frame cost: the model is a thread-local memo keyed on the draft tool, the
  document's tool, the landed cutter and the landed body (its facets by
  `Arc` identity, its frame, library entry and source). The projected
  screen mesh is a second memo keyed on the model mesh (an `Arc` it holds),
  the rect and the orbit. Counters `model::builds` and `preview::projects`
  pin "a still frame builds nothing".
- Side view: "stick-out" is the tip to the holder face (the body's foot,
  or the shank top); "gauge" is the tip to the holder top. The holder
  dimension is the largest band diameter and the body length, or the stated
  `holder_d` x `holder_len`.
- The Properties preview (`preview_inputs`) shows a landed drawn cutter on
  every tab, not only under "Draw cutter" (#93 leftover).
- Review of PR #3914 (Frame cost): the Properties cutter check and outline
  are memoised in `egui_app::tool_preview_inputs`, keyed on the draft tool
  value and the cutter draft points, not on `doc_version`. `preview_inputs`
  moved there out of `tool_profile_page` (415 -> 370 lines), along the seam
  "what the Properties preview draws" versus "the shape editor's pane".

**Dropped.**
- Making `app::tool_body_view::revolve` public: a change in `crates/app`
  adds the app suites to the quick gate for a display-only change. The UI
  has its own 20-line revolve with the same ring count. To change later:
  make the app one public and delete the UI copy.
- A preview to the right of the rows: the window would need about 900 px
  and would then cover the Properties pane.
- A cache in egui memory: the steering file says that no widget holds a
  cache; the memos key on the inputs.
