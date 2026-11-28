## #962 - Complete the remaining MCP skill rows

**Decision:** Document `set_units`, `import_model.facet_budget`, and the missing
`get_job` units, machine and setup-post fields. Remove `SetUnits` from the gaps.
Keep the other requested rows: they already exist on `main`.

**Why:** The issue thread requests these fields after their implementation PRs
merged. The current registry, schemas and commands define their syntax and undo
rules. Add focused Python documentation tests beside the existing skill tests.

**Considered and dropped:** Do not add duplicate rows or change Rust behavior.
Do not bump `PROCESS_VERSION` in this PR. The current `AGENTS.md` assigns the
version bump and announcement to the merging session. The PR declares
`Process change: yes` instead of following the issue's old bump instruction.

**Cost to change later:** Update the affected skill rows and their documentation
tests when the public MCP schema changes. No runtime migration is needed.
