## #1313 - The mid-turn sweep (law c3) reads swept cells, not five poses

**What was decided.** Law (c3) no longer samples the turn. For every band
of the tool stack it builds *cells* (`crates/sim/src/verify/turn_cells.rs`):
one per tip chord of the move and per interval of the turn. A cell holds
law (c)'s column for every pose inside its interval and every tip position
on its chord:

- in the turn: the convex hull of the column's two end discs, grown by the
  cell's *sag*, with the bottom at the lower end bottom less the sag;
- along the path: the hull is swept along the tip chord exactly (a
  segment-against-hull clip), and the bottom rides the chord's own Z;
- the flutes' kerf excuses a sample column only for the tip positions that
  actually stand over it.

The sag is proven, not measured: the letters lerp, each letter turns the
axis at most one radian per radian and bends it no faster, so over a cell
the axis strays from the chord of its end axes by at most
`(letters' sweep in the cell)^2 / 8`; times the band's reach above the tip
that bounds the column's centre, radius and bottom. The cell count per
band is chosen so the sag is at most 0.001 mm (`SAG`), capped at 64 cells
(`MAX_CELLS`); past the cap the cells stay conservative and the sag grows
(a 90 degree turn at 150 mm reach: 0.011 mm).

The reachable set is unchanged in meaning - the whole rectangle of (path
position, turn fraction), "either family may arrive first" - but it is now
read whole, edges included, where the grid read 4 x 5 interior points.

**Why.** The grid let a band cross stock between two sampled poses
(#1313, P1: a collision that passes verify). A bound has to hold between
samples by construction; a denser grid only moves the gap.

**Readings.**

- Probe (`no_band_position_of_a_random_turn_lies_outside_the_cells`, 240
  random turns up to +-85 degrees in A and B, a third also in C, 5,076,000
  band positions: the true cylinder's surface and law (c)'s own column at
  9 path stations x 25 turn fractions): zero positions outside the cells,
  in XY and in Z. The same positions against the old grid: 272,961 outside
  it, on 240 of 240 turns (the old grid also left the move's two ends to
  law (c), which reads only the to-point at the to-pose).
- Pin (`a_holder_crossing_a_stock_post_between_two_sampled_poses_is_convicted`):
  a holder 100 mm up the stack swings through a stock post between the
  20 and 40 degree samples of an 80 degree turn. Clean on main, `SweepHit`
  here.
- False convictions the wider reading brings: none in the test suites - the
  48 gate-mapped suites of sim, motion and app (every existing mid-turn
  test among them) keep their verdicts with no assertion edited,
  including the 0.004 mm clearance of
  `the_mid_turn_sweep_grows_the_band_the_same_way`. By construction the
  new convictions against the old reading are: a band over a column
  between two sampled poses or stations (the defect), the move's two ends
  under the other family's poses (reachable by the law's own model, never
  read before), the 0.001 mm sag, and a band wider than the cutter read
  one band radius lower at an upright START pose (the form the old grid
  applied from its second sample on). Not measured on real posed
  programs: none of the debug suites verifies one end to end with a
  near-miss; the release lanes do, and were not run here.

**Considered and dropped.**

- *A padded column per interval* (midpoint pose, radius and bottom padded
  by the swing, as the posed fixture sweep does since PR #1307). First
  order in the interval: at 0.25 mm of pad it convicts
  `the_mid_turn_sweep_grows_the_band_the_same_way` (a holder 0.004 mm
  clear of a wall, turning away) at full fidelity, and the issue asks that
  the existing tests stay green without loosening. The hull is second
  order, so 0.001 mm costs a handful of cells.
- *More samples, count from the turn angle.* Still a grid: the bound on
  what lies between samples is first order, so honest coverage needs the
  same padding as above.
- *Sharing `midpose::intervals` with the posed sweep.* That walk ties the
  pose to the path position (one clock); (c3)'s reachable set lets either
  family arrive first, so the two are different sets. Shared instead: the
  letter-rate bound and `midpose::pose`.
- *An upright end keeping law (c)'s upright bottom.* A cell that is not
  upright throughout reads the leaning form at both ends (bottom one band
  radius lower), which is what the old grid read one sample further on;
  mixing the forms inside a cell would not bound the poses between.

**Not changed, and owed.** The overhang backstop (`hidden_in_annulus`,
stock hidden under a ledge) keeps the old 4 x 5 grid for this law: it is a
sampled law by its own statement (XY rings, one plane per ask, a model
slice per distinct plane), and covering a cell with its discs would
multiply those slices. Filed as #1316.

**Cost to change later.** `SAG` and `MAX_CELLS` are two constants;
loosening `SAG` only trades cells for overstatement. Replacing the hull by
another enclosure touches `turn_cells.rs` alone - `sweep.rs` asks one
question of a cell (`bottom_over`).

**Merged with #1303's span floor and exact rim dip (PR #1306, main's
3cee560c / de1d8641).** Main's sampled sweep had gained two changes to the
pose loop this PR replaced: each pose's bottom dips by the rim's exact
R·sin θ (not the whole R), and each pose reads the lower of that and the
`tilt_span::span_floor` of the two spans it ends (a lerp whose A or B
letters leave (-90, 90) can lean past both samples). Chosen: the two laws
compose by `min`, so the merged (c3) convicts everything either side
convicts on its own.
- *The cells* read the rim's exact dip (below) and additionally the
  span floor of every sampled pose fraction (0, 0.25, ..., 1) the cell
  holds (`sweep::band_floors`, applied in `push_cells`). Every such
  fraction lies in at least one cell (the cells tile [0, 1]); that cell's
  hull holds the sampled pose's column (same offset and radius as
  `tilt_band::lean`), its tip chord holds the sampled stations, and its
  bottom is no higher than the sampled reading `min(exact dip, span
  floor)`. Inside the cap the floor is `+inf` and nothing moves.
- *The backstop columns* read main's column exactly: `tilt_band::lean`
  with its exact dip, lowered by the same span floor.
- *Dropped:* reading the span floor of each cell's own letter interval
  (tighter, so higher than the sampled law's floor over a quarter of the
  turn - it would let a sampled conviction go); and lowering every cell
  that overlaps a span (convicts more than either side, with no physical
  need - the cell's sag bound already holds every in-between pose).

**The exact rim dip per cell - #1303's law wins over the whole radius.**
The merge first kept this PR's cell bottom (the whole band radius under
the lower end station), and main's
`tilt_tolerance::a_tilt_just_above_the_tolerance_reads_like_upright_and_a_real_dip_convicts`
failed: a holder 5 mm clear, tilted 1.01e-9 rad, convicted - the cliff at
the upright tolerance #1303 removed. The two readings cannot both hold, so
"convict everything either side convicts" does not apply to this one
class. Chosen (main session, 2026-11-12, for the maintainer to overrule):
each cell's bottom is `min(lo0, lo1) - R * min(1, max(sin θ0, sin θ1) +
kappa) - sag`, `lo` the lower end station over the tip at each end pose.
Sound: over the cell the axis strays from its chord by `kappa` at most, so
its horizontal part - the rim's sine - stays under the larger end's sine
plus `kappa`, and the stations stay within the existing sag. Why: #1303
already calls the whole-radius convictions false (a band a hair off upright
read R lower than upright), and the bound above is still never under the
true band. Dropped: keeping the whole radius and changing main's
tilt_tolerance test (it would re-open the cliff #1303 closed). The one test
number this PR changes follows from it:
`the_kerf_excuses_only_where_the_tip_stands` now expects the bottom under
`5 - 4.8` less a dip of at most 6·sin 1°, not a whole 6 mm lower.

Pinned by `turn_cells::tests::a_cell_holding_a_sampled_pose_reads_its_span_floor`
(A 100 -> 260: a column under the far rim of the A 100 column that only
the span floor reaches). Mutation-checked: without the floor in
`push_cells` that pin fails; with the cells replaced by the five sampled
poses `midturn_swept_band::a_holder_crossing_a_stock_post_between_two_sampled_poses_is_convicted`
fails.
