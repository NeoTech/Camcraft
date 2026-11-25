## #3275 - V-Carve default and catalog block boundary

**What was decided:** State the V-Carve Stepdown default as 5 in the guide.
Make `section_lead` accept only a leading block with the exact generated
catalog format. The block must contain the title and either one or more
Markdown link bullets or the no-case message, followed by its end marker.

**Why:** The value matches the default supplied by `Carried.stepdown`.
The strict block format prevents a missing end marker from consuming
section text and the end marker of a later block.

**Considered and dropped:** An unrestricted search for the next end marker
can hide malformed content. Runtime operation changes are unnecessary
because the application already uses the correct default.

**Cost to change later:** If the catalog generator changes its block format,
update `section_lead` and its tests to accept the new format.
