## #3325 - Slot start and exit points (first slice of the 2D ring and chain kinds)

**What was decided.**

1. **One kind for this slice.** The Slot takes a start pick and an exit
   pick (`PickCapability::Ends`). Pocket, 2D Offset, Circular, Trochoidal
   Slot, Trace, Engrave and 2D Chamfer keep `NotTaken(PICK_OWED)` and their
   refusal by name. Each is a later slice of #3325.
2. **A pick swaps the two ends, and does nothing else.**
   `machining2d::slot_picked_ends` returns `(a, b)` or `(b, a)`. The pass of
   `slot(b, a)` is the pass of `slot(a, b)` turned half a turn about the
   middle of the slot. A half turn keeps the stadium, the rows, the wall lap
   and the climb sense. So a pick adds no new move shape, and the entry law,
   the link law and the gouge checks judge the same pass as before.
3. **Start.** The pass starts at the slot end nearest the start pick. Under
   climb the earlier pass starts on `a`; under conventional on `b`.
4. **Exit.** The row count decides where a pass stops: an odd count stops at
   the end opposite the start, an even count at the same end (the wall lap of
   #2906 closes on the end of the last row). An exit pick alone turns the
   pass so that it stops at the nearest end. With a start pick, the start
   decides; an exit pick at the other end than the pass stops on is refused
   by name. The issue table says "the other end" for the exit: this holds for
   an odd row count, and the refusal says the end for an even count.
5. **Refusals, never a moved pick.**
   - Reach: a pick farther from the cutter path than `1.5 x tool diameter +
     stock to leave` (the contour's reach law, `contour_start.rs`). The
     cutter path is the stadium of the rows, so a pick beside the middle of a
     long slot is in reach and takes the nearest end.
   - A pick the same distance from both ends names no end.
   - A tilted (3+2) slot takes no pick yet, as the tilted contour: a pick is
     a model-frame point and the tilted slot's ends are not.
   - Not finite, or two picks of one role: the shared `PathPicks::refusal`.
6. **End extensions keep their ends.** The swap comes after
   `slot_extended_ends`, so `extend_start` stays on `a` and `extend_end` on
   `b`, and the #1504 room still judges the authored slot.
7. **UI.** The Slot's Lead in / Lead out tab draws the same Start point and
   Exit point groups as the 2D Contour, with the Slot's own help words. The
   viewport pick and the viewport marks take the Slot too.
8. **MCP.** No new tool: `edit_operation`'s `picks` field (#3339) already
   reaches the Slot. `list_operation_types` now answers `start: true, exit:
   true` for it. The sentence in the `edit_operation` schema that lists the
   kinds is not changed here: PR #3957 changes the next line, and one more
   edit there makes a merge conflict. It names the Slot after #3957 merges.

**Why.**
The slot generator already starts on either end (the climb flag chooses), so
the end swap is the smallest change that gives the user the choice, and it
cannot make a pass that the generator did not make before. Without a pick the
function returns its input, so every earlier slot is the same to the byte.

**Considered and dropped.**
- A new pass order for an exit that the row count makes impossible (an extra
  row, or a lap that stops at the other end): dropped. It changes the cut,
  and the issue says a pick is refused, never moved.
- The descent place moved along the slot to the projection of the pick:
  dropped. The issue's row says "the end `a` or `b` nearest the pick".
- Reading the reach from the two end points only: dropped. A pick on the
  slot's wall near the middle is a natural click, and it names an end.
- Generating both orders and reading the ends from the moves: dropped for the
  closed form (row parity), which the motion test checks against the moves.
- A row in the `camcraft-mcp` skill: not written in this PR, to keep process
  files out of a safety-tier change. The tool schema text carries the fact.

**Cost to change later.**
Low. The planner is one function with no state; a later slice (the
Trochoidal Slot, the open chains) can call it or replace it. A file that
holds a Slot pick from this version stays readable: the `picks` field is the
shared one of #3339.
