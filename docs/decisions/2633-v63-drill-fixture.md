## #2633 - Load the historical v63 drill reference

**Decision:** Remove `break_through_from` from the v63 Drill fixture in
`sketch_project.rs`. Assert that loading supplies `BreakThroughFrom::Tip`,
keeps the stated `break_through: 0.0`, and saves the reference as `"tip"`.
Compare the complete migrated JSON after adding that key and updating the
version. Keep the byte-stable current-format round trip.

**Why:** The reference key first appeared in v68. Changing only the version
of a current save does not exercise the missing-key migration. The existing
historical fixture in `break_through.rs` removes the same key. The migration
already supplies Tip; this change corrects coverage, not production behavior.

**Considered and dropped:** Renaming the test would describe its old coverage,
but would not test the historical migration. A production change is not needed.

**Cost to change later:** Update this fixture and its assertions if the project
format changes. No runtime interface or dependency changes are required.
