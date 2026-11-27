## #1533 - Pin each operation-pick exemption

**Decided:** Extend the existing `pick_mode` structural suite. Give each armed
source a fresh shell with Contour1 open off Geometry. Check that pausing or
disarming the source restores the lock. Keep the existing boundary test.
Include the Start/Exit exemption that current main added after this issue.

**Why:** Each test must fail when its own term is removed from
`interact::op_pick_locked`, without another armed source masking the removal.
Run each removal separately and restore production source after each run.
For the box guard, send actual modified pointer drags in Edge and Face modes,
with Command and Shift. Assert no box, no selection change, and no dialog
change. Check that Geometry still arms the same mode and modifier.

**Dropped:** A combined all-sources test can hide a missing exemption.
Pixel snapshots cannot prove the lock condition. Changing correct production
behavior to create a failing baseline would test the wrong change.

**Cost to change later:** Update the test for the affected source when its
arming contract changes. There are no runtime changes or new dependencies.
