## #3959 - Wire arc path: two neighbour arcs meet at the mean of their ends

**What was decided.** The wire contour of a drawn line-and-arc profile
(#3937, `crates/motion/src/wire_edm/arc_cut.rs`) moves each arc onto its
lattice centre with its radius kept. Two neighbour arcs with different
centres (tangent arcs, an arc and a pivot arc) then end a few nanometres
apart. Before this change `stitch` wrote a connector line across that gap.
The post rounds the line to the arc's end, so the program had a `G1` block
with no motion after each such arc.

The rule now:

- Two neighbour arcs whose ends are no more than `JOIN_MAX` (2.5 um, ten
  times the drawing's joint tolerance) apart meet at the mean of the two
  ends. The first arc ends there and the second arc starts there. No
  connector move is written.
- The rule applies only when the half gap is under a quarter of the chord
  of each arc. Then no end can pass the other end, and no arc can read as a
  full circle.
- Each moved end stands up to half the gap off its arc's circle. The path
  margin (the waste-side offset of #3937) now adds that half gap (the
  widest one, measured on the snapped first walk). The path says so in a
  note when the margin is not zero.
- The kerf and cover laws read the path as it is recorded. They read an
  arc whose two end radii differ as the circle of the mean radius, less
  half the difference (`arc_laws::move_span`). So the laws still certify
  the exact drawing.
- A gap wider than `JOIN_MAX`, or an arc too short for the rule, keeps a
  connector line. Such a gap is a real move of more than 2.5 um.

**Why.** A tangent arc chain (a DXF SPLINE or ELLIPSE read as tangent arcs,
#1865) had one zero-motion `G1` block for each arc. That doubled the program
length, and some controls warn on a block with no motion. The mean point
keeps each end within one gap of both circles, so the error is split
between the two arcs and stays in the nanometre range.

**Considered and dropped.**

- Meet at the intersection of the two snapped circles. Two tangent circles
  that are moved by up to 0.7 nm intersect at an ill-conditioned point,
  possibly far along the arcs, or not at all.
- Move only the second arc's start to the first arc's end. This puts the
  full gap on one arc. The mean puts half on each arc.
- Drop the snap of the arc centres. The CL lattice of the recorded centres
  is part of the CL contract (`cl::lattice`).

**Cost to change later.** Small. The rule is in `arc_cut::joins` and
`arc_cut::stitch`, and `arc_tests::tangent_arcs_join_arc_to_arc_with_no_connector_move`
and the `wire_dxf_selection` test
`tangent_arcs_post_arc_to_arc_with_no_block_that_stands_still` pin it.
