## #826 - plate follow-ups

**Decided.** A plate whose mesh changes outside `ImportFixture` (a same-name
re-import, a script `cam.import`, `RepairModel` - all ride `UpdateModel`)
drops its grid and unmounts its riders in that same undo step, instead of
re-validating the old grid against the new mesh. `square_edges` accepts a
staggered plate when the least OR the greatest edge on each axis is straight.

**Why.** The grid was measured on the old mesh; dropping it is the safe,
simple reading (the user re-detects, and the riders keep where they stand).
Re-validating would need a second detect pass and a rule for which riders
survive. For the edges, a clamp ear or chamfers spoil one side of a plate,
not both; a turned plate still has no straight edge on either side.

**Considered and dropped.** Re-detecting the grid inside `UpdateModel`
(an algorithm in a command constructor, and it can refuse). Accepting on
either axis only (would let a 45-degree diamond through).

**Cost to change.** One field in `UpdateModel` and one function in
`plate/detect.rs`.
