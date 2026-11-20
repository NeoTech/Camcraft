## #2612 - Nesting raster 1: conservative bit-packed rasteriser

**Decided (unattended, issue-worker-high):**

- **Place.** `camcraft_geometry::nesting::raster` (`crates/geometry/src/nesting/raster.rs`
  and `raster/scan.rs`). No new crate, no new crate edge, no new external
  dependency.
- **Integer decisions.** Region vertices are quantized with the `geo2d`
  Clipper-bridge rule (`geo2d::quantize`, 1 unit = 1 um). All cell tests
  are exact `i128` arithmetic with checked multiply and add. No float takes
  part in a cell decision.
- **One algorithm, two outputs.** Two integer passes: (1) "touched" - each
  cell whose closed square touches a closed boundary edge (row slabs, with
  the x extent of the edge in the slab rounded outward: floor of the
  minimum, ceiling of the maximum); (2) "centre inside" - even-odd parity of
  a +x ray at each cell centre, with the half-open edge rule of
  `geo2d::Loop::contains_xy`, in doubled coordinates so that the centre is
  an integer. Over = touched OR centre inside. Free = centre inside AND NOT
  touched. Why this is safe: a closed square that meets the region but no
  boundary edge is fully in the interior, so its centre is inside; a square
  that no edge touches and whose centre is inside is fully in the interior.
- **Rounding direction.** Always outward for "touched": a value on a cell
  line touches the two cells; the x extent in a slab is widened to whole
  units. This can mark one extra cell, never one fewer.
- **Cell packing.** Rows of `u32` words, `ceil(width / 32)` words for each
  row. Cell `i` is bit `i % 32` (least significant bit first) of word
  `i / 32`. Padding bits are always zero. `u32` matches a WGSL storage
  buffer word for the later GPU slice.
- **Grid origin.** `RasterGrid::covering` snaps the origin down to a multiple
  of the cell size (the global lattice), so that part and sheet rasters at
  the same cell size share one lattice and a placement moves in whole cells
  (research comment 2 on #2525, rule 3). `RasterGrid::new` takes any origin.
- **Cell-size default.** `DEFAULT_CELL_UNITS` = 1 mm (the coarse level of
  the coarse-to-fine search of #2525).
- **Cell limit.** `MAX_CELLS` = 2^24 (4096 x 4096, one fine tile of #2525)
  and `MAX_SIDE` = 2^16 on a side. A 3000 x 1500 mm sheet at 1 mm (4.5e6
  cells) fits. A larger grid is `RasterError::GridTooLarge`.
- **A part that does not fit is refused.** `rasterize_over` returns
  `RasterError::RegionExceedsGrid` when the region bounds are not in the
  closed grid extent: a clipped part raster is no longer an
  over-approximation. `rasterize_free` clips freely: a cell outside the
  region is never free, which is the safe side.
- **Empty region.** `RasterGrid::covering` refuses it (`EmptyRegion`); the
  two rasterize functions return an all-zero raster for it.

**Dropped:**

- Float scanline with an epsilon: not exact, and an epsilon can lose a
  touched cell.
- Rasterize the Minkowski-inflated part here: the spacing inflation belongs
  to the placer slice; this slice takes the region that it is given.
- `u64` words: `u32` is the native word of a WGSL storage buffer.

**Cost to change later:** the bit layout is internal to the raster and the
later slices; a change touches only `nesting::raster` and its tests.
