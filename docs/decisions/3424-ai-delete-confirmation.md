## #3424 - Confirm native AI delete calls

**Decided:** Require inline Allow/Deny for every `delete_*` tool in Native
AI chat mode. Reuse `requires_confirmation` and the existing per-call gate.
Deny returns the refusal to the model and leaves the document unchanged.

**Why:** `delete_operation`, `delete_tool`, `delete_setup`, and `delete_model`
remove document entities. Native chat previously dispatched them without
confirmation. The prefix also covers future tools with the same delete naming.

**Considered and dropped:** A list of four names would leave future delete
tools unguarded. Confirming every native mutation would expand the change to
ordinary additions and edits. Changing the app's live MCP gate would affect
other live MCP callers beyond AI chat.

**Cost to change later:** Adjust the local predicate and update the registry
policy test and the scripted denial/approval test.
