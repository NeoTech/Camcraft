## #1198 - drag-to-move handles (slice b)

- **Handles appear only in the bar's Move mode, off by default.** A new
  seventh bar button (a four-armed cross) toggles `State::move_handles`;
  the selected part then wears three axis arms ending in balls. Why: the
  issue ties Move/Rotate to the bar, and handles always on would put grab
  targets in the way of every click-drag orbit on a selected part.
  Dropped: handles whenever a part is selected (clutter, accidental moves).
- **Only parts are dragged in this slice** - one selected, shown, non-
  fixture model. Fixtures keep the Fixtures page's `MoveFixture` (their own
  locate and clash laws), the stock follows the part through `MoveModel`
  exactly as for a typed move, and the work origin keeps its page (issue).
  Fixtures could join later through `MoveFixture`; the drag law in
  `crate::move_drag` is command-agnostic up to the release.
- **No new app command, so no new MCP tool.** The release queues the UI
  verb `CommandKind::MoveModelBy { name, delta }`, which the drain turns
  into the same `MoveModel::new` the Model page's fields run, with the
  rotation copied bit for bit (no Euler round trip - a round trip could
  read as a turn and re-fit a square stock, #571). The MCP `move_model`
  tool already places a part through `MoveModel`, so agents can do every
  move a drag does.
- **The delta is in the shown frame.** A drag happens in the active
  setup's view; when that setup holds the part turned (#326), the shown
  delta is rotated back by the alignment's inverse before it is added to
  the document position, so the part moves along the axis the user sees.
- **The drag distance is the closest approach of the cursor ray to the
  axis line**, measured from the press point, so the part stays under the
  cursor in perspective and ortho alike; when the axis points straight at
  the eye the drag holds its last value rather than jump.
- **Arms are a constant 70 px on screen**, scaled from the camera's right
  vector at the gizmo centre (not along the arm, which degenerates when an
  arm points at the eye). Balls draw at 7 px, hit at 11 px.
- **The preview moves only the dragged body**; a following stock jumps to
  its new place on release (the `MoveModel` law decides what follows).
  The distance shows just above the bar while dragging; typed values and
  snapping are slice (d).
