## #4039 - a work bottom at the lower guide is within reach

**What was decided.** The guide-reach law of the wire EDM compares the
work with each guide plane within a tolerance, `GUIDE_REACH_TOL` in
`app::machine::wire_edm`. It is the wire path's own tolerance,
`wire_edm::PATH_TOL` (10 nm). One pair of helpers (`below_guide`,
`above_guide`) is used by the four places that apply the law:

- the span check of the wire laws (`operation::wire_verify`, all wire posts:
  generic ISO, Sodick, G-EDM),
- the pose check of a tapered or 4-axis cut (`machine::wire_edm::judge_pose`),
- the native Agie job check (`operation::agie_profile`).

The messages write heights through `reach_z` / `guide_z`: three
decimals, never "-0.000". When a refused height and its guide read the
same in three decimals, six decimals are written, so a refused gap never
reads as no gap.

**Why.** The user put the origin at the part bottom on a machine whose
lower guide is at Z0. The work bottom (the lowest facet of the part or
stock, in world Z) and the guide plane (the guide Z plus the WCS origin Z)
come from two different float paths. The origin is written from a box
(the anchor buttons read the stock box or the part bounds, in the setup
frame), and the facets are moved by their own transform. The two values
can differ by round-off, for example 1e-12. The strict `<` comparison
read that round-off as a gap and printed "Z-0.000". A tolerance far
below the 0.001 mm that a program writes removes the round-off and
keeps each real gap refused.

**Dropped.**
- Snapping the origin or the work bottom to the guide: it changes the
  values that the post writes, and it hides where the round-off came from.
- A tolerance of 0.001 mm (the anchor-name tolerance of `setup_origin`):
  it lets a gap pass that the program could write.
- A new numeric constant: the brief asked for an existing length
  tolerance. No single project-wide length tolerance exists, so the wire
  family's `PATH_TOL` is used.

**Cost to change.** One constant. The tests in
`crates/app/tests/wire_taper.rs` pin the round-off cases and a 0.0001 mm
gap.
