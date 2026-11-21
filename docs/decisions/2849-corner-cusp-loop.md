## #2849 - The corner check reads a cusp loop that crosses itself by the even-odd rule

**What was decided.** `connected_pieces` in
`crates/motion/src/machining2d/unreached.rs` builds the fill of each outer
loop with `Region::from_slices`, not `Region::new`. A loop that crosses
itself is read as its even-odd lobes. The corner check then runs and applies
its usual noise filters (`PIECE_AREA_MIN`, `SLIVER_HALF_WIDTH`,
`CORNER_HALF_WIDTH`).

**Why.** The loops that `connected_pieces` reads are the output of a
boolean (`piece.difference(&swept)`), not sketch input. A bore drawn as
chords less its own swept area leaves a cusp a few microns wide at each
chord vertex. On the micron lattice, the boolean can return such a cusp as
one loop that crosses itself. `Region::new` refuses that loop, so the whole
corner check stopped with the note "the check for corners of the stock
tighter than the tool did not run (invalid geometry: loop crosses itself)".
The refusal in `Region::new` is for ambiguous user geometry. It has no
purpose for a noise cusp that the next filter drops. A loop that does not
cross itself gives the same region from the two constructors.

The defect does not need the solidcore importer. A synthetic 25 mm bore
shows it on `main`: in a sweep of 24 to 400 chords, 9 tool sizes and 4
phases, 4726 of 13572 cases stopped with the error. With the fix, no case
stops.

**Scope: partial.** The issue names three toolpath tests. The owner's
diagnosis on the issue says that each has its own cause (the pass start
choice, `axis_reseat`, the ring order) and that this fix does not change
them. Their `known-fail #2849` markers are on the unpushed PR B of #917.
This PR therefore ends with `Refs #2849`, not `Fixes #2849`.

**Merge with main.** `main` got the same code fix in `bc5daa32d`
(`outer_fills`, pinned by a hand-made bow-tie loop). The refresh takes
main's code. This PR now adds only the chord-bore pin: it reads the real
path (a bore less its swept area) through `unreached_corners` and
`unreached_corner_note`, where main's pin reads `outer_fills` alone.

**Considered and dropped.**
- Relax `Region::new` itself. Dropped: the refusal protects sketch input
  (Prime Directive), and many callers rely on it.
- Skip a loop that fails `Region::new`. Dropped: that hides a loop without
  a look at its size. The even-odd reading keeps the area and lets the
  existing filters decide.
- Raise `PIECE_AREA_MIN` above the cusp area. Dropped: the filter runs
  after the constructor, and a threshold tuned to one mesh breaks with the
  next mesh.
- Change the planners for tests 1 to 3 (start choice, reseat tolerance,
  ring order). Dropped here: they cannot fail on `main`, and other pins and
  goldens read that behaviour.

- Close this PR as superseded. Dropped for now: the chord-bore pin covers
  the full corner check, not only the fill constructor. The re-review
  decides if that coverage is worth the test.

**Cost to change later.** One line. The pin
`a_chord_cusp_that_crosses_itself_does_not_stop_the_corner_check` fails if
the refusal comes back.
