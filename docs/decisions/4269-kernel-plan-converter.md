## #4269 - Path kernel K6a: the converter from a kernel plan to a toolpath

The converter is in `crates/motion/src/kernel_emit/` (`mod.rs`, `pen.rs`,
`arcs.rs`). Motion calls the kernel; the kernel does not change. No old
planner and no app code calls the converter yet, so the output with the
default planner switch does not change. The first recipe (#3752) calls it.

**What was decided**

1. **The public surface.** `emit_plan(plan, plane, heights, feeds)` and
   `emit_levels(&[LevelPlan { plan, plane }], heights, feeds)` return
   `Result<Emitted, EmitRefusal>`. `EmitHeights::new(clearance_z, top_z)`
   and `EmitFeeds::new(cut, plunge)` are checked values. `Emitted` holds
   the `Toolpath`, the move range of each cut, and the counts of entries,
   stay-down links, lifts and exits. `EmitRefusal` has a named reason (a
   constant of the module), the level and the step.
2. **No decision of its own.** The converter emits the steps in the
   plan's order. It never drops a step and never adds a cut. Each step must
   start where the tool stands (within 1e-5 mm): a cut after its entry or
   its stay-down link, a stay-down link and a lead-out at the end of the
   cut. Else it refuses with `STEP_OFF_TOOL`. The plan does not check this
   continuity, so the converter does: a move between two steps would be a
   move that no stage proved.
3. **The heights.** One rapid plane: the clearance height. A lateral rapid
   is only at or above it. A rapid along the tool axis goes only up (the
   retract of a lift or an exit). The move down from the clearance height
   to the first approach point is a plunge-feed move, not a rapid: the
   rule "no rapid under the clearance height" holds without exceptions.
   The approach points must be at or above the stock top
   (`APPROACH_IN_MATERIAL`). A lift is a vertical rapid up to the
   clearance height. An exit retracts to its own height or to the
   clearance height, whichever is higher.
   Dropped: a rapid down to a retract plane above the stock (the old
   planners do this). The kernel plan has no retract plane for a lift,
   and a second rapid plane would need its own proof that it is above all
   stock. A later PR can add it if the time in air is too long.
4. **The lift note.** Each lift is counted by its reason in the motion
   `LinkTally`. At the end, one note for each reason goes to the path in
   the text form of `links.rs`: `link: the tool lifted to the clearance
   height N time(s) instead of the keep-down in-plane link - <reason>; ...`.
   The status line and the #4154 measures (`named_lifts`) read it.
5. **Arcs.** A cut whose curve holds an exact circle becomes G2/G3 arcs.
   The points must be on the circle (1e-3 mm) at one depth and turn one
   way (`ARC_OFF_CIRCLE`, `ARC_NOT_MONOTONE`). Each arc ends on a plan
   point after a quarter turn or a little more, so a full ring is four
   arcs and never one arc with the same start and end. Each other cut is
   the plan's chords, as feeds.
   **Arc leads, helices and ramps are chords.** The entry stage keeps the
   arc of a lead and the helix of a descent as chord points within its
   chord tolerance (#4178 point 5). The plan has no exact circle for them,
   so the converter emits feeds along the chords. The law measure then
   reads a chorded helix as a ramp (an XY travel without an arc). G2/G3
   leads and helices need the entry stage to record the circle first.
6. **The moves of each step.** The approach is fed in air (plunge feed
   down, cut feed else, a rapid only at or above the clearance height).
   Each descent move is a plunge-feed move. The lead, the cut, the
   stay-down link and the lead-out are cut-feed moves. A stay-down link is
   at the depth where the tool stands: the next cut must start there.
7. **The work plane.** The converter maps a plan point through the level's
   `WorkPlane`: Z is the plane origin plus the depth. It refuses each plane
   that is not the setup's XY plane (normal +Z, X axis +X, origin on the
   Z axis) with `NOT_SETUP_XY`, and each cut with a tool axis other than
   +Z with `AXIS_NOT_NORMAL`. The types do not assume XY: a lathe or 3+2
   plane is a new arm of the mapping.
8. **One plan for each level.** The caller plans each level: it gives one
   `Plan` and one plane (its origin on the level) for each Z level, in the
   present level order of `levels.rs`. When the stock state differs
   between levels (an upper level cleared the stock that a lower level
   enters through), the caller's entry stage reads that state through its
   `cleared` area and gives a different entry in that level's plan. The
   converter does not re-plan, merge or sort levels. Each plan ends with
   its exit and the next starts with its entry, so the move between two
   levels is an exit, a rapid at the clearance height and an entry.
9. **Notes of the stages.** The plan holds the entries, not the verdict
   of the entry stage. The caller writes the fallbacks and notes of the
   stage verdicts; the converter writes only the lift notes.
10. **A step kind that is not known** (`Step` is `#[non_exhaustive]`) is
    refused with `UNKNOWN_STEP`. No such kind exists yet, so no test can
    make one.

**The pins.** Unit tests in `kernel_emit/tests.rs` on hand-made plans.
The K4 pin of motion (`kernel_laws`) now builds the whole plan (guide,
order, link, and the entry and exit stage with a helix asked) and converts
it with the real converter: the counts stay (disc and dog bone Inside 0
lifts; Outside 13 and 15, each named; 0 crossings). The new pin
`k6a_plan_emit` runs the verifier of `camcraft-sim` and the entry measure
on the same converted plans.

**Readings** (tool radius 4, the 80 x 50 stock, one level at Z -2,
clearance Z 10, a helix asked):

| Fixture | Moves | Lifts (named) | Entries | Verifier |
|---|---|---|---|---|
| disc, Inside | 7355 | 0 | 1 into solid (read as a ramp: the helix is chords) | nothing |
| disc, Outside | 6402 | 13 (13) | 14 in air | nothing |
| dog bone, Inside | 5368 | 0 | 1 into solid (ramp) | nothing |
| dog bone, Outside | 4784 | 15 (15) | 16 in air | nothing |

The verifier area is the stock grown by four tool radii, less the part.
With the band of `fixture::band` (two radii) the verifier gave 72 and 84
`Gouge` warnings on Outside. Each one is on an air entry or its arc lead
outside the band, 17 mm or more from the part: the band was too small for
the air round the stock, and no warning is near the part. The law measure
also reads 44 (disc) and 52 (dog bone) Outside feeds whose centre leaves
the reach of `fixture::centre_reach` (the stock grown by one radius):
these are the same arc leads in the air. The K4 pin does not assert the
reach. The entry stage plans them in the kernel's air zone, so this is a
reading of the entry stage, not of the converter.

**The load pin of #4265 (`k3_load`).** It measures the cuts that the
load stage gives, with the old test pen. With the real converter its
Outside rows would also hold the arc leads of the entry stage (more
engagement readings, and feeds out of `centre_reach`). So `k3_load` now
reads `cuts_path_with`: the same plan, without the entry leads, with the
moves of the old test pen. A test compared the two paths: they have the
same moves and the same notes. Dropped: the converted path for `k3_load`
(it mixes the entry stage into a load reading, and it needs a relaxed
reach assertion).
The merge of `side/path-kernel` (df9ec30) into this branch showed that
`k3_load` is red on the side head itself: #4265 was gated on a branch
without #4253 (the link edge band). The disc Outside row with constant
engagement reads 337 moves above the limit and 77 unnamed, not 334 and
74. The old test pen gives the same 337. This PR pins the row as measured,
with a comment that names the cause.

**Cost to change later.** A retract plane under the clearance height, or
exact arcs for leads and helices, are local changes in `pen.rs` and need
no change of the public surface.
