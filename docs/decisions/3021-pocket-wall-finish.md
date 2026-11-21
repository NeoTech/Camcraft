## #3021 - Zigzag pockets close with the wall and island rings

**What was decided.** Each 2D pocket that cuts as Zigzag (strategy
Zigzag, or Auto that resolves to Zigzag) now uses the rest pocket's pass
plan: the rows stand 0.01 mm inside the cutter-center area, and one
closed ring follows each boundary loop of that area (the outer wall and
each island). The row points stand on the 1 um lattice.

**Why.**
- Rows alone touch a side wall or an island only at their ends. Between
  two ends the cutter leaves a cusp over the full depth. With a 6 mm tool
  and a 2 mm stepover, the cusp is about 0.17 mm on a straight wall and
  0.75 mm on a rounded corner. Verify had no law for that material, so
  the program passed as clean.
- The rest pocket already had this plan (`pocket_zigzag::rest_passes`).
  The 0.01 mm gap keeps the row ends from crossing the ring.
- The gap puts the row ends off the lattice. A helix entry snaps its
  centre and its turn ends to the lattice. Two levels' helices at one
  off-lattice start then differed by 3.5e-7 mm, and the post check read
  their chords as crossings (`setup_align`). Lattice row points remove
  that. The snap moves a point by less than 0.5 um, inside the 10 um gap.

- With blended turns, a row-turn fillet or a ring-corner fillet can
  cross the other pass. The Zigzag pocket now holds its blends to the
  crossing law (`hold_crossings`), as the rest pocket does. A fillet
  that would cross stays a sharp turn. The ring corners of a square
  pocket then stay sharp.

**Dropped.**
- A verify law that names unfinished walls. The cut is the defect; a
  warning on every Zigzag pocket is noise.
- An Auto that resolves to Offset rings when the pocket has an island.
  It changes the strategy that the user sees, and an explicit Zigzag
  pocket keeps the cusps.
- The same rings in `pocket_avoid` (Do not touch) and in the 3D pocket
  levels. They use other pass plans; a follow-up issue tracks them.

**Cost to change later.** Low. One condition in `pocket_over_with_relief`
selects the plan. The goldens of `pocket_zigzag_plate_with_island` and
the two `first-part` sample programs hold the output.
