## #3327 - 2D Adaptive start and exit points (the HSM plan's seed and its last pass)

**What was decided.**

1. **Capability.** The 2D Adaptive takes a start pick and an exit pick
   (`PickCapability::Region`: a point in the area). The other kinds keep
   their capability. The picks are the shared operation-level `picks`
   field of #3339, so the command, the project file, the stamp and the MCP
   `edit_operation` patch need no change. `list_operation_types` answers
   `start: true, exit: true` from the capability.
2. **Start = the plan's seed.** The start point is the seed of the HSM
   plan (`AdaptiveSpec::seed`). The helix or the plunge goes down there,
   through the same entry law (`choose_entry` hands the ladder the same
   triangle room that it hands it for the automatic seed), and the plan
   grows the cleared area from that bore. Each level enters there. The
   engagement law is the planner's own: each cut is measured on the
   in-process stock before it is kept, and a plan that crosses itself or
   leaves stock is not posted.
3. **Exit = the last pass runs back along itself.** The last pass of each
   level ends where the plan ends, on its closing ride along the walls.
   With an exit point, the tool then runs back along the same posted
   moves in reverse (a line is the same line, an arc is the same arc with
   the other sense), over one round of the area's outer wall at most, to
   the point of that tail nearest the exit point. The climb out of the
   level moves there, and the rapid after it starts there. The run back
   is made on the posted toolpath (`adaptive_picks::exit_on_path`), not
   on the planner's walks: the arc fit of a reversed walk differs from the
   fit of the forward walk inside its tolerance, and the two fits cross
   (the verifier's feed-feed law read two such crossings on the first
   try). Reversed posted moves cut no new material and cross nothing.
   The pieces of the area are ordered so that the piece nearest the exit
   point is cut last.
4. **Refused by name, never moved.**
   - A start point outside the area that the tool centre can reach (the
     walls plus the tool radius and the stock to leave, each island, each
     "Do not touch" keep-out).
   - A start point where the helix has no room: the helix needs
     `helix radius / 0.6` of room (the plan's own `HELIX_SHARE`, so the
     first rings can part from the orbit). The automatic seed narrows the
     helix and says so; a picked seed does not, because the user chose the
     place and the radius.
   - A start or exit point inside a "Do not touch" keep-out, with the
     items' names.
   - An exit point farther than `1.5 x tool diameter + stock to leave`
     (the contour's reach law, #1448) from the closing ride.
   - An exit point at the piece where the cut starts while other pieces
     are cut after it.
   - An exit point on a level that does not end with a straight climb
     and a rapid (a level link that stays down): the move after the
     climb is not moved without its own proof.
   - A plan from the start point that keeps a cut over the optimal load
     (the planner's "least overload was cut" and "worst planned load ...
     exceeds" notes). The automatic seed may keep such a cut and say so;
     a picked seed holds the engagement limit from the first pass on, as
     the issue asks.
   - A plan from the start point that the planner does not post (it
     crosses itself, leaves too much stock, or runs past its budget): the
     Offset rings and the trochoid that would cut the area instead choose
     their own entry and end, so the pick is refused with the planner's
     reason.
   - At the door and at generation: a tilted (3+2) op, the Trochoidal
     method, Pass starts at Inside or Outside, a rest pass, and a start
     point with an entry other than Helix or Plunge.
5. **No pick, same path.** Without a pick no new code runs: the pieces
   sort as before and the plan is the same to the byte. The runtime fields
   `Approach::start_point` / `exit_point` are `None` for every stored op
   and for every other kind.
6. **WCS yaw.** The picks are model-frame points; under a Z-rotated work
   offset they turn into the generation frame with the corner-relief
   points (`wcs_yaw::rotate_params`), as the Slot's picks do.
7. **UI.** The 2D Adaptive's Lead in / Lead out tab draws the Start point
   and Exit point groups with its own help words. The viewport pick and
   the marks take the 2D Adaptive.

**Why.**
The planner already starts from a seed and grows the cleared area under
its engagement law; a user seed is the smallest change that gives the user
the place, and the planner still measures each cut. A retrace is the only
exit move that is safe without a new proof: it touches only cut material
and makes no crossing for the verifier's law (a). The issue says a pick is
refused, never moved; the refusals carry the planner's reason.

**Considered and dropped.**
- A straight feed from the plan's end to the exit point: dropped. It
  crosses earlier revolutions (the verifier's feed-feed crossing law) and
  can meet the corner scallops that the plan leaves.
- Running on along a closed final lap: dropped. The plan's last pass ends
  on its wall ride without closing the lap, so no closed tail exists to
  run on.
- The run back on the planner's walk (before the arc fit): dropped, see
  item 3.
- Moving the helix or narrowing it at a picked start: dropped. A pick is
  never moved; the refusal names the room and the radius.
- The Trochoidal method: refused by name in this slice. Its seed is on the
  medial line and its loops have their own order (#3932 is its separate
  entry defect; this PR does not change that path).
- A rest pass, Pass starts at Inside / Outside: refused by name. They
  choose the start themselves.
- A row in the `camcraft-mcp` skill: not written, to keep process files
  out of a safety-tier change. The capability and the tool schema carry
  the fact.

**Cost to change later.**
Low to medium. The start is one branch in `choose_entry`; the exit is one
function over the posted path (`adaptive_picks::exit_on_path`). A later slice
can take the Trochoidal method or a rest pass by changing the door and
its own planner. A file that holds an Adaptive pick from this version
stays readable: the `picks` field is the shared one of #3339.
