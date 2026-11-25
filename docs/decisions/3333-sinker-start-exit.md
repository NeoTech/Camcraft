## #3333 - Start and exit picks of the Sinker Burn and the Sinker Side Burn

**What was decided.**

1. **Sinker Burn: a pick is an angle round the burn position.** The start pick is the angle at which the orbit starts. The exit pick is the angle at which the orbit ends. The capability is `PickCapability::Ring` (the orbit is the ring).
   - Circular: the circle starts at the start angle and makes one full turn counter-clockwise. With an exit it continues counter-clockwise to the exit angle.
   - Planetary: the spiral turns from the start angle, so it reaches the radius at the start angle. The closing circle starts there and ends as the circular orbit does.
   - Vector: the vector toward the start pick is burned first. The vector toward the exit pick is burned last. With both, the exit is the vector before the start (counter-clockwise order) or the vector after it (clockwise order).
2. **The orbit returns to the burn position before the retract.** The exit pick sets the last position on the radius. The electrode then feeds back to the burn position and retracts up the burn axis, as it did before.
3. **A circle keeps its own vertices.** The start and the exit are added vertices. The four axis directions stay vertices, so the walls of a rectangular cavity move out by exactly the radius. Every vertex stays on or inside the radius, so the undersize law and the cavity limits hold at every angle.
4. **A vector orbit keeps its directions.** A pick changes the order only. A pick between two vectors is refused by name.
5. **Sinker Side Burn: a pick is an end of the side move.** The move has two ends: the burn position and the wall end (the burn position moved by the amount toward the wall). A pick on the wall side of the middle of the move chooses the wall end. The start pick chooses where the electrode plunges. The exit pick chooses where it retracts. The capability is `PickCapability::Ends`.
6. **Refusals, by name, at the command door and at generation:** a pick on a Sinker Burn without an orbit; a pick on the burn position (no angle); a pick on no vector; an exit vector that the order cannot end on; a side burn pick at the middle of the move.
7. **No pick keeps every path.** With no pick the generators call the earlier functions, so the moves and the notes are identical. A picked path adds one note that names the picks.
8. **The page types the points.** Each kind has a Start point group and an Exit point group on the Geometry tab, with a switch and X / Y rows. A Sinker Burn without an orbit shows "Not taken" with the reason. A viewport pick button is not added.

**Why.**

- The issue asks for the start angle of the orbit and for the place where the orbit ends. An angle is what a point round the burn position gives for every orbit radius, also for a radius of 0.1 mm that no click can hit.
- The undersize law (`sinker_undersize::judge`) counts the orbit radius and requires a vector orbit to reach the four walls. A rotated circle grid or a rotated vector fan would leave a wall short of size on a finish pass. Added vertices and a changed order cannot.
- The layered burn (`sinker_wear::burn_compensated`) plunges each layer on the burn axis. The orbit must be at the burn position between two layers. One rule for the last layer and the others keeps the retract law unchanged.
- The side burn's swept volume is the same for each choice of ends: the electrode is at the same two positions at the same tip. The allowance law, the wear law and the layer law are read from the unchanged `side_burn_worn` first.

**Considered and dropped.**

- Retract at the exit angle, off the burn axis. Dropped: it changes the retract law of the burn and does not work between layers. The issue says "where the orbit ends before the retract", and the orbit ends at the exit angle.
- Rotate the vector directions to the start angle. Dropped: a finish pass needs the axis directions to bring all four walls to size.
- Rotate the circle's chord grid to the start angle. Dropped: the axis directions would no longer be vertices, and the walls would be up to one chord sagitta short.
- Snap a vector pick to the nearest vector. Dropped: the series law is "refused by name, never moved".
- Read the side burn's start pick as the wall direction. Dropped: the operation has the `direction` parameter for that, and two inputs for one value can disagree.
- A new `PickCapability` variant for the orbit. Dropped: `Ring` and `Ends` describe the two cases, and a new variant changes the shared hook of the other family slices.
- A field on `SinkerParams` or `Orbit` for the angles. Dropped: it changes about 30 struct literals in other tests. The picked generators take the ends as an argument (`burn_compensated_between`, `side_burn_between`).

**Cost to change later.**

Low. The pick readers and the picked generators are in one module (`motion::sinker_picks`). The stored data is the shared `PathPicks`, so a different reading of a pick needs no file format change. A change of the reading changes the path of an operation that has a pick; the stamp already includes the picks.
