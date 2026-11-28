## #3145 - Pin the posed first-traverse seam

**Decided:** Add two private unit tests and one test in the existing
`first_traverse_table_tilt` integration suite. Keep all runtime code unchanged.
The unit tests cover a cross drill entering `turn_of` through `posed_kind`
and the fixed table height for `PosedTurn::Line`. An axial drill, a missing
machine and a non-mill posting order are negative controls.

**Why:** A cross drill on a mill stops before generation. The private seam
therefore gives a direct test of its posed-kind predicate. The line test uses
a displaced origin and a non-unit tilted axis to distinguish its height
transform from the existing sphere test.

The posting test uses an inline table-A machine with stated `nose_to_axis`
and `axis_to_table`. Its synthetic cutter has long flutes to clear the tilted
stock. Only the posed verdict blocks the low-clearance case. The test checks
the operation name and the complete diagnostic. Changing only the clearance
removes that verdict and posts the control program.

**Considered and dropped:** Do not reuse the earlier `Face1` substring check
as proof of this refusal. Its machine stops at the index-swing clause. Do not
remove warnings or weaken the tool-body checks to isolate the new test. Do
not add a machine asset, a golden file or another integration binary.

**Cost to change later:** Test fixtures and assertions only. There is no
runtime, file-format or public API change.
