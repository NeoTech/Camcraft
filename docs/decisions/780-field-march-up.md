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
