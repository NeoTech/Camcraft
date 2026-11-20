## #2397 - the tool body and profile editor opens in its own window, and the old tool editor is removed

**Decided.**

- Properties shows the tool settings only: the label-left property grid of
  #2014, the one side preview of #2017, and one button, `Edit tool shape`.
- The tool shape editor is its own in-app modal window (`pages.rs`
  `tool_shape_editor_window`, state `State::tool_shape_editor`). It holds
  the holder library first, then the Body / Profile tabs, then the body
  mesh tools or the spun profile. The holder library is above the tabs, so
  a holder is the first thing the user sees and not a row under the
  profile table.
- The old combined tool editor window is gone. `State::tool_editor`,
  `pages::tool_editor_window`, `actions::open_tool_editor` and
  `close_tool_editor_window` are removed or renamed. No door opens a
  second settings surface.
- `New tool…` in the Tool library starts a blank numbered draft in
  Properties (`actions::begin_new_tool`, state `State::new_tool_form`). The
  empty `UiTarget::Tool` is the destination; the flag says which face of it
  shows - the pointer to the library, or the draft. A retarget drops the
  draft.
- Each shape control still queues the same undoable command as before:
  `Apply profile`, `Apply holder`, `Apply body`, `Clear body`, `Apply
  cutter`. No MCP tool changes.
- The shape window is a fixed frame with a scrolling body and a Close
  button that stays under it. The profile and cutter sections use two
  columns - the outline controls on the left, the points table and the
  side view on the right - so no row or button is clipped at the bottom.

**Why.** The maintainer reported that the tool page had two editors for
one tool and that the holder was only reachable under the profile table
(#2007, reopened). One surface for the settings and one window for the
shape matches how the user described the work: set the numbers, then edit
the shape. The two-column window also answers the "right half is mostly
empty" half of the report without a new layout law.

**Dropped.**

- A settings field inside the shape window. It would put a second
  editable surface back on screen; the window is the shape only.
- Removing the side preview from the shape window. #2017 keeps one side
  preview on Properties, and the shape window needs its own picture while
  it covers the dock. Two surfaces, one preview each.
- A single "Apply" in the shape window for the profile, the holder and the
  body at once. The three commands are separate undo steps today; one
  button would either hide that or merge three commands into one.
- A landing that keeps the Properties draft when the shape window closes.
  The shape drafts (body, profile, cutter) still reset on close; the
  settings draft in `State::tool_ui` now survives, because it belongs to
  Properties.

**Cost to change.** The window is one function in `pages.rs`; the shape
body is `tool_profile_page::editor` with the two columns in
`profile_section` and `tool_cutter_page::cutter_section`. The state field
is `State::tool_shape_editor`, closed by `State::close_tool_shape_editor`
and by the ghost law in `State::drain_actions`. The new-tool form is
`actions::begin_new_tool` and `State::new_tool_form`.