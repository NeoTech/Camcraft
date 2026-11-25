## #3750 - Ring planning step 1: the shared modules, taken from the 2D Offset path

**Decided.** The ring code of the 2D Offset path moves into the three
shared modules of #3743 (`docs/decisions/3743-shared-ring-planner.md`). The
output does not change: the CL moves of the pinned cases are identical to
the head before the move (origin/main `138a592a1`).

- **`ring_set` is a new module, `level_rings` keeps the level curves.**
  `ring_set.rs` holds the planner contract: `RingRef`, `RingSet` (the
  input), `RingSet::plan` (the old `side_plan`: the start-side plan, the
  completeness guard and the named fallback), the op entry points
  `ring_plan` and `level_ring_plan`, and the guard `unswept`. `level_rings.rs`
  keeps only the level curves of one start side (`level_pieces`, the
  material parts, the core-first rings, the side split, the fills). The
  seam is the 400-line law: one file with both is about 400 lines, and the
  two are two concerns (the contract and the geometry).
  Dropped: one file `ring_set.rs` with all of it (over 400 lines).
  Dropped: a rename of `level_rings.rs` (each later step reads its
  history).
- **`RingRef` holds only the reference edge.** `RingRef::Part` is the old
  `StartSide::around_part == true`, `RingRef::Boundary` the old `false`,
  and `level_pieces` now reads the reference from the `RingSet`, not from
  the side. `StartSide` keeps its `around_part` field: the links, the
  entry, the Auto rule and the order still read it, and it is public API
  that the app and the tests set. Dropped: a `RingRef` field in
  `StartSide` (it changes the public surface while the path kernel record,
  #4024, can still rename it).
- **No chord tolerance and no circle in the input yet.** No ring op of
  step 1 reads them: the rings use the geometry layer's fixed arc
  tolerance, and no 2D Offset ring is an exact circle. The circle input
  (exact G2/G3 rings) comes with the first op that needs it, Circular
  (#3755). An unused input would be code with no test.
- **`RingRef::Boundary` with Inside or Outside keeps the #3316 rings.** The
  pocket's Inside measures from the holes and islands, its Outside from the
  outer edge. The research text says "all walls of the area" for
  `Boundary`; that is true of Auto (`waterline_passes`, pinned within 1 um),
  but the start-side rings of a pocket differ today. To measure them from
  all walls changes the 2D Pocket output; that is #3752's work, with its
  own pins.
- **`ring_links` is the existing module of #3747, extended in place.** It
  is already the one in-plane link rule of the rings (`RingLinks`,
  `for_side`, `link`, the named lift reasons through the ladder's
  `LinkTally`). No second module with the same name, and no rename: its
  callers (`ladder_links`, `level_order`, `levels`, `pocket_emit`) stay as
  they are. The module doc now names it as the #3743 link rule.
- **`ring_entry` is a new module for the ring-specific entry decision.**
  `RingEntry::plan` holds the steps that were written twice, in
  `pocket.rs` and in `pocket_avoid.rs`: the air room outside the stock box
  (#3746, less the avoid keep-out of #3810), the refusal of an Outside Arc
  with no air room (#3884), and the lead-in advice note (#3748). The other
  steps of the entry order (a pre-drilled hole, a cleared column, a helix
  or a ramp, the named stand-in) stay in the shared ladder (`levels`,
  `pocket_emit`, `entry_warn`), because the passes that are not rings use
  them too; the module doc names the order and where each step is.
  The function `contour_link::ring_entry` (the 2D Contour's ring-to-ring
  arm) keeps its name: it is a different thing in a different module, and
  #3757 moves the Contour onto these modules.
- **`ring_side` stays as it is.** It is the side logic of the one ring set
  (the mirror, the Auto rule, the shared ring ends), and `ring_set` names
  it.
- **The `Links::RINGS` seed is not built here.** It changes the output of
  old project files (see the #3743 record).

**Pins.**
- `offset2d_rings_unchanged_by_the_shared_planner` (motion,
  `pass_start_levels`): the 2D Offset round the dog-bone part of #3741, for
  Auto (with and without a stock footprint), Inside and Outside, for
  Waterline and Offset rings, with a Helix and an Arc entry: eight goldens
  `crates/motion/tests/golden/offset2d_dogbone_*.cl` with the notes.
- `pocket_rings_unchanged_by_the_shared_planner` and
  `level_rings_unchanged_by_the_shared_planner` (motion): the 2D Pocket
  (`RingRef::Boundary`) and the 3-D level plan on the split band of #3316.
- `fig8_offset_rings_unchanged_by_the_shared_planner` (app, heavy lane):
  the 2D Offset on `fig8.step` for Auto, Inside and Outside, and the Arc
  entry for the two sides, with the notes and the verifier warnings.
- `ring_set` unit tests: a disc gives circle rings at `rc - k * step`; a
  square with an island gives the `waterline_passes` rings for
  `RingRef::Boundary` (Auto) within 1 um.

**Cost to change later.** The contract is `pub(super)`: a rename by the
path kernel record (#4024) touches only the `machining2d` module.
