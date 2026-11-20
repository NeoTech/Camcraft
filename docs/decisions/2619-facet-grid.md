## #2619 - CPU facet grid for the drop-cutter support scan

**Decided.**

- The support facets and their grid are one type, `SupSet`
  (`crates/motion/src/finishing3d/grid.rs`). `build_sups` returns it. Each
  `&[Sup]` parameter in `finishing3d` became `&SupSet`.
  - `SupSet` derefs to `[Sup]`, and `for s in sups` still reads every facet.
    The loops that are not point queries (shadow region, crease, rings,
    bridge, gap link) did not change.
  - `support_at`, `support_grad`, `ball_clear`, `column_clear` and
    `edge::fold_best` read `sups.near(x, y)`: the facets of the one cell that
    holds the query point.
- Each scan keeps its own padded-box reject. The grid only removes facets
  that the reject drops. The same f64 code runs on the same facets in the
  same order, so each answer has the same bits.
- The cell map is `floor((v - origin) / cell)`, clamped. The same map bins
  the boxes and places the query. Each step is monotone in IEEE-754, so a
  facet whose box holds the point is always in the point's cell. Each cell
  lists its facets in ascending index, one time each (CSR arrays).
- Cell size: half of the mean padded box, not finer than the extent / 1024.
  The size doubles until the cells are at most `4n + 64`, each axis at most
  1024, and the binned entries at most `32n + 65536`.
- Fallback to the full scan, never a guess: a non-finite box coordinate
  (the old reject never drops such a facet), a non-finite query point, an
  extent that is not finite, more than `u32::MAX` facets, or no size that
  fits the caps.
- The old scan stays as the reference: `SupSet::full_scan` (no grid). The
  parity tests run the same functions on the two sets.
- The parity and superset tests are lib unit tests
  (`finishing3d/grid_tests.rs`), not files in `crates/motion/tests/`. The
  issue named a motion test. The scan functions are `pub(super)`, and an
  integration test cannot reach them without a new public API.

**Why.** The issue asks for bit-equal goldens. The wrapper type keeps every
call site's code and changes only the parameter type. A cache keyed by the
slice address was refused: a freed and re-used address could give a stale
grid, which is a gouge.

**Dropped.**

- Parallel raster lines (the issue's optional item). A follow-up can add
  them; this slice keeps the output order and the thread model as they are.
- A grid shared between `build_sups` and `flat`'s floor set: `flat` builds
  its own (the boxes are the same; the build is O(n)).
- A query for a box (more than one cell). Every scan asks about one point.

**Readings.** Release build, parallel finishing, a height-field mesh, tool
diameter 6, tolerance 0.01, one run each (a temporary harness, not
committed):

| Facets | Before | After | Output hash (same before and after) |
|---|---|---|---|
| 1800 | 13.161 s | 13.908 s | `a629dd9f259d227d` |
| 7200 | 162.915 s | 161.515 s | `c719175f144437ae` |

The grid gives no measurable gain on this job. The point scans are not the
hot path here. The grid is kept: it is the CPU reference for the GPU filter
of #2526, its cost is O(n) at build, and the output is the same. A profile
must find the real hot path before more speed work (follow-up issue).

**Cost to change.** The grid is private to `finishing3d`. A different cell
law changes `grid.rs` only. The GPU filter of #2526 compares against
`SupSet::full_scan` and the grid.
