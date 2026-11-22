## #3444 - Mesh a hole on a pocket wall

**What was decided:** If the display planner reports a face above its chord
tolerance, try the existing constrained mesher with the same chord tolerance.
Do not start this fallback after cancellation. Keep the existing checks for
planner shortfalls, continuous deviation, weld movement, triangle count,
watertightness and positive volume.

**Why:** The reported STEP plate reaches the display planner's refinement
limit on its stepped cylindrical face. The unchanged code refuses face 12
at 3.306568 mm sag for a 0.005 mm tolerance. The constrained mesher in the
pinned Solidcore version gives the existing checks another plan to judge.
The STEP regression passes with that plan. The normal display path stays
the first choice because it costs less.

**Considered and dropped:** Do not relax the tolerance or ignore the capped
face. Either choice could admit an unsafe machining mesh. Do not change
the Solidcore dependency or add a triangulation algorithm. The existing
public API provides the required alternative within this issue's scope.

**Cost to change later:** A different recovery path must retain the same
acceptance checks and pass the pocket-wall regression tests. Moving the
fallback into Solidcore would require a dependency update and new review
of its mesh bounds.
