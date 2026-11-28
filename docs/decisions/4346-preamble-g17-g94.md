## #4346 - G17 and G94 in the Fanuc and Doosan DNM section preambles

**What was decided.** The section preamble of `fanuc.cps` is now
`G90 G17 G94 <wcs> G40 G49 G80`. The preamble of `doosan-dnm.cps` adds
`G94` at the same position: `G90 G17 G94 <wcs> G40 G49 G80`. The Fanuc
post also sets its plane memory (`planeG`) to 17 after that block, so the
memory agrees with the control.

**Why.** A FANUC control can keep G18 / G19 or G95 from an earlier
program, from MDI, or from its power-on parameters (FANUC Series 0i-F
operator's manual B-64604EN-2). Then the XY arcs, the cutter compensation
and the drilling cycles run in the wrong plane, and an F word in mm/min is
read as mm per revolution. Neither post writes G93 or G95, so G94 is always
the correct mode. The two words go in the one preamble block that already
cancels the other modal groups, so each section states its modes before
its first motion and the line count of a program does not change.

**Test changes.** `conformance::the_dialects_actually_differ` compared each
pair of mill posts on `drill_peck` only. Before this fix, Fanuc and Doosan
DNM differed there only by the missing G17, which was the defect. With the
fix the two write the same drill program: each Doosan DNM word (brake,
conveyor, AICC) is opt-in. The test now requires that a pair that is
identical on `drill_peck` differs on a rotary index on the DNM machine file
with the `rotaryBrake` option. The test still finds two scripts that are
copies of each other.

Literal asserts that read `G90 G54` / `G90 G55` as "the work-offset
word" now read `G94 G54` / `G94 G55`. Their purpose does not change.

**Considered and dropped.**
- A separate `G17 G94` block before the preamble. Dropped: it adds a line
  to each section for no gain. Fanuc reads all of these words in one block.
- An exemption of the Fanuc / Doosan DNM pair from the diversity test.
  Dropped: an exemption would also let a future copy of the script pass.
- The `.camproj` files that `cargo xtask catalog` rewrote (the project
  version 83 to 84) were put back. That change is not part of this fix.

**Cost to change later.** Low. The change is one block in each post, and
the goldens are regenerated with the usual tools.
