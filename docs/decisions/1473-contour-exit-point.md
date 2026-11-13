# #1473 - a user-set exit point on the closed 2D contour (slice 1)

- **Chosen:** the exit point is a second optional field of the operation's
  `Approach` (`links.approach.exit_point`, `[x, y]` in mm, absent = leave
  at the seam), beside #1448's `start_point`, so `CreateOperation` /
  `EditOperationParams`, undo, the project file and the MCP
  `edit_operation` patch carry it with no new command; the digest grows
  only when it is set. **Why:** same reasoning as the start point; one
  door per parameter. **Dropped:** a `SetExitPoint` command.
- **Chosen:** the ring nearest the exit pick is cut all the way round from
  its start, then on along its own cutter-centre polyline to the pick's
  projection, and the lead-out leaves there. **Why:** the whole wall is
  still cut (never less material removed than without a pick), and the
  extra run re-cuts the ring itself, so it adds no new crossing and needs
  no new room proof. **Dropped:** ending the cut at the exit before a full
  loop (leaves an uncut stretch of wall); re-using the overlap's whole-
  segment retrace (it stops at the next vertex past the pick, which moves
  the exit silently).
- **Chosen:** refused by name, never moved: a lead-out asked for that does
  not fit at the exit, a lead-in the start had that the longer walk loses,
  a lead-out that crosses the lead-in, a pick farther than 1.5 D plus stock
  from every ring, and - not certified in this slice - tabs, compensation
  in the control, the keep-down level link (the next level descends from
  the seam), an overlap set beside the exit (both say where the cut
  leaves), open chains, tilted contours and every other kind.
  **Why:** a pick is a wish, never an override of a check. **Dropped:**
  dropping the overlap with a note when both are set (silently changes
  what one of the two parameters asked for).
- **Chosen:** the "no lead fits at the pick" refusals are pinned on a
  narrow bore (40 x 6.4 mm, 6 mm tool: a 0.4 mm wide centre ring) with a
  20 mm perpendicular lead. **Why:** the review of PR 1478 asked for the
  start's refusal to be pinned; a perpendicular lead into a 0.4 mm room
  fails even after four halvings, while a linear lead would fit along the
  long run. **Dropped:** a linear-lead fixture (it fits, so it pins
  nothing).
- **Chosen:** no UI row in this slice (the viewport pick and the X/Y rows
  stay item 2 of #1473). **Why:** the time box; the motion law and its
  refusals are the safety-relevant part.
