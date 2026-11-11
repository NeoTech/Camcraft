## #780 - the field column read from below: `march_up`, the mirrored sliver law, the shell-crossing read (#637 E2b)

- **Decided:** `SdfField::column_span` reads a column from both sides.
  The top stays `march`'s descent; the floor is `march_up`'s ascent
  (`march_up.rs`, the mirror of `march`/`refine` by construction: same
  keyed probe, same budgets, descending from the first material probe to
  the LOWEST material centre and reading the floor from `cut_downs` -
  a flipped bake's `cut_ups` records moved whole - or the shell's lowest
  crossing in the bracket, the HIGHER binding). Where the field has no
  `cut_downs` at all (no cut ever came from below), a material centre
  still proves the shell floor by E1's monotone law and no ascent runs:
  the upright path is byte-identical to E1 in value and cost.
- **The shell-crossing read replaces the cell boundary in both marches.**
  `refine` read an uncut surface at the cell boundary where the stored
  sign flipped - exact only for a lattice-aligned plane; a flipped stock
  bottom (or any mesh plane) is not aligned in general. Both `refine`s
  now read the cached parity column's crossing inside the bracket
  (`march::shell_in`, the highest for a top, the lowest for a floor),
  exact to rounding for any plane, and where a cut entry and a crossing
  both bind, the one nearer the material centre wins (`march::face`) -
  the column engine's own `min(top, cut)`. On aligned fixtures the
  crossing IS the boundary, so `stock_backend_parity`, `stock_sdf*` and
  the field verdict suites read the same floats; a plane off the
  lattice now reads exactly where it was off by up to half a cell
  (`field_floor::an_unaligned_plane_reads_its_shell_crossing_on_both_sides`).
- **A level facet's crossing is exactly its height** (`solid_rest::plane_claim`).
  The heavy lane (`stock_engine_agreement`, release) caught the crossing
  read's one caveat: the barycentric sum `(1-u-v) a.z + u b.z + v c.z`
  rounds by an ulp on some rays even when the three z are equal (the
  stepped facing golden's uncut columns at y -6.25 read a top of
  2.0 + 1 ulp over a 2.0 stock top), and the rest ladder's strict
  `above(z)` mask at the stock's own plane saw it where the old boundary
  read happened to be exact. The root is fixed where it is born: a facet
  whose three z are equal answers that z; a tilted facet keeps the sum.
  Every consumer gains (E1's floor read -638.0000000000001 on the same
  columns is -638.0 now); nothing else in `solid_rest` changes. Pinned by
  `solid_rest::tests::a_level_facet_crosses_at_exactly_its_height` (three
  boxes, 441 rays each, bit for bit). Dropped: snapping a crossing within
  a few ulps of a cell boundary to the boundary - a heuristic over the
  symptom, wrong for a mesh stock whose column engine reads the crossing.
- **A sign flip with neither a crossing nor a record is `Unreadable`.**
  Every proven-inside cell records entry 0 with its claim, and the cell
  under it always holds a finite entry (the same sweep, 1-Lipschitz), so
  on an upright field this cannot happen; it does for a flipped bounded
  tool's bottom face (the truncated taper's top before the flip - no
  record was ever made going down). The old boundary guess is gone:
  refused by name, never a film (`a_flipped_bounded_tool_refuses_by_name`).
- **The sliver law is one scan, not two mirrors.** With no material
  centre the material lies between two adjacent air centres; the lower
  centre's `cut_ups` entry is the upper cut's bottom, the upper centre's
  `cut_downs` entry is the lower cut's top, the shell binds where no
  record does, and a centre proven inside a cut (entry 0) is that cut's
  whole half-line. Scanning every pair from the cell under the shell
  floor to the one over its ceiling reads E1's floor sliver, its mirror
  under a flipped cut, AND a sliver floating between a cut from below
  and one from above (E2c's fixture, pinned now through the bake and the
  residual translation by hand:
  `carry_engine_agreement::a_sliver_between_a_cut_from_below_and_one_from_above_carries_the_same`).
  An air centre strictly inside the shell proves a cut, so a record that
  must then exist and does not is refused, as E1 refused.
- **E2a's interim refusal is lifted.** `turned` no longer sets
  `bounded_cuts` on a flipped bake; the flag follows the tool alone. The
  interim law was pinned first (commit 1 of the branch) and the pin then
  replaced by the exact read, as the review of #784 asked.
- **Considered and dropped:**
  - Reading the floor by the monotone law mirrored (`floor = cut top` by
    the record at the cell below the ceiling, no ascent): covers the
    just-flipped field only; after E2c's cuts from the new top the
    lowest material centre is the only exact anchor. One ascent per
    column of a flipped field is the design's stated cost.
  - Keeping the cell-boundary fallback where no crossing exists: it is
    the inexact read the issue names, and on an upright field it is
    unreachable; keeping it would only serve the bounded-tool case as a
    silent guess.
  - A `CutFromBelow` name for the flipped bounded tool's top face: the
    march's own `Unreadable` already refuses by name, and the flag's
    `cut_near(floor)` refusal keeps its meaning (a floor its cut came
    near). Mapping the top-face refusal onto it would misname it.
  - A second bit-for-bit assert for flips in `baked_agrees` was expected
    to need a tolerance (the E2a decision states bit-identity off the
    crossings); it holds exactly through all 8 turns, so it stands.
- **Cost to change later:** `march_up` has one caller (`column_span`);
  `face`/`shell_in` are two ten-line functions shared by both marches;
  the sliver scan is one loop whose per-pair rule is stated in the
  module doc.
- **Review fix (PR #817): the marches walk centres, never points.**
  The review found `march_up`'s `phi == 0 => Floor(z)` exit (copied from
  `march`) reading a flipped floor 0.25 high: a probe's magnitude is the
  CELL's, so a cell whose centre lies on a stock plane reads zero at
  every point of the cell, and the exit answered whatever height the
  ascent had reached. `march` has the same exit upright (#824, filed by
  the reviewer). Tracing the sweep further showed two more holes of the
  same root in both marches: the sphere trace stepped by the cell's
  magnitude from points BETWEEN centres (the band value can over-read
  the truth by up to sqrt(3) cells, `field_gen`), so it could land past
  a web thinner than a cell and read the body below it; and its "last
  air probe", returned where the landing cell's centre was air, was no
  proof under a cut claim (a claim proves the centre and its half-line,
  not the cell's other half). Decided: one shared `march::walk` visits
  cell centres only, with the exact parity-and-claim sign at each
  (`sign_at`; the half-open `[floor, top)` law puts a centre on a floor
  plane in material and one on a top plane in air), skipping only runs
  the field proves air (a centre outside the shell: its magnitude's
  cells less two; a claimed centre: its claim's cells), and reads the
  surface at the first material centre from the one-cell bracket to the
  proven-air centre before it (`face`, unchanged). `march` and
  `march_up` are the two directions of that walk; `refine`,
  `refine_down`, `probe`, `STEP_MIN` and `MAX_MARCH_STEPS` are gone. The
  sliver scan's upper bound follows the same law (a centre ON the
  ceiling is already the one above it), which is what lets the
  reviewer's scenario read `[5.8, 6.25]` exactly through the record
  instead of refusing. Pinned by
  `field_floor::a_stock_plane_on_a_cell_centre_reads_exactly_on_both_sides`
  (the reviewer's flipped scenario, #824's upright twin, the uncut
  plate both ways, a top on a centre) and
  `field_floor::no_column_under_reports_across_plates_and_slots_on_and_off_the_lattice`
  (the reviewer's sweep: 4 floors x 3 tops x 9 slot depths, under and
  beside the slot, upright and flipped - 432 reads, every one exact,
  none under-read; both fail on the old marches). Every existing suite
  reads the same floats (the bracket read is unchanged; only the way to
  the bracket is), and the app's release heavy lane agrees with the
  column engine on every golden program as before.
- **Considered and dropped (review fix):**
  - Treating `phi == 0` as "refine from this cell" with the old
    `refine_down`: the cell's centre on a top plane is air, so the
    refine's promise breaks and it answers the last air probe - which
    under a cut claim can stand in material (in the reviewer's scenario
    the ascent's last "air" probe sits in the cut cell's upper half,
    above the tool's top). The hole is the point probe, not the exit.
  - Restoring E2a's `bounded_cuts` on a flipped bake (the review's
    "until then"): it would refuse every column under an earlier
    setup's cut - the read E2b exists to give - and leave `march`'s
    upright twin (#824) and the two trace holes in place.
  - A safe sphere-trace step (`magnitude - sqrt(3) cells` from a
    centre only): it is the centre walk with a floating-point step;
    whole cells make the skip's proof a count and the budget a bound.
  - Fixing `march_up` alone and leaving `march` for #824: the fix is
    the shared walk, and with `march` unfixed the reviewer's scenario
    reads a top of 6.44 (the same exit, over-reporting) and the floor
    `Unreadable`; `Refs #824` on the commit, the main session decides.
- **Review fix 2 (PR #817): a centre-less floor skin is a verdict, never
  NaN.** The second review found that `walk` answered `NoMaterial` for a
  column whose skin held no cell centre (a plate -6.05..0 at cell 0.5
  with a slot at -5.8 leaves [-6.05, -5.8] between the air centre -6.25
  and the claimed centre -5.75), and every verdict reader that marches
  alone - `Stock::top` (`column_top`), `deviate_field`, `measure_field`,
  the sheet, `Columns::top_at`/`above` (the rest and program masks) -
  read that as NaN: the rest under the slot vanished from the report and
  the rest mask would have treated the steel as air. Only `column_span`
  read such skins, through `field_floor::sliver`. On the old sphere
  trace the same columns over-read (-4.5 for -5.8) through its
  broken-promise path; the centre walk removed the accident and left the
  hole. Decided: the walk reads the centre-less column itself, before it
  ever answers "no material", through the ONE sliver law
  (`field_floor::sliver`, now `pub(crate)`, called from
  `march::centreless` with the shell's extent from the shared
  `shell_extent`): a column the shell never crosses is `NoMaterial` (air
  beside the stock), a sliver the records place answers its top (a
  descent) or floor (an ascent), one the records prove cut through is
  `NoMaterial`, and one the records cannot place - or a bounded tool's -
  is `Unreadable`, never `NoMaterial` where the shell says material
  stands. The mesh-only walk (`cuts == false`, `measure_field`'s own-top
  read) answers the shell's highest crossing (a descent) or lowest (an
  ascent), exact for any plane. `column_span`'s own `NoMaterial =>
  sliver` arm is now `Empty` (the walk has read the sliver). Pinned by
  `field_skin::a_floor_skin_under_a_slot_is_a_verdict_on_every_path`
  (the reviewer's scenario through `Stock::top`, `deviate` - 192 of 192
  columns rest, 0 NaN, dev 0.2 under the slot - `Columns::top_at` and
  `above`, flipped through `Stock::top` and `column_span`, the older
  -6.0 / -5.8 skin, and a cut-through column still air on every path)
  and `field_skin::no_column_the_shell_proves_reads_nan_or_air_through_stock_top`
  (6 floors x 3 tops x 12 slot depths with skins 0.01-0.3, upright and
  flipped, `Stock::top` under and beside the slot plus the flipped
  `column_span`, and `deviate` against the plate's bottom on every
  upright fixture: 1296 reads, 1296 exact, no NaN, no refusal; on
  dc4873e7 the same sweep had 186 NaN reads where the shell proves
  material and 81 `deviate` runs with NaN columns). Both pins fail on
  the unfixed source.
- **Considered and dropped (review fix 2):**
  - Falling back to `column_span`'s top from `march` (the review's
    example): `column_span` calls `march`, so the fallback belongs in the
    walk, where both marches and every verdict reader reach it.
  - Over-reporting the shell's ceiling (or the record's height) where
    the sliver cannot be placed: the module law is that a surface the
    field cannot place is refused by name, and a ceiling over-read on the
    verdict path would be a false rest in `deviate`; `Unreadable` blocks
    the same way NaN did but is a named refusal in the enum, and the
    sweep pins that no column the shell proves reaches it.
  - A new `Read` arm for the bounded tool's `CutFromBelow`: `walk` maps
    it to `Unreadable` (the sliver's own refusal); `column_span` still
    refuses by name before marching wherever `cut_near(floor)` holds, so
    the name is kept on every fixture that reaches it.
