## #732 - MCP cutting-data tools

**Decided:** three tools in `crates/app/src/mcp/cutting_tools.rs`
(`set_work_material`, `suggest_cutting_data`, `apply_cutting_data`) and a
`work_material` field in `get_job`. The tools take the operation as
`operation` and return the suggestion fields in snake_case with units in
the description. `set_work_material` requires the `material` key so an
omitted key is not read as "clear".

**Why:** each write queues the very command the Cutting data page queues, so
undo/redo and refusals are the GUI's.

**Dropped:** returning `SetWorkMaterial::seeded()` (#890, PR #1076) - not on
main yet; add it when that merges.
