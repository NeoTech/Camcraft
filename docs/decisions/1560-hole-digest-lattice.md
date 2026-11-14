## #1560 - an op's stamp digests its hole list in the lattice form the file stores

**Decided:** the Drill, Bore and Thread arms of `OpParams::stamp_with`
digest each hole (and each radial hole) through the new
`operation::hole_digest`, which writes every hole float snapped to the
1 um lattice (`cl::lattice`) - exactly the snap `HoleDto::from_hole` and
`CrossHoleDto::from_hole` apply when the project file writes the list.
The radial direction stays verbatim, as the file keeps it. A floor typed
as `0.190 in - 0.040 in` (3.8099999999999996 mm) and the 3.81 the file
reloads now stamp alike, so an op generated Ok and saved reopens Ok.

**Why:** this is #1351's remedy (PR #1352, `path_sig` hashes moves on the
lattice) applied to the other place where the in-memory form and the
stored form of one value disagree. It is one function on the one path
every writer shares: the MCP `edit_operation`, the GUI hole table, a
script, an undo/redo, `detect_holes`, and files written by older builds
all reach the stamp through `stamp_with`. On-lattice holes (everything
recognition emits) hash bit-for-bit as before, so no stamp that matched
before changes; a change of half a micron or more still moves the stamp
(pinned in `hole_digest`'s unit test). Nothing posted reads the stamp.

**Considered and dropped:**
- Snap the typed holes when they enter the document (`EditOperationParams`
  / `CreateOperation`, the brief's first preference): the document, stamp
  and file would hold one number, but the snap would have to be repeated
  at every door a hole list comes through (create, edit, paste, the GUI
  table, scripts, load of an older file), and a door added later without
  it reopens the bug. It would also move the generator's input by up to
  half a micron, a posted-output change for a staleness fix - the reason
  #1352 dropped snapping in `Toolpath::new`.
- Compare stamps with a tolerance: loosens a safety comparison.
- Re-stamp on load: hides a real change made between save and load.

**Known edge:** a file already saved with a stamp taken from an
off-lattice hole (before this fix) still reads Stale once on reopen; one
regenerate stores the new stamp.

**Sweep:** the only *params* the file snaps are `HoleDto` and
`CrossHoleDto`, both covered here. The other `lattice(...)` calls in
`project/` (`MoveDto`, `CompSpanDto`, cycle, thread-cycle and edge-break
DTOs) store a generated cache, not stamp inputs; the moves feed
`path_sig`, already lattice since #1352, and nothing digests the rest.
Every other params float is written verbatim (serde's shortest form
round-trips the bits, #169).

**Cost to change later:** one module; the stamp's byte layout per hole is
unchanged.
