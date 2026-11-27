## #3334 - Plasma Profile takes a start pick and an exit pick

**What was decided**

- `KindInfo::pick()` answers `PickCapability::Ring` for `OpType::PlasmaProfile`.
  The picks are the operation-level `picks` of #3339 (`PlasmaOp.picks`). No new
  field, command or MCP tool was added.
- The generator has one new door, `camcraft_motion::plasma::plasma_profile_picked`.
  `plasma_profile` calls it with no pick, so a cut without a pick is the same
  path, move for move.
- A pick goes to the closed contour whose part profile is nearest to it. Each
  other contour keeps its automatic seam.
- Start pick: the cut of that contour starts at the projection of the pick on
  the kerf ring. The lead-in lands there. The pierce is where the lead-in
  starts, in the scrap.
- Exit pick: the torch cuts the full contour, then cuts on past its start to
  the projection of the exit pick. The lead-out leaves there and the torch
  goes off at its end. The cut direction does not change.
- Small-hole rule: the pierce stays at the centre and there is no lead-out.
  A start pick sets where the straight lead-in meets the hole. An exit pick
  sets where the torch goes off, and it must be at or past the overburn.
- Refused by name (the text names the role and the point of the pick), never
  moved:
  - a pick farther than 1.5 kerf widths from each closed profile
    (`PICK_REACH_KERFS`; the 2D contour uses 1.5 tool diameters);
  - a pick when the operation has open runs only;
  - a lead-in or a lead-out that finds no room beside the part at the pick
    (the pierce or the torch off would be on the part);
  - a small-hole lead-in from the centre that would cross the part;
  - a small-hole exit pick inside the overburn;
  - an exit pick with control compensation (G41/G42). The command door
    (`PlasmaOp::pick_refusal`) and the generator say the same words.
- The kerf law and the cover law of `plasma::emit::cut` still judge the
  complete path. The new guards only give the refusal the name of the pick.

**Why**

- The issue asks for the pierce at the lead-in start that lands on the
  projection of the start pick, and for the lead-out and the torch off at the
  projection of the exit pick. It asks that the laws refuse a pick and never
  move it.
- The run past the start is the only way to end the cut at a different place
  and still cut the full contour. The stretch between the start and the exit
  is cut two times. The note of the path says its length.
- Under G41/G42 the control offsets the programmed contour, and that contour
  ends at its start. A run past the start needs a longer programmed contour
  and a new proof of the compensation span. A mutation check showed that the
  generator would emit a path without that proof, so the pair is refused.

**Considered and dropped**

- End the cut at the exit pick without the full lap (an open cut). Dropped:
  the part would stay attached.
- Move a pick that has no lead room to the nearest place with room. Dropped:
  the issue says "refused by name, never moved".
- Change the cut direction so that the exit is reached sooner. Dropped: the
  direction sets the good side of the kerf.
- An exit pick with control compensation. Dropped for this PR (see "Why").
- The waterjet slice. `WaterjetProfile` is on `main` now, with its own
  planner and its tabs (`crates/motion/src/waterjet/`). Its capability stays
  `NotTaken(PICK_OWED)`: a pick on a waterjet operation is still refused by
  name. The follow-up issue for that slice is named in the PR body.
- A new top-level test binary for the plasma pins. Dropped: the pins are the
  submodule `crates/app/tests/start_exit_shared/plasma.rs` of the existing
  `start_exit_shared` suite (**New tests join a directory suite**). The MCP
  pin is in the existing `mcp_plasma.rs`.

**Cost to change later**

- The reach (`PICK_REACH_KERFS`) is one constant.
- To permit an exit pick with G41/G42: extend the programmed contour in
  `plan_closed`, remove the two refusals (generator and command door) and
  add the pins for the compensation span.
- The golden `crates/app/tests/golden/plasma_start_exit_picks.cl` pins the
  picked path. A change of the run past the start moves that one file.
