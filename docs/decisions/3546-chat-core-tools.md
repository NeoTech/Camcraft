## #3546 - Offer the existing editing tools in local chat

**Decision.** Add `rename_operation`, `edit_tool` and `set_work_material`
to `CORE_TOOLS`. Keep the existing 25 entries. Read all schemas from the
live MCP registry, as before. Change no tool implementation, validation,
confirmation policy or undo command.

**Why.** Native local chat cannot request these existing tools when their
names are absent from its advertised set. The three additions support the
reported rename, tool-edit and material requests without a new interface.
The core set now has 28 tools. Its serialized schemas remain under one
third of the full table, as the existing budget test requires.

**Verification choice.** Extend the provider test to check both tool sets
in all four provider formats against the live registry. Extend the real
panel test to require the three names in both local payload modes. Use a
scripted native OpenAI response to rename `Face1` to `Roughing` through the
panel worker and live MCP inbox. Check the document label, stable key,
returned call identity and single history entry. Click Undo Turn and check
that the original label and history depth return.

The provider and panel assertions fail on the unchanged core table. They
pass with the additions. The scripted test proves that the benchmark's
rename task can execute. It does not measure model selection or claim a
rerun of the 12-model benchmark. No live inference service is required for
this deterministic contract.

**Dropped.** Do not widen the whole local tool table or add dynamic tool
selection. Do not change prompting, catalog filtering, nested edit
semantics or model links. No visible layout changes need new screenshots.

**Cost to change later.** Change the core list and its exposure and budget
assertions. The live registry remains the only schema source.
