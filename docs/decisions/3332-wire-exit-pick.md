## #3332 - Wire Contour takes an exit pick

**What was decided.**

1. **The capability.** `PickCapability::Threading` takes a start and an exit. The start pick stays the threading point (PR #3871 moved it into `picks`).
2. **An open run.** The cut ends at the end of the run that is nearest the exit pick. The wire rides the same line on the same side. Only the direction of the walk changes. The kerf side stays "left / right of the run's point order", so the kept side does not change when the walk turns round.
3. **A closed profile.** The cut is the automatic cut, all the way round from the threading point. The wire then rides its own kerf on, in the same direction, to the point of the path nearest the exit pick. It leaves by a straight lead-out to the pick and does not return to the threading point.
4. **The laws of an exit pick, refused by name and never moved.**
   - A pick the same distance (1 um) from the two ends of an open run names no end.
   - A pick on the wire path, a pick nearer the profile than the kerf offset, and a pick on the product side (outside a die, inside a punch).
   - The lead-out cannot cross the profile when the pick is on the waste side at the kerf offset or more. The kerf certificate of the generator judges the lead-out as each other move.
5. **Pairs that are refused by name, at the command door and at generation.** An exit pick with tabs, skims, a taper, a no-core burnout or a 4-axis cut. A selected DXF chain that is cut on its own arcs refuses the pick at generation.
6. **The Geometry controls.** The Wire Contour page shows an Exit point group beside the Start point group. A checkbox states the exit. The X and Y rows edit its coordinates, and Clear exit removes it. Apply commits the draft through the existing undoable command.
7. **Where the code is.** The exit is a new module, `motion::wire_edm::exit`. It generates the cut without the pick first, with the existing generator, then reads the exit. A cut without an exit pick does not enter the module.

**Why.**

- The issue states the open run rule ("the cut ends at the end nearest the exit pick"). The closed rule follows the 2D Contour ring: all the way round, then on to the pick.
- Each refused pair starts or ends its passes at the threading point. The post reads a skim pass by its return to the threading point (`posting_wire::announce_skims`). The tab cut-off starts there with the wire threaded (`wire_edm::tabs`, `cutoff_split`). The taper builder reads the flat path as one ring with its return. The 4-axis cut pairs its profiles from the start hole, so the sync pair rule holds without a change.
- The separate module keeps every shipped wire golden byte-identical: no line of the existing generator changes.
- The review found that a stated exit changed the path but was hidden on the Geometry tab. Typed coordinates and Clear exit use the same draft workflow as the threading point.
- PR #3617 (open, `conflicts`) changes `cut_pass` and `wire_open`. Its head is older than PR #3871, so this work cannot stack on it. The separate module changes none of the functions that #3617 changes.

**Considered and dropped.**

- An `exit` field in `WireParams`, read inside `cut_pass` and `wire_open`: dropped. It changes the same functions as #3617, and each test that builds a `WireParams` must change.
- Reverse the point order of an open run and flip its side: dropped. The stated side is of the run's point order, and a silent flip of the side is a move of the kerf.
- Ride the shorter way round to the exit on a closed profile: dropped. The same direction is one rule, and it is the rule of the 2D Contour.
- An exit pick with tabs (the issue says "before the tab or the cut-off"): refused for now. The main cut of a tabbed profile returns to the threading point one time for each stretch, and the cut-off starts there. An exit before the cut-off needs a second threading, and the post has no word for it.
- A start pick that chooses the first end of an open run: not changed. An open run ignores the start pick today, as before this change.
- Viewport picking for the wire exit: deferred. The existing viewport handler takes only a 2D Contour on its Leads tab. Typed rows let the user see, set, edit, and clear the wire exit without changing that handler.

**Cost to change later.**

Low. The exit rules are in one module with its own tests. A later slice can lift a refused pair (tabs, skims, taper) without a change to the file format: the pick is already in `picks.exit`.
