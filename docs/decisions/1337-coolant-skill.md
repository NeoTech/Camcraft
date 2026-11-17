## #1337 - Document the operation coolant MCP tool

**Chosen:** Add `set_operation_coolant` to the MCP skill's operation tool
list, using the implemented schema and command behavior. Document required
non-empty strings, all five modes, accepted aliases, named input refusals,
undo, cached-path validity, and the warning that precedes a post refusal
when a machine's coded entries do not serve the selected mode. Keep the
implemented acceptance of a mode that is already set.

**Why:** PR #974 shipped the tool without a skill entry. Agents need the
same parameters and refusal behavior that the MCP implementation exposes.
A standalone Python unittest compares the row's arguments and modes with
the Rust schema and pins its key behavior descriptions. It proves the
documentation repair without an app rebuild or new dependencies.

**Dropped:** Changing coolant runtime behavior or refusing an unchanged
mode would expand this documentation issue. An app integration test would
retest the already shipped tool rather than pin the missing skill row.
Do not bump `PROCESS_VERSION`; the merging session owns that bump. The PR
declares `Process change: yes` instead.

**Later cost:** A coolant schema or behavior change must update this row
and its documentation pin together. The test uses only Python's standard
library and runs directly from the skill directory or repository root.
