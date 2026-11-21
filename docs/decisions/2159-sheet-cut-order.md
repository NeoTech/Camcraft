## #2159 - Global sheet cut order

**Chosen.** Keep the planner in the geometry crate. Its API accepts placed
closed loops, open points, and stable feature IDs in sheet coordinates.
The planner does not depend on the placement kernel's output types.
It adds no dependencies or serialization changes.

Stack on #2808 because it changes the shared nesting entry file.
Add the planner export beside the placement API.
Keep the placement functions unchanged.

Build one containment forest on the geometry integer grid. Refuse touching
or crossing closed boundaries and chains that cross those boundaries.
Check self-intersections and adjacent retracing before containment.
Use checked `i128` area accumulation and refuse values outside its range.

Traverse the forest in iterative post-order. Sort siblings by increasing
enclosed area, then stable ID. Resolve `Auto` closed sides by the parity of
material-releasing ancestors. Retain non-releasing geometric parents for
scheduling, but exclude them from this depth because they do not separate
material. Reject counting all closed ancestors: an engraving or blind pocket
would reverse the side of a nested through hole.
Resolve open chains to `OnLine`. Refuse invalid side requests by name.

Schedule four global phases: marking, internal cuts, rough profiles, and
final breakthrough. Each feature states its `release_phase` explicitly.
A blind pocket can retain stock through its internal passes. A through cut
can release material in the internal or breakthrough phase. Refuse any
schedule that cuts a feature after its material or an ancestor is released.
Refuse conflicting requests instead of changing their phase.

Report circular-tool corner limits from `cut_width / 2`. Keep the original
vertex indices in the diagnostics. The planner does not offset cut paths.

**Why.** Global containment protects parts placed inside another part's hole.
Small parts cut first while the surrounding sheet still supports them.
Explicit release information distinguishes a pocket from a through cut.

**Dropped.** Input order and position do not set sibling priority.
Vertex-only containment misses boundary crossings. Region normalization can
remove a slit or spike, so it does not validate the requested cut path.
A phase alone does not determine release. Unsafe partial schedules are refused.

**Cost to change.** The placement kernel will need an adapter for this API.
Changes to phase rules or sibling priorities need new policy pins.
