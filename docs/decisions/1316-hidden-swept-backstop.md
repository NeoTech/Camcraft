## #1316 - The mid-turn overhang backstop reads swept cells, not 4 x 5 poses

**What was decided.** Law (c3)'s overhang backstop (stock hidden under a
ledge, `Rest::hidden_*` in `crates/sim/src/verify/hidden.rs`) now asks over
the same reachable set the column law reads since #1313. It does not reuse
the column law's 0.001 mm cells: the hidden-stock law answers one plane per
ask and slices the model once per distinct plane, so it gets its own,
coarser cover (`crates/sim/src/verify/turn_hidden.rs`):

- the turn split so the sag is at most 0.05 mm (`BACKSTOP_SAG`, a tenth of
  the hidden law's own 0.5 mm XY spacing), capped at 20 intervals
  (`BACKSTOP_PLANES`); past the cap the cells only grow (more convictions,
  never fewer);
- the tip's chords grouped into at most 4 runs (`STATIONS`), and no more
  than `20 / intervals` of them;
- one ask per (interval, run), at the lowest bottom any column of its cells
  reaches, over the union of their footprints. A sample point counts only
  where some position of a cell stands the band over it outside the
  flutes' kerf at that tip position (the cells' own `bottom_over` test).
  The points asked are the hidden region's boundary samples in the box
  plus probes of the footprint (disc centres and rings at 5 tip positions
  per chord), built only once the plane has hidden stock near the box.

**Slice count.** At most 20 distinct planes per live band per move - the
old grid's 4 stations x 5 poses - so the slicing budget is unchanged. A
90 degree turn at 150 mm reach hits the cap and overstates by 0.12 mm.

**Why.** The 4 x 5 grid let a holder swing through hidden stock between two
sampled poses with no `SweepHit` (P1: a collision that passes verify). The
new cover is a superset of every pose and tip position of the move.

**Considered and dropped.**
- Reusing the column law's cells (up to 64 intervals x every chord): up to
  64 x chords planes per band per move, several times the old budget.
- A dedicated capsule/hull query in `hidden.rs`: the cells already carry the
  exact swept membership, so the new entry point (`hidden_in_cover`) takes
  that membership as a closure instead of re-deriving the geometry.
- One plane per band per move (the lowest of the whole move): on a long
  ramp it asks the deep end's plane at the shallow end, a false conviction.

**Kept from the old grid.** One plane per ask (stock the band meets above its
bottom plane is the column law's), and XY sampling of the hidden region by
its own law. Using the lowest bottom of an interval follows #1313's column
law, which does the same.

**Cost to change later.** The three constants in `turn_hidden.rs`; a finer
budget only costs slices.

**Composed with #1319's refresh (2026-11-12).** #1319 moved the grid
backstop's column to main's exact rim dip lowered by #1303's span floor
(`band_floors`), and put the same floors into the column law's cells. The
backstop's cells now take those floors too (`push_cells_with` threads
them; `turn_hidden` passes `band_floors`), so every ask stands no higher
than the span-floored exact-dip column the grid read at that pose: the
whole-turn reading convicts all the grid did, plus what lies between its
samples. #1319's pin `the_backstop_reads_the_exact_dip_lowered_by_the_span_floor`
was ported from `backstop_columns` to `backstop_asks` with the same
verdicts and the same exact reading (-56.43).
