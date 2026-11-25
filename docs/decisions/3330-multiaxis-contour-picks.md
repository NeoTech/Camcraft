## #3330 - Start and exit points: the Multi-Axis Contour slice

**What was decided.** The Multi-Axis Contour (`OpType::MultiAxisContour`,
the drive-curve planner `multiaxis::contour_curve`) takes a start point
and an exit point (`PickCapability::Ring`). The picks act on the picked
drive curves before the walk (`multiaxis::curve_seam`,
`multiaxis::contour_curve_seamed`):

- One curve for each pick: a pick acts on the drive curve nearest it.
  The other curves keep their automatic cut. This follows the #1448
  design (one pick for each chain) and `PICKS_PER_ROLE = 1`.
- A pick with a Z is read in 3D. A pick without a Z is read in XY.
- Closed curve, start: the curve is re-sequenced to begin at its point
  nearest the pick. A foot between two vertices becomes a new vertex on
  that segment. A foot within the dedupe distance of a vertex starts at
  that vertex.
- Closed curve, exit: the curve cuts its whole lap from its seam, then
  runs on along its own points to its point nearest the exit pick, where
  the tool leaves. The carried curve is walked as an open run.
- Open curve, start: the cut starts at the end nearer the pick. The
  direction of the curve can reverse.
- Open curve, exit: the cut leaves at the end nearer the pick. With a
  start pick on the same curve, the exit must be nearer the far end.
- Refused by name, never moved:
  - a pick farther than 1.5 tool diameters from every drive curve;
  - an open curve's selected end farther than 1.5 tool diameters from
    the pick (a click in the middle of an open curve does not snap to
    an end 50 mm away);
  - an exit pick nearer the end where the start pick begins the cut;
  - a plunge column at a picked start, or a retract column at a picked
    exit, that crosses the part (`curve_seam::column_clear`: the ball
    swept up the vertical line to the hop plane every 0.25 mm, and the
    shank at each step when shank avoidance is on).
- The tilt limit, the shank dodge and the rest law: the walk runs on the
  re-sequenced curve, so each station of the picked path is posed and
  judged by the same laws as the automatic path. The app's centre-cut
  audit (`audit_posed`) and the verify pass also read the picked path.
- The operation has no leads, so there is no lead-room law to hold.
- The cut order of the curves does not change.
- An empty pick list leaves the curves as they are: every path without a
  pick is byte-identical (pinned at motion scale).

**Why.** The walk (`curve_walk`) already treats a closed curve as a
cycle and an open curve as a run with two ends. A re-sequence of the
curve points before the walk keeps each station law on the picked path.
A re-sequence of the posed output would need its own copy of the rest,
pose and shank laws.

**Dropped.**
- One seam for all curves (the 5-Axis Contour rule): the drive curves
  are separate picks of the user, often far apart. A pick far from a
  curve would seat that curve at a point that the user did not choose.
- A start that snaps to the nearest vertex only: a coarse ring (a box's
  four corners) would then start at a corner for a click at mid side.
- A re-order of the curves so that the picked curve is cut first: this
  changes the cut order, which the slice must not do.
- A column check at the automatic seam: it would change the output of
  existing files. The automatic seam keeps the verify pass as its guard.
- The 860-line `curve.rs` holds only the hook. Its unit tests moved to
  `curve_tests.rs` (no test change), so the file is shorter after this
  PR than before.

**Cost to change later.** Small: the law is in `curve_seam.rs`, with
one hook in `contour_curve_seamed` for the re-sequence and one for the
columns. A per-curve list of picks needs only `apply` and the pins.
