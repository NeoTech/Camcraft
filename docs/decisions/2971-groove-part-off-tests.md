## #2971 - Named tests for the groove and part-off catalog case

**What was decided.** Add `catalog_turning_groove_part_off::regenerates`
and `catalog_turning_groove_part_off::rebuilds` to the existing catalog
suite. Use the same `run_case` helper as the other cases. Add a focused
test that requires the case directory and both names in the test listing.

**Why.** The groove and part-off example merged after the catalog acquired
its named-test contract. The example had no wrappers for that contract.
The shared helpers already check regeneration, script rebuilds,
verification, timing, and byte-for-byte NC output.

**Considered and dropped.** Do not weaken the catalog contract or skip the
case. Do not duplicate the regeneration or rebuild logic. Do not change
the example fixtures or the posted program to fix missing test registration.

**Cost to change later.** Small. A case rename must update its wrappers
and the focused listing test together.
