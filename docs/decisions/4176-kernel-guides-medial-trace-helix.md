## #4176 - Path kernel K2, second part: the medial line, the trace and the helix guide planners

Stage 2 of the K0 record (`## #4024` in `RECOMMENDED.md`, "Stage 2: Guide
curves"), second of two sub-PRs. The first part (PR #4189,
`4176-kernel-guides-rings-rows.md`) gave the ring and the row planners.
This part gives the medial line, the trace of a chain and the helix (with
the exact circle rings of a hole). No old planner changes, and no planner
calls the new code yet: with the default switch the output is identical.

**What was decided**

1. **Files.** `crates/pathkernel/src/guide/`: `medial.rs`, `trace.rs`,
   `helix.rs` (the helix and the exact circle rings), `lead.rs` (the lead
   of a helix curve and its checked constructor) and `choice.rs` (the
   guide choices, moved out of `guide.rs` with no change, so that
   `guide.rs` stays under 400 lines; the public paths stay the same by
   re-export). In `camcraft-geometry`, three new files of `geo2d/`:
   `delaunay.rs` (crate-private), `medial.rs` (`Region::medial_axis`) and
   `side_offset.rs` (`offset_side`). They are new files, so that the
   geometry change of K4 (#4177) merges without a conflict except the
   module list of `geo2d.rs`. No new dependency edge between our crates.
2. **The medial line is the medial axis of the cutter-centre area** (Blum
   1967). camcraft-geometry had no medial function. The axis is computed
   as the Voronoi diagram of points sampled on the boundary (Brandt and
   Algazi 1992), as the duals of the Delaunay edges (Bowyer 1981, Watson
   1981, with the visibility walk of Devillers et al. 2002). A Voronoi
   edge is kept when its two triangles have their centres inside the area
   and its two boundary points subtend 60 degrees or more at it (the
   theta-medial axis, Attali and Montanvert 1996, Foskey et al. 2003).
   - The sample spacing is `sqrt(8 * tol * 0.25 mm)`, so the axis is
     within the tolerance where the clearance is 0.25 mm or more.
   - Each branch is simplified by Ramer-Douglas-Peucker at half the
     tolerance.
   - Side branches shorter than four sample spacings are sampling noise.
     They are left out and their count is named in a note.
   - The samples are sorted before the triangulation, so the axis
     depends on the boundary point set only, not on the loop order or
     the start vertex (law 1, pinned).
   - Why 60 degrees: the centre line of a slot subtends 180 degrees and
     the branch into a 90-degree corner subtends 90. The branches of a
     flattened arc (corners flatter than 120 degrees) are noise.
   - Each branch is a curve of family `Medial`, index 0. A branch round
     a hole is closed. The verdict is done; the uncut area beside the
     line is named in a note (new `Ask::Line` of the one guard
     `cover::judge`), because a centre line does not clear an area.
   - Dropped: a port of the old `adaptive/medial.rs` spine. It averages
     the two sides of one channel and refuses any other shape (an
     island, a fork). The medial axis gives each branch and keeps the
     clearance radius of each point for later stages (V-Carve depth,
     the trochoidal width).
   - Dropped: a straight skeleton (it differs from the medial axis at a
     reflex corner, so the tool would not be centred).
3. **Medial refusals.**
   - `NO_MEDIAL`: no branch is longer than the tolerance. A round area
     has a centre point, not a line.
   - `NARROW_NECK`: a piece of the area is narrower than the tolerance
     at a point, or touches itself. The test is the topology of the
     erosion by half the tolerance: the piece splits, or one of its
     holes opens. A test on the sampled clearance radius cannot see a
     neck that is narrower than the sample spacing, so it was dropped.
   - `MEDIAL_TOO_FINE`: more than 200000 boundary samples.
   - `NOT_MEDIAL`, and `NOT_AN_AREA` for the chains of a trace.
4. **The trace needs a region that holds the chains.** The region of an
   area (K1) refuses an open chain (it has no material side), and its
   part walls are cutter-centre loops, not the selection. So a new
   variant `KernelRegion::Chains(ChainRegion)` holds the picked chains,
   the work plane, the tool and the keep-out area. It has no material
   side and no start side. `chain_region(sel, plane, tool, keep_out)`
   builds it. `KernelRegion` is `#[non_exhaustive]`, and the record says
   that a later phase adds a region that is not a planar area as a new
   variant. `KernelRegion::chains()` and `KernelRegion::tool()` are new.
   The planners of an area refuse a chain region by name (`NOT_AN_AREA`).
   The kernel's entry stage refuses it today (`NOT_PLANAR` of
   `entry::stage`), so a trace cannot run through all stages yet. That is
   owed to the entry stage before the switch-over (follow-up issue #4236).
   Dropped: the chain in the guide choice (`GuideChoice` is `Copy`, and
   the chain is region data with its plane); a planar region round the
   chain (it would hide the chain behind an area).
5. **Trace offset.** `TraceOffset` above zero is the left of the chain in
   its point order, below zero the right, zero the chain itself.
   - A closed chain is read counter-clockwise, so its left is its
     inside. One rule gives the same curve for either pick direction
     (pinned).
   - A closed chain is offset as its area (`Region::offset_fine`, round
     joins at the tolerance).
   - An open chain uses the new `offset_side` of camcraft-geometry: the
     boundary of the disc sweep of the chain (the Minkowski sum), less
     the end caps and the other side. The parts that the chain comes
     back over are trimmed (the global invalid loops of Tiller and
     Hanson 1984). An offset that splits into pieces is named in a note.
   - A chain whose offset collapses is named in a note. If all chains
     collapse, the trace is refused (`TRACE_COLLAPSES`).
   - The stretches where the tool disc meets the keep-out area (grown by
     the tool radius) are left out, and their length is named in a
     note. If the whole trace is in the keep-out area, it is refused
     (`TRACE_KEPT_OUT`).
   - Each curve is of family `Trace`, index 0. The trace has no coverage
     guard: it does not clear an area.
6. **Exact circle rings of a hole** (Circular, Bore; the old
   `CIRCLE_NOT_YET` is gone).
   - A rings choice with a circle goes to the circle rings of
     `helix.rs`. The circle is the boundary of the cutter-centre area of
     the hole. The planner refuses an area that is not its disc
     (`CIRCLE_NOT_REGION`: one loop, each vertex and edge midpoint
     within the tolerance plus the 5 um offset flattening).
   - Ring `k` has the radius `circle - first_offset - k * stepover`
     while it is above the tolerance. This is the same level-set rule as
     the ring planner.
   - Each ring carries its exact `Circle` (`Curve::arc`) and a carrier
     polygon with the least chord count of 8 or more whose sagitta is
     within the tolerance (#3756: no fixed 256 chords). The guard and
     the marks read the polygon. A later stage posts the circle.
   - A clearing choice whose last ring's disc does not reach the centre
     gets one clean-up circle at half that radius, marked `Fill`.
   - The full-width marks and the one guard are those of the ring
     planner, so the guard decides done or incomplete. A ring count
     names the uncut area in a note.
   - The refusals of the ring planner apply: `WALL_STRIP_UNCUT`,
     `NO_RING`, `OFFSET_BELOW_CENTER`, `TOO_MANY_RINGS`,
     `STEP_ABOVE_DIAMETER`.
7. **The helix** (Thread, a helical bore).
   - `GuideChoice::Helix(HelixChoice)` is new: the helix circle (the
     cutter-centre path), the pitch, the drop below the plan level, and
     the hand. `HelixChoice::new` refuses more than 10000 turns
     (`TOO_MANY_TURNS`).
   - The planner gives one open curve of family `Helix` from depth 0
     down to minus the drop, from the angle 0 (the first axis of the
     plane). Its chord count per turn follows the tolerance as above.
   - The hand is the shape of the curve: a right hand rises when it
     turns counter-clockwise. The direction of travel stays with
     stage 4.
   - `Curve` has a new field `lead: Option<Lead>` (pitch and hand) and a
     checked constructor `Curve::helix`. It refuses a point off the
     circle, a step of half a turn or more, and a depth that does not
     follow the pitch and the hand (`NOT_ON_HELIX`). `Curve::new`
     refuses the helix family (`HELIX_NEEDS_LEAD`). So a helix curve is
     valid by construction. The canonical order and `same_cut` read the
     lead.
   - Refused: a helix outside the cutter-centre area (`HELIX_OUTSIDE`),
     more than one million points (`HELIX_TOO_FINE`), and another choice
     (`NOT_HELIX`).
   - One helix for each choice: a thread with several infeed passes asks
     for one helix for each pass radius. Dropped: a list of radii in the
     choice (`GuideChoice` stays `Copy`).
8. **`Guides` plans every family.** `FAMILY_NOT_YET` is gone. Rings
   (circles included) go to `RingPlanner`, rows to `RowPlanner`, the
   medial line to `MedialPlanner`, the trace to `TracePlanner` and the
   helix to `HelixPlanner`. No family is planned as another one (law 4).
9. **The guide stays open for later phases.** The curves are in the 2D
   frame of the region's work plane (both region kinds carry it). A
   point has a depth along the plane normal (the helix uses it). No type
   names XY. A surface guide or the lathe plane is a new region variant
   or a new choice, because both enums are `#[non_exhaustive]`.

**Laws.** The medial line, the circle rings and the helix run through the
law kit (`kit::run` with the stand-in order, load, link and entry stages)
and hold L1 to L4 (`guide_laws.rs`). The kit's L4 check now maps the
helix choice to its family. The trace cannot run through the kit's stages
(the entry stage plans an area, decision 4). Its laws are read in
`guide_trace.rs` with the kit's measures:
- L1: a second call, the chains reordered and restarted, and `Guides`;
- L2: at compile level (the chain region takes no material side or
  start side);
- L3: the stand-in order stage for each start side;
- L4: every curve is of the asked family.

**Agreement with the old output** (motion pins in
`crates/motion/tests/pass_start_levels/kernel_guides.rs`; the guide
geometry only):
- Trace (plate with a hole, round loop) and Engrave without an avoid
  item: every old cut point is on the kernel trace, and every kernel
  vertex is an old cut point.
- Slot as wide as the tool: the trace of the centre segment is the old
  cut.
- Circular (hole 15, tool 8, step 3): kernel radii 11, 8, 5 and 2 are the
  old arc radii at each level.
- Bore (hole 10, tool 6): with the stepover `rc / count` (7/3), the kernel
  radii 7, 4.667 and 2.333 are the old ones.
- Thread (internal, right and left hand): the old helix radius, the pitch
  of 1.5 and the hand agree.
- Named differences:
  - Bore spaces its rings evenly (`rc * k / count`); the kernel spaces
    them at the stepover from the wall. The recipe gives `rc / count`.
  - The old rings are fixed chord counts; the kernel rings are exact
    circles (#3756).
  - The wide slot (18 mm, tool 10): the old plan cuts rows across the
    centre band (y = -4, -1, 2, 4). The medial line is the one centre
    line (y = 0), inside those rows. Rows across a slot are the row
    family.
  - Engrave with an avoid item is not compared. The old keep-out comes
    from the avoid item with its own pad; the kernel takes the keep-out
    as a value.

**Why**: the record fixes the guide signature and the laws, but not how a
centre line, a trace of an open chain or a helix is represented. The
choices keep each guide a pure function of the region and the choice
(laws 1 to 3), name every substitution or loss (law 4, "nothing is
dropped without a note"), and keep the exact circle data for the post.

**Cost to change later**: low for the planners (internal to the kernel).
The chain region variant and `Curve::lead` are public. A later change to
them changes the callers of the kernel, and there are none before the
switch-over.

**Later stages keep the data of a family** (the review of PR #4244 at
33c116cfa): the K4 order stage reversed a curve with `Curve::new`, which
refuses the helix family, so a valid helix guide set was refused with the
constructor's reason `HELIX_NEEDS_LEAD`.
- Chosen, one rule for the class: a later stage that restarts, reverses
  or replaces a curve makes it again through the constructor of the
  curve's family (`Curve::with_points`: `Curve::helix` with the same
  circle and lead, else `Curve::new` with the same circle), or it refuses
  by a reason of its own stage. The table test
  `later_stages::each_guide_family_keeps_its_data_or_is_refused_by_the_stage_in_each_later_stage`
  pins each family (ring, exact circle ring, row, medial, trace, helix of
  each hand) against each later stage.
- Order and a helix: a helix always runs down its lead. A reversal in the
  plane also turns the helix up its lead (the same circle and lead, the
  other order of points), and up is never a valid cut of a descent (the
  tool would start at the bottom, in material). So the order stage
  reverses a helix only when the guide set holds it upward (the canonical
  guide set has no direction). The hand then sets the side of the
  material: in a hole, a right hand cuts conventional and a left hand
  climb. When the asked direction is the other one, the stage keeps the
  helix down its lead and names it with the order note
  `HELIX_HAND_SETS_DIRECTION` (choose the other hand). A failed rebuild
  is `NOT_REVERSIBLE` (an order-stage reason).
  - Dropped: a refusal for a hand against the direction. The hand is an
    explicit part of the helix choice (a thread has one hand), and the
    default direction (climb) would refuse every right-hand helix in a
    hole. The note keeps the mismatch visible.
  - Dropped: a mirror of the hand. A helix of the other hand is another
    curve, not a permutation (law 3).
- Load and a helix (K3, PR #4265): constant engagement inserts passes
  only before a ring; a helix on a circle round the centre of a hole is
  at one distance to the wall, but it is not a level of the field, so it
  is named `NOT_A_RING`. Trochoidal loops are flat and have no circle:
  a curve with a lead, an exact circle or a changing depth keeps its
  shape, and its full-width cut is named `SHAPE_KEPT`. A rebuilt set that
  is refused is the load stage's `SET_NOT_KEPT`, and loops that make no
  curve are `LOOPS_NOT_A_CURVE`.
  - Dropped: loops on an exact circle ring without its circle. The
    circle is data for the post; the loops would drop it silently.
- The link and the entry stages read a curve and make no new one.
