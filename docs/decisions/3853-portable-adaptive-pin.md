## #3853 - The "keeps its links" pins compare live, not with a recorded hash

**Decided.** The pins `adaptive_spiral_round_a_part_outside_equals_main_before_3818`
and `zigzag_rows_keep_their_links_round_a_part`
(`crates/motion/tests/pass_start_levels/pocket_outside.rs`) no longer
compare a recorded FNV hash of the path. Each one computes, on the same
machine and in the same run, the same plan with `StartSide::around_part`
off. That plan never links by the ring law of #3747
(`ring_links::for_side` returns `None` for it). The pin then asserts the
same move count, the same move kinds, the same move ends (exactly) and the
same notes. The `digest` helper is gone.

**Why the hash was not portable.** The hash read each coordinate to
1e-4 mm. The Adaptive spiral of the square fixture has 5687 moves from a
long chain of float operations (offsets, engagement search, arc fitting).
A different target (macOS aarch64: fused multiply-add, other `libm`
results for `sin`, `cos`, `atan2`) changes the last bits of some results.
Some coordinates then round to another fourth decimal, and the hash
changes completely (#3853: 131034927421005443 on macOS against
13996707157284497335 on Windows). A tolerance on a hash is not possible,
and a recorded reference list of moves would need a tolerance per
coordinate and a large fixture file.

**Still strict.** Both runs use the same build on the same machine, so
equal output is exact. With the predicate `ring_links::rings` removed, the
`around_part` plan links the Adaptive spiral by the ring law, and the pin
is red (checked on this branch: both pins fail on the move count).

**Dropped.**

- A recorded move list with a tolerance of 1e-6 mm: a large fixture file,
  and still not proved stable across targets for 5687 moves.
- A hash with coarser rounding: a coordinate near a rounding edge still
  flips, and a coarse hash misses small changes.

**Cost to change.** Two helpers in the test file (`without_ring_law`,
`assert_same_moves`). If `around_part` gets other effects on these
strategies later, the pin must compare with another reference.
