## #3212 - Use one MCP key for EDM codes

**Chosen:** MCP machine definitions use `edm.codes` in both `edit_machine`
and `get_job`. The internal `Machine.edm_codes` storage stays unchanged.
The MCP edit rejects the legacy top-level `edm_codes` key by name.
The edit reuses `codes_sane` to validate code names and text before mounting.

**Why:** Machine files and wire post diagnostics already name `edm.codes`.
One visible key lets callers edit the same definition that they read.
Rejecting the storage key prevents two competing maps in one patch.
The existing undoable mount remains the command that applies the edit.

**Dropped:** A change to the shared serde schema would also change embedded
project machine definitions. This fix needs only an MCP view conversion.
An alias for the top-level key would add precedence rules without a need.

**Cost to change later:** A shared schema change needs project compatibility
tests. An alias needs an explicit conflict rule and MCP tests.
