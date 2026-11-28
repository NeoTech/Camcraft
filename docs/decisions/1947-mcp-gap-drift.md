## #1947 - Finish the held MCP skill correction

**Decided:** Keep the existing shipped rows for document units and face lay.
Add the program number and comment to the shipped section. Replace the stale
gap list with its maintenance rule. Link the generated reference and document
the reference and local-prompt commands.

**Why:** PR #3031 supplied the catalog and prompt implementation but left this
skill correction open. The current registry includes `set_units`,
`set_program` and `lay_part_on_face`. Their handlers use undoable commands.
The existing units and face-lay rows need no duplicate entries.

**Coverage:** Extend the existing `mcp_doc_drift` suite. Scan the gaps section
for snake_case and CamelCase identifiers, and single-word tools in code spans.
Compare candidates with the live catalog. Stop at the next section or
separator. A control test checks these boundaries and excludes ordinary prose
such as "post" and "undo". Keep all six existing drift tests. A separate pin
checks the reference link and the two CLI commands.

**Dropped:** The suggested scan of every prose word would treat ordinary
"post" or "undo" as tool names. No catalog regeneration, tool implementation,
guide rewrite, or geometry change is needed. No new test binary is needed.

**Cost to change later:** Update the focused scanner controls if the gap list
uses a new identifier format. Update the reference-link pin if the skill or
catalog path moves. The production tool table and runtime behavior stay
unchanged.
