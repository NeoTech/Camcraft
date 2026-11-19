## #2599 - New tilt angles on a legacy operation take the current frame basis

**What was decided.** `EditOperationParams` steps a tilt's `basis_version`
from 0 (the pre-#2133 frame, kept for a file without the key) to 1 when the
edit changes the tilt's A or B angle (`OpParams::with_stepped_tilt_basis`).
An edit that keeps the angles (feed, depth, origin) keeps basis 0. A loaded
operation that nobody edits keeps basis 0 and posts what it posted before.
The UI page and MCP `edit_operation` both go through this one command door.

**Why.** PR #2157 fixed new tilts (the JSON door and the UI set basis 1), but
a tilt loaded from an old file keeps basis 0, and both the operation page
(it edits the loaded `Tilt` in place) and MCP `edit_operation` (it merges a
partial patch into the loaded fields) carried basis 0 onto new angles. Set
to A0/B180, such an operation posted its plane XY turned 90 degrees
(`G1 X0 Y-10` for a slot from (10,0) to (30,0)), while a new operation at the
same index posted `X10 Y0` - the defect of #2599, still reachable on main.
New angles are a new index, so the old plane coordinates no longer describe
anything the user drew for that index.

**Dropped.**
- Migrate every legacy tilt to basis 1 at load: this silently moves the cut
  of an old file that the user does not touch (a breaking change of what an
  old file posts; `tilt_basis_compat` pins that it must not).
- Refuse a legacy tilt whose basis-0 frame differs from basis 1 at load or
  at post: the old file's frame is the one its preview, its saved geometry
  and its saved program (`examples/first-4axis-part.nc`) use, so it is
  correct for that file. A refusal would also block opening (`is_sane` runs
  at load) or force a re-edit without a cause.
- Step the basis also on `CreateOperation`: a copy of a legacy operation has
  the same angles as its source and must cut the same as the source.

**Cost to change later.** One call in `commands/operation/edit.rs` and one
small file (`operation/types/tilt_basis.rs`). Undo restores the old tilt with
its basis, because the command stores the whole previous operation.
