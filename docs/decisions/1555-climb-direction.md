## #1555 - climb milling runs climb: outside clockwise, inside counter-clockwise (M3)

**Decided.** With a right-hand spindle (M3, clockwise seen from +Z), climb
(down) milling keeps the material on the RIGHT of the tool's travel: an
outside profile runs clockwise round the part, an inside profile (a hole
or pocket wall) counter-clockwise, and control compensation is G41 (cutter
left of the programmed contour). Conventional is the reverse of all three.
The 2D Contour family had the reading inverted ("material on the left" was
called climb); it now follows the handbook convention:

- `machining2d::contour` - outside / inside ring travel (`climb == inside`
  walks the stored ring), the air side the leads, fillets and arc entries
  stand on (`air_side`, `Legal::air_left`), and the compensation side
  (`contour_comp::side_of`: climb = G41, conventional = G42, wear and
  control alike). The comp room's arc law (`contour_comp_room::arcs`) now
  reads the tool side from the travel (climb = left) instead of from the
  contour side - the old rule was only right for one of the four cases.
- The open-boundary contour (`app::operation::contour_open`) hands its
  run on with the material on its right; before, its order was the
  chain's lexicographic order, unrelated to the side.
- 2D Chamfer takes the contour's side: a boss edge (Outside / Auto) runs
  clockwise under climb, a bore edge (Inside) counter-clockwise.
- Seams do not move: each ring keeps the seam the pre-#1555 default put it
  on (an outside ring's first point, an inside ring's last - the
  tessellated bore's axis-aligned point its arc entry posts on as one
  G18/G19 arc); climb and conventional differ only in direction
  (`ring::walked`).

**Why.** Machinery's Handbook ("Milling Practice", climb vs conventional
milling) and the shop-floor convention every CAM manual states: in down
milling the tooth meets the stock at full chip thickness and leaves it
thin, which with a clockwise spindle means the stock lies right of the
feed direction; RS-274 / ISO 6983 define G41 as the cutter left of the
contour along the direction of travel - so climb is G41. Adaptive, Bore,
Circular and the 2D Pocket already followed it; only the contour family
was inverted.

**Spindle direction.** Every 2D mill op posts M3 (only thread milling can
ask for M4, and it has its own hand law), so the direction is derived for
M3 and not read from the spindle - the same as Adaptive, Bore and
Circular. If a mill op ever gains a reverse spindle, every climb rule
flips with it and must read the spindle then.

**Checked, not changed.**
- 2D Pocket (Waterline, Offset rings, Adaptive): the wall pass already runs
  counter-clockwise under climb - pinned (motion and app), not changed. The
  issue's pocket row did not reproduce on main (block-with-hole fixture,
  every strategy).
- Zigzag pocket rows, Face rows and Slot rows are serpentine - half the
  rows are climb and half conventional whatever the flag, which only picks
  the first row's direction. Left as is.
- Trace and an open centerline contour run ON the line (material both
  sides or none): climb has no side there. Trace is unchanged; an open
  centerline run keeps its travel, and its lead now leaves to the other
  side (the shared air-side rule) - both sides are air as far as the run
  knows.
- Deburr has no climb parameter (its direction comes from the edge chain).

**Breaking.** Posted G-code of every existing project with a 2D Contour or
2D Chamfer changes direction (and G41/G42 swaps under control
compensation). A project that set `climb: false` as a workaround now posts
conventional. The PR is marked `fix!`; the release notes must say so.

**Dropped.** Flipping the flag at the app boundary (`climb: !climb` into
motion) - it would keep the inverted word inside the generator and in
every motion test; and changing the axis-aligned reseat rule to fix the
bore seam - keeping each ring's seam is smaller and moves no golden but
the direction.

**Cost to change later.** Low: the rule is `climb == inside` in two places
plus `air_side` / `side_of`; the pins in `crates/motion/tests/climb_direction.rs`
and `crates/app/tests/climb_direction.rs` state it.
