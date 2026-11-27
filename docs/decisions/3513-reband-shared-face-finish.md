## #3513 - Explicit, measured band edges for shared_face_bodies_finish

**Decided.** `crates/app/tests/shared_face_bodies_finish.rs` no longer cuts
the sample's Z span into six equal bands. It measures the span in 48 equal
parts from the model's top, and the table `EDGES` gives 12 bands:
`0, 26, 29, 30, 32, 33, 35, 37, 39, 41, 43, 45, 48`. The upper band
(0..26) holds the whole fast upper half. Below the half, each band is 1 to
3 parts deep. There are now 12 heavy band tests in place of 6, and a new
default-suite test (`the_3d_finish_bands_are_narrow_where_the_levels_are_slow`)
keeps the table narrow in the slow half. The assert body of each band is
not changed.

**Why.** Release readings on one host (32 cores, each band alone, so the
rayon pool is not shared):

- Old sixths: 11.8, 6.8, 11.6, 558.9, 162.0, 217.8 s (sum 969 s).
- New 12 bands: 48.5, 59.2, 46.5, 53.5, 44.0, 56.7, 53.4, 35.4, 67.3,
  58.7, 51.4, 71.5 s (sum about 646 s). Slowest / fastest is about 2.0.

The generate time is not additive in the band depth. Old band 4 took 559 s
alone, but its eight 1/48 parts took 247 s together. The time depends on
where the level ladder of each band falls (each band's ladder starts at its
own top). So the edges come from measured bands, not from a model of the
cost. The 1/48 parts at the model's top (0..3) have no `shank_hit`
finding, so the top band must start at the top and must be deep. The
upper half as one band costs about 33-49 s, near the other bands.

**Dropped.**

- 24 or 48 equal bands: many 5-15 s bands next to 60 s bands. That breaks
  the 2x limit and adds tests that cost little.
- Fewer, deeper bands (about 8 bands of 80-150 s): a deep band can hit the
  same non-additive cost as old band 4. Narrow bands were measured; deep
  ones were not.
- An env var or a file that sets the bands at run time: the gate must run a
  fixed set of tests.
- A light twin: out of scope; the issue asks only for the re-band.

**Cost to change later.** The table and the `band!` list change together
(a `const` assert holds the count). New readings on the cut runner can move
an edge by one part. The host was loaded (load average 21-30) while the
readings were taken, so one band read 103 s once and 35 s on a re-run.
The readings are good to about 2x per band.
