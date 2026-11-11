## #779 - a carried field bakes the turn's signed permutation into its bricks (#637 E2a)

- **Decided:** `SdfField::turned(&LatticeTurn) -> (SdfField, Point3)` re-indexes
  every layer (band bricks, cuts, `cut_ups`, the new `cut_downs`) brick by
  brick: brick `b` goes to `!b` on a negated axis (`!b == -b - 1`), its slot
  mirrored `s -> 7 - s`, axes swapped on a quarter turn. Shell facets and the
  closed-form box are negated/swapped exactly, with no translation. The
  translation is returned as the residual view.
- **Why bitwise NOT:** `!i` is `-i - 1` with no overflow on any `i32`, so the
  re-index can't fail and `turned` has no error arm. Checked arithmetic on
  `8b + s` would need a refusal path for keys that can't occur.
- **Bit-identity is stated off the lattice planes and the shell crossings.**
  A point exactly on a cell boundary belongs to the other of its two cells
  after a negation, and a point exactly on a crossing is decided by the
  parity cluster's summation order, which a flip reverses. Everywhere else
  the bake reads the same floats the view reads. Pinned in
  `field_carry::a_baked_turn_reads_bit_for_bit_what_the_view_reads` (block
  and mesh-seeded fields, 8 turns) and
  `carry_engine_agreement::a_baked_turn_reads_what_the_view_reads_through_every_turn`
  (the column centres at every cell-centre height for 8 turns, plus the whole
  `column_body` read-back, bit for bit, for the 4 upright turns).
- **A flipped bake with cuts sets `bounded_cuts`.** `field_floor`'s exact
  floor under a cut assumes every cut came from above, and a flipped bake's
  cuts came from below. Until E2b's up-march reads `cut_downs`, a floor that
  a cut came near is refused by name (`CutFromBelow`) and never guessed. An
  upright bake keeps the law. Dropped: leaving the flag alone, which would
  let the E1 sliver law read a cut from below as a cut from above.
- **The dirty ledger marks every cut brick dirty** (when tracking is on)
  instead of clearing it. Everything moved, and the ledger promises a superset
  of what moved, never a subset. `full` is re-keyed, `region` is rebuilt
  (`region_of`), and `stats` carry over.
- **`field.rs` split:** the input gate (`validate_shell`, `edge_key`, and
  their tests) moved to `field_shell.rs` so that adding `cut_downs` brings
  `field.rs` down from 467 to ~340 lines. `field::validate_shell` is
  re-exported, so callers are unchanged.
- **Cost to change later:** the bake has no callers yet. E2b (`march_up`)
  and E2c (`CarriedField::simulate`) are its first consumers.
