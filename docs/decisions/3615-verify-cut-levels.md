## #3615 - cut the verifier island-slice cost with an exact index (accepted)

### Decision

Keep every cut plane and every sample. Cut the cost of law (c)'s island
membership test with an exact point-location index on each cached slice.

- The index is a Y-slab edge index (`crates/sim/src/verify/slice_index.rs`).
  It stores the directed edges of a `Region` and, for each slab between two
  distinct edge-endpoint Y values, the edges whose half-open Y span holds the
  slab.
- A query reads only the edges that cross its own Y. It computes the crossing
  `x` with the same expression, in the same order, as `Region::contains`
  (`Loop::contains_xy` in `crates/geometry/src/geo2d.rs`), so the answer equals
  it bit for bit.
- `Rest::island_holds` asks the cached index instead of cloning and scanning
  the region. `Rest::islands_at` still returns the region, because law (c)
  builds its wall samples from it.
- The plane set is unchanged. Law (c) slices every cut plane, the same set
  the all-plane reference keeps. No sample is dropped.

The equality holds because a ray-parity boolean is the XOR of its per-edge
crossings, and XOR is associative: the source XORs crossings per loop and then
per loop, the index XORs the same crossings in slab order. Each edge's
crossing is independent and uses the source's exact arithmetic, with the
directed endpoints in the source's order (never swapped), so rounding matches.

### Why

In release, 98 percent of the time is `Rest::surface_before` in `band_reach`:
about 154 s to 174 s for one generation, at about 34 microseconds a call on
4.7 M calls. Each call runs `island_threshold` (about 16 `island_holds`), and
each `island_holds` was a full `Region::contains` over a slice of some
thousands of edges. The index reads only the few edges that cross the query's
Y (about two per loop), and it drops no sample, so it can only cut cost.

### The rejected design (dropped)

The first attempt sampled the model planes at the cut LEVELS plus the deepest
cut, in `crates/sim/src/verify/cut_planes.rs`. It was less conservative and is
removed here.

Finding, on the sloped ramp `ramp_at(500, 36.0)`:

- All-plane reference: `ShankHit { part: true }` at moves 250 to 292 and 293
  to 329.
- Levels plus deepest (the ramp has no level move, so its single deepest
  plane): `ShankHit { part: true }` at moves 250 to 273 and 293 to 329.
- Lost: moves 274 to 292, 19 part-true convictions.

The per-chord planes also place triangulation-tie wall vertices at
z-dependent positions, and those points catch the lost moves. One plane
cannot reproduce them. A verifier that loses a part-true collision is less
conservative, and the Prime Directive forbids it.

### Considered and dropped

- **Memoise `surface_before` per plane.** Kept as a fallback only. The exact
  index attacks the same hotspot without a second cache to invalidate.
- **Quantize the plane key and cap the sample count.** Dropped: a bound that
  is not conservative drops a collision, which is the failure above.
- **Adjust the ramp fixture to match.** Forbidden by the issue. The fixture
  and the reference are unchanged.

### The gate

- `verify::slice_index::tests::the_index_equals_region_contains_bit_for_bit`
  compares the index with `Region::contains` over a grid that lands on every
  vertex line and between them, for a U shape, a staircase, and a holed
  region. Any difference fails.
- `verify::reach_tests::a_many_plane_ramp_reads_only_the_samples_near_its_discs`
  runs law (c) and the all-plane reference (`reach_reference`) on the sloped
  ramp `ramp_at(500, 36.0)` and on a stepped fixture, and asserts the same
  findings in the same order, the same slice count, and no more sample tests.
- The stepped fixture has a different island section in each Z band: a wide
  low block (X and Y 40..60, Z 0..5) under a narrow tall block (X and Y
  45..55, Z 5..10), with the ramp at Y 49. Measured on a temporary variant
  that keeps only the wide low block (the deepest section's sample set), the
  all-plane reference convicts 93 `ShankHit { part: true }` moves, the
  wide-only set convicts 63, and it loses moves 250 to 279, 30 part-true
  convictions. The narrow band's Y 45 wall sample (4 mm off the path, so the
  flute cannot clear it) is what catches them. Keeping every plane keeps
  them.
- The reached heavy sim/verify lanes run on the final head.

### Readings (debug, this branch)

- `ramp_at(500, 34)`: new `slices = 500, sample_tests = 0`; reference
  `slices = 500, sample_tests = 2,004,000`.
- `ramp_at(500, 36)`: new `slices = 500, sample_tests = 20,192`; reference
  `slices = 500, sample_tests = 1,394,117`.
- Stepped ramp Y 49: new `slices = 500, sample_tests = 28,368`; reference
  `slices = 500, sample_tests = 1,390,052`.

### What changing this later costs

Remove the index by pointing `Rest::island_holds` back at
`Region::contains`: local to one method, and the plane set is untouched. Make
the plane set smaller again: that re-opens the 19 sloped-ramp and 30
stepped-ramp lost convictions, so it needs a new equivalence proof, not a
cost measurement.

Changelog: none (perf, no behaviour change)
