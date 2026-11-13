# #1448 - a user-set start point on the closed 2D contour (first slice)

- **Chosen:** the start point is an optional field of the operation's
  `Approach` (`links.approach.start_point`, `[x, y]` in mm, absent = the
  automatic seam), so `CreateOperation` / `EditOperationParams`, undo,
  the project file and the MCP `edit_operation` patch all carry it with no
  new command. **Why:** it is an entry/lead parameter like the lead-in
  and axis-aligned start beside it; one field keeps every earlier file
  and every op without a pick byte-identical (the serde-default law), and
  the digest only grows when it is set. **Dropped:** a separate
  `SetStartPoint` command (a second door to the same parameter, with its
  own undo unit); a per-ring list of picks (the issue asks for "a start
  point"; one pick on the nearest ring covers it, more can come later).
- **Chosen:** the pick takes the ring nearest to it and starts that ring
  at the pick's projection onto the cutter-centre path (a vertex added
  mid-edge), on every level; other rings keep their automatic seam. A
  pick farther than 1.5 tool diameters plus stock to leave from every ring
  is refused by name. **Why:** a click on or beside the wall must work
  without the user knowing where the offset path lies; a click across
  the part must not silently land on some far wall. **Dropped:** snapping
  to the nearest vertex (moves the entry away from the pick).
- **Chosen:** when the lead law finds no lead-in that fits at the pick,
  the generation is refused by name instead of moving the entry to where
  one fits (as the automatic seam does). **Why:** the pick is a wish, it
  never overrides a check, and the entry never moves silently (Prime
  Directive). **Dropped:** a note and a moved entry.
- **Chosen:** refused by name in this slice, never ignored: tabs (their
  windows are measured from the automatic start), compensation in the
  control or wear (the seam belongs to the compensation law), open chains,
  tilted (3+2) contours and every other operation kind. **Why:** each
  needs its own proof; refusing keeps the slice small and safe.
- **Chosen:** no UI row in this slice; the parameter is set through MCP or
  the project file until the viewport pick and the property row land.
  **Why:** the time box; the motion law and the command path are the
  safety-relevant part. **Dropped:** an X/Y row on the Lead in / Lead out
  tab now (owed with the viewport pick, follow-up issue).
- **Chosen:** the follow-up issues are `P4`: the maintainer asked for the
  whole feature in his own words (#1448 quotes him), so its remaining
  slices are his requested work, not agent-found polish (`P7`).
