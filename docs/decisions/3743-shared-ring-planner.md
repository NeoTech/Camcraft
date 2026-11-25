## #3743 - One shared ring planner: ring set, ring links, ring entry

**Decided.** The ring ops plan their rings, links and entries in three
shared modules of `crates/motion/src/machining2d/` (research result 2 on
#3743, "The shared design"). Each op gives only its own parameters.

1. `ring_set`, the one ring planner. Input: the cutter-centre area, the
   reference edge (`RingRef::Part` = the selected contour, the 2D Offset;
   `RingRef::Boundary` = the walls of the area, the pocket), the first
   offset, the stepover and the cut sense. Output: the rings, closed or
   open, clipped by the area, in the order of the start side. "Pass starts
   at" gives only the order of the rings and the start point. Auto selects
   Inside or Outside by the #3744 rule (`ring_side::resolve`). It never
   selects another planner.
2. `ring_links`, the one linking rule. Stay at depth. Link ring to ring in
   the cut plane, through cut material or air. The proof is the keep-down
   proof of `ladder_links` (`link_proof` + `through_cut`). A lift is taken
   only where each in-plane link fails, and the tally names the reason
   (`LinkTally::lift`).
3. `ring_entry`, the one entry rule, in this order: an air entry outside
   the stock box by the approach distance (#3745), then a G17 arc lead in
   the cut plane (#3746); a proved pre-drilled hole; a cleared column; a
   helix or a ramp only where the tool must enter material; else a refusal
   with a named reason. Never a silent plunge.

The HSM spiral, the rows of Face and Slot, Thread and the trochoids are not
rings. They share only `ring_entry` and the lift reason.

**Migration.** One op for each step (the table of result 2 on #3743):
#3750 (step 1, this design taken from the 2D Offset path, identical
output), then #3752 (2D Pocket), #3753, #3754, #3755 (Circular), #3756
(Bore), #3757, #3758, #3759 and #3760. A P2 step keeps the CL moves of its
pinned tests identical (a golden recorded on the head before the step). A P1
step keeps the ring radii or the ring multiset within 1 um, and pins each
lift, entry or ramp count that it fixes red first.

**The pass link default of the ring ops (recorded, not yet built).**
- Recommended: a new seed `Links::RINGS` (`pass: KeepDown`, else as
  `PLANAR`) for the ring ops. A ring op of an old project file that stores
  the `PLANAR` default loads as `RINGS`, because `Retract` at each pass
  break is the defect of #3741 and #3423. A user who sets Retract by hand
  gets Retract, and the status line says so.
- Dropped: keep `Retract` for old files. The old files then keep the
  needless lifts.
- Dropped: ignore the user's pass link. That removes a user parameter.
- Not built in #3750: the seed changes the output of old project files,
  and step 1 must keep the output identical. The first step that migrates
  a ring op whose output may change (#3752) builds the seed and its load
  rule, with its own pins.

**Why.** The maintainer (#3743): "so we are not having 18 different ways of
doing ring planning." (#3741): "The only thing that differs and inside and
outsided strategy is actually where the tool starts.. inside the bounding
box, or outsided the bounding box." One planner, one link rule and one
entry rule give each op the same safety proofs. A fix in one place reaches
each op.

**Cost to change later.** Each later step moves one op onto the modules.
A change of the contract (`RingSet`, `RingRef`, `RingEntry`) touches each
op that has moved, so the contract stays small until the path kernel
design (#4023, #4024) is settled.
