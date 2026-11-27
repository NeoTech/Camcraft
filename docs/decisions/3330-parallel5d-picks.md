## #3330 - Start and exit points: the 5-Axis Parallel slice

**What was decided.** The 5-Axis Parallel (`OpType::Parallel5D`) takes a
start point and an exit point (`PickCapability::Raster`, the existing
"corner of the first raster row and end of the last row" capability).
The picks act on the raster's passes before they are linked
(`finishing3d::raster_seam`, `finishing3d::parallel_seamed`,
`multiaxis::parallel5d_seamed`):

- A zigzag raster can start only at one of four corners: each end of
  the first pass and of the last pass. Each corner is one order of the
  same passes: reverse the pass order or not, and run every pass the
  other way or not. The zigzag alternation stays in all four.
- Start: the corner nearest the pick. Exit alone: the order whose last
  pass ends nearest the pick. Start and exit: of the orders whose start
  is within reach of the start pick, the ones whose end is within reach
  of the exit pick; the smallest sum of the two distances wins. A tie
  keeps the automatic order first.
- A pick with a Z is read in 3D (to the tool tip). A pick without a Z
  is read in XY.
- Refused by name, never moved:
  - a pick farther than 1.5 tool diameters from every corner of its
    role (the same reach as the contour and the slice 2 drive curves);
  - an exit that no order from the start pick ends near ("a raster that
    starts at the start point ends at ...");
  - a pick on a raster with no pass (no shallow ground in the depths);
  - a seam on a pattern other than the one-direction Parallel raster
    (only Parallel5D reads a seam, and it uses that pattern).
- Cut direction: the raster cuts both ways by its own zigzag law and
  does not read `climb`, and Parallel5D has no one-way mode. So no
  order asks for a direction the operation forbids, and there is no
  direction refusal to make. The waterline rings that follow the raster
  keep their own climb order: the pick does not touch them.
- The geometry of each pass does not change. The links between passes
  are planned on the new order by the raster's own link laws
  (`march::emit_runs`: certified straight links or the hop with its
  clearance fallback).
- The tilt limit, the shank dodge and the rest law: the stamp poses the
  re-ordered plan, so each station is judged by the same laws as the
  automatic path. The app's centre-cut guard reads the picked path.
- The plunge column at a picked start and the retract column at a
  picked exit: no new refusal. Every corner is a raster sample, and the
  raster keeps a sample only where the ball's vertical column to the
  clearance plane is clear (`support::plunge_support`, the column law).
  With shank avoidance, the stamp checks the shank at each cut station
  with a column that is open upward (`avoid::shank_hits`), so the
  plunge station and the last cut station already prove the shank
  column. A new check would be dead code: no input can make it fire,
  and no mutation of it can turn a test red. The pins hold the law
  that the pick relies on: under a canopy the trimmed corner is refused
  by name, and a pick beside it starts at a sample whose column a
  0.25 mm ball sweep proves clear. With the trim mutated away, that pin
  turns red.
- An empty pick list leaves the passes as they are: every path without
  a pick is byte-identical (pinned at motion scale against
  `finishing3d::parallel` and `multiaxis::parallel5d`).

**Why.** The four corners are the only places a zigzag of fixed passes
can start without a change of the passes. An order change before the
link step keeps every link law on the picked path. A re-order of the
posed output would need its own copy of the link, pose and shank laws.

**Dropped.**
- A start at the end of a middle pass: the raster would then cut its
  passes in two groups with a long link between them, which changes
  the path more than the issue asks ("the first pass nearest the pick").
- A snap of a far pick to the nearest corner without a refusal: a click
  in the middle of the raster would start the cut 20 mm away from it.
- A new column refusal at the picked corners (see above): dead code.
- A UI row on the operation page: as in slices 1 and 2, the picks ride
  `edit_operation` and the viewport pick; the page shows no
  "not taken" text for the kind any more.

**Cost to change later.** Small: the law is in `raster_seam.rs` with one
hook in `raster_pass_seamed`. RotaryParallel can reuse `order` when its
slice ships.
