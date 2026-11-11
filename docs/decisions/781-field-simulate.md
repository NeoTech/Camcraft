## #781 - cuts through a carried field in the new frame (#637 E2c)

- **Decided:** `CarriedField::simulate(&mut self, ops)` (new module
  `field_carry_cut.rs`) runs `check_ops` on the caller's moves, bakes the
  view's signed permutation into the bricks (`SdfField::turned`, E2a) so
  that only the translation is left as the view, shifts every move by that
  translation (`from`, `to`, and the arc centre in its plane's word order,
  `ArcPlane::words`), and cuts through `simulate_field`. `simulate_field`
  is the field arm of `Stock::simulate`, moved out word for word so both
  callers share one tool construction (`spanning_flank`, batched per
  brick). The clip stays at read-back: a cut and the clip block are both
  intersections, so they commute. `column_body` keeps reading through
  `column_span` (E2b).
- **The bake is lazy.** A view that is already a pure translation is cut
  as it stands. `then` composes a new turn onto the translation, and the
  next `simulate` bakes it. Dropped: baking at `new`/`then`. That would
  cost an O(bricks) re-index on every carry, including the many carries
  that are only read back and never cut.
- **The flip is remembered on the carry, not read off the view.**
  `CarriedField` gains a private `flipped` flag, set by `new` and
  OR-ed by `then`. `column_body` uses it for `floor_judged` in place of
  `turn.flips()`. A baked flip leaves only a translation as the view, so
  `turn.flips()` would forget that the floor has faced the spindle. This
  also corrects two flips composed by `then` (an upright turn): the
  column engine's `Body::carry` keeps `floor_judged || flips` sticky, and
  the field now agrees with it. Dropped: a `baked_flip` flag set only by
  the bake. It would leave the `then(flip).then(flip)` case disagreeing
  with the column engine.
- **A refusal leaves the stock as it was.** `check_ops` runs before the
  bake. A shift that would overflow a move to a non-finite value is
  refused by the same gate inside `simulate_field`, before any
  subtraction. By then the bake has changed the representation but never
  the solid (E2a's view equivalence).
- **The translation helpers stay out of `carry.rs`.** `carry.rs` is 394
  lines. The bake checks the identity through `LatticeTurn::linear`
  (crate-visible) and builds the translation-only turn with `from_matrix`.
  That call cannot fail on a finite shift, and it is mapped to a named
  `StockError` instead of an `expect`.
- **Agreement (`carry_engine_agreement`):** #780's three by-hand pins
  (cut after the flip, the slivers between a cut from below and one from
  above, the second slot through the first) now run through the real
  `simulate`. Added:
  - an XY arc after the flip, at three depths: above the first slot's
    floor, a sliver, and through it;
  - a second carry after a cut (`then`), at three depth pairs: a quarter
    turn about Z, and a flip back that moves the `cut_downs` records up
    again.

  Both engines agree per column on `empty_at`, top and floor to 1e-6,
  and on `floor_judged`. `field_carry` pins `simulate` bit for bit
  against the by-hand bake and shift through all 8 turns, with an XY arc
  and an XZ arc.
- **A tie of measure zero is kept out of the fixtures, not fixed.** A
  third cut at exactly `5 - second` leaves a column whose top equals its
  floor. The column engine's strict `top < floor` empty law keeps that
  column as material, while the field reads no material. No steel stands
  there in either case, and no verdict reads a zero-thick column. The
  fixture's depths sit off that mirror (`3.7`, not `3.5`). Dropped:
  changing either engine's empty law. That would be a shipped-verdict
  change for no material.
- **Sweep through the verdict path (local, not committed, after the PR
  #817 round-3 review's approach).** The setup is an 8 x 6 mesh plate at
  cell 0.5:
  - floors {-6, -6.25, -6.1, -6.3, -5.9, -6.05, -6.2} and tops
    {0, 0.25, -0.2, 0.1, -0.05};
  - a first slot from the top along y 3, r 2, at 10 depths from 0.5
    under the floor to 3 above it;
  - flipped through `CarriedField::simulate` with a zero shift, then a
    second slot from the new top along x 4, r 2, at 10 depths, so the
    two cross;
  - flat and ball tools, read at 8 columns: under both slots, under one,
    beside them, and one off-lattice.

  The checks were `Stock::top` and `column_span` against the exact
  interval, and `Stock::deviate` against a part under the new floor
  (counting NaN columns where steel stands). Results: 56000 reads, 56000
  exact, 0 under-read tops, 0 over-read floors, 0 NaN or refusal where
  steel stands, and 0 `deviate` runs with a NaN column on material.
  The off-lattice column (4.4, -2.6) is judged at its cell's centre,
  which is how both engines sample a column. Judged at the point itself,
  a ball's curved floor differs by up to 0.025 there. That difference
  also appears upright on #817's head, and the round-3 review excluded
  it the same way ("on-lattice").
