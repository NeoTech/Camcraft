## #3356 - Face has no pass order: the "Pass starts at" row says so

**What was decided.**

- Face does not get a `pass_start` field. Face cuts serpentine rows. The
  rows go from the low Y edge of the face to the high Y edge
  (`machining2d::face::face_over`, `band`). The first row and the last
  row are both at an edge of the face. There are no rings, so there is no
  "inside" and no "outside" to start from.
- The issue gives this choice: "If Face has no pass order, its row says
  so by name, and the issue records why." This PR takes it.
- The Face Passes tab shows the shared `Pass starts at` row
  (`pass_start_row::no_pass_order_row`). The combo shows Auto and is
  disabled. The line under it says why, in view (not only on hover).
- MCP: `edit_operation` with `pass_start` on a Face is refused by name
  (`mcp/edit.rs`): "Face has no pass order: its rows run edge to edge".
  Before, the refusal was the generic "unknown field". Every word is
  refused, `auto` included, because the field does not exist on Face.
- The project file is unchanged: Face writes no `pass_start` key.
- Face output does not change. No golden moves.

**Why.**

- An Inside or Outside order for rows has no clear meaning. A
  centre-out order would break the serpentine: each row would need a
  lift and a link through the clearance plane. That gives more rapids
  and no less engagement. The first row of a facing pass is already at
  the stock edge.
- A field that is stored but changes nothing is a misleading control.
  A disabled row with its reason is honest, and the user sees that the
  shared option exists.
- No new field on `OpParams::Face` keeps about 40 constructors and the
  project DTO unchanged.

**Considered and dropped.**

- A `pass_start` field on Face that the planner ignores: it misleads
  the user and the agent.
- A centre-out ("Inside") serpentine: new motion with more lifts, and no
  machining reason for it.
- Reversed rows (high Y first) as "Outside": both ends are an outside
  edge, so the name would be wrong. The climb option already sets the
  row direction.

**Cost to change later.** If a future Face strategy has rings (for
example a spiral facing), it adds the field with the shared type and
key, passes `Some(&mut pass_start)` to `pass_start_row`, and removes the
refusal in `mcp/edit.rs`. The start point and exit point of #1448 go
beside the row in the same way.

**Review repair, 2026-11-22.** The latest review of PR #3387 found an
inherited slotting defect and a UI validation failure. Merge corrected
parent `9939a6238` and upstream main `68d645261` to address these findings.
The parent replaces whole-loop ranking with level curves from one side.
Its pins measure engagement along the whole level, and its claims state
where full-width cuts can still occur. Upstream main contains the fix for
the post dialog overlap. Use these existing corrections to keep the repair
within the latest review. Do not introduce another planner. Verification
results are recorded on the PR after the checks run.
