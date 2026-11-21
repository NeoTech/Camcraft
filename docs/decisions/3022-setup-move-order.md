## #3022 - A move into another setup keeps the program order

**Decided.** `SetOperationSetup` (`crates/app/src/commands/setup.rs`) gives
the moved operation a position at the end of its target setup. It uses the
same `append_position` helper as `CreateOperation`. That helper also pins the
target's siblings at their current positions, and undo restores them. A move
that does not change the effective setup (the implicit `None` to the first
setup, or the reverse) keeps the position of the operation.

**Why.** The old command cleared the position (`order = None`). The program
order puts an operation without a position after the others, in name order.
Operations moved one by one into a new setup thus came out in name order: a
tap came before its tap drill. The issue asks that the program order stays
the user's. An append in move order keeps the relative order of a batch of
moves. It also is what a new operation does, so the rule is the same for
both commands.

**Dropped.**
- Keep the old position value. Positions count inside one setup, so the old
  value has no meaning in the target and would interleave the operations at
  random places.
- Insert the moved operation at its old index in the target. The target has
  different siblings, so that index is not defined.
- Append also when the effective setup does not change. The first move of
  the issue's case (implicit to the first named setup) would then send each
  operation to the end, one at a time. The order would be correct only after
  the last move.

**Cost to change.** Low. One command and its tests.
