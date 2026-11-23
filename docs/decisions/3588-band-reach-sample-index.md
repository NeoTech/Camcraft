## #3588 - law (c) band reach reads its samples through an exact XY index

**What was decided.** `band_reach` (`crates/sim/src/verify/reach.rs`)
keeps ONE sample set for all cut planes, as before. The set is now held
in a `SampleGrid` (`crates/sim/src/verify/sample_grid.rs`): the samples
are kept once for each exact bit pattern, and put in square XY cells.
Each band disc reads only the cells that its box touches (the box is
widened by a relative 1e-9 against rounding). A sample or a query that
the grid cannot place (NaN, infinity, more than 1e12 cells out, a box
wider than 64 cells, a bad cell width) is read by every query, or the
query reads all samples. The sweep (c3) gets the deduplicated samples.
The old function stays as a test-only reference
(`crates/sim/src/verify/reach_reference.rs`, `#[cfg(test)]`).

**Why the findings do not change.**
- The law reads no plane of a sample. Its test for a sample `q` is: `q`
  in the band disc, `q` outside the flute kerf, `in_stock(q)`, and
  `h0 < surface(q, i + 1)`. Then `part_at(q)`. A sample sliced far below
  or above a move can convict that move. So the planes are NOT split:
  a move that only read "its own" planes could lose a finding.
- Each reading is pure in `q`. The verdict of a disc is a max over its
  samples (part, then stock, then clean), and the `'worst` break only
  stops when the max is reached. So the order of the samples and a
  repeated sample cannot change the verdict.
- The grid gives a superset of the samples that the old `> r2` skip
  kept. A sample outside the box has a distance above the radius, and
  the old loop skipped it too. Where a NaN makes the old skip fail, the
  grid reads the sample (the "always" list or a full scan).
- The sweep (c3) reads its samples through a max and a memo keyed by
  the exact bits (`sweep.rs`): a deduplicated set gives the same
  answer.
- Proof by test: `verify::reach_tests` runs the old and the new function
  on rings (clear and convicted), a helix, ramps of 400, 500 and 2000
  moves (clear and convicted), leaning rings that run the sweep (c3),
  Boundary mode, an empty path and non-finite moves. It asserts the same
  warnings in the same order, and the same slice count.

**Cost, measured by counters** (`ReachStats`: slices asked, and
band x point x sample tests):

| Path | Old tests | New tests |
|---|---|---|
| ramp 500 moves, shank 2 mm short of the wall | 2,004,000 | 0 |
| ramp 2000 moves, the same | 32,016,000 | 0 |
| ramp 500 moves, shank 1 mm into the wall | 1,394,117 | 20,192 |
| ramp 2000 moves, the same | 22,197,103 | 322,697 |

The slices stay one for each distinct end Z (500 and 2000).

**What stays.** A move whose disc comes near a wall still reads that
wall's samples of every plane. Each plane's section puts its own
vertices on a wall (a face is two triangles, and the diagonal crosses
each plane at a different place), so these are different points, and an
exact law must read them. That cost grows with moves x planes near the
walls only; the index takes away the large share that no disc can hold.

**Considered and dropped.**
- Test a move only against the planes in its Z span: not exact (see
  above). Dropped.
- Merge planes within a tolerance, or snap the island cache key: no
  sim code treats two Z values within a tolerance as one plane (the
  island cache keys on the exact bits, `rest.rs`). It would also change
  the samples. Dropped.
- A tangency fix in this PR: the tangency law (b) costs one slice for
  each distinct Z of a move (`section.rs`), and the slice itself is the
  cost. That needs its own design (a slice that is reused between the
  facet vertex heights is not the same Region). Filed as a follow-up.

**Cost to change later.** Low. The index is one private type of 140
lines that only `band_reach` uses. The reference and the equivalence
tests pin any later change of the scan.
