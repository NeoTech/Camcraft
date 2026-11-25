## #3933 - 2D Pocket Offset rings: one outward spiral, rings spaced by the engagement law, wall ring last

**What was decided.** The 2D Pocket's Offset rings strategy keeps
contour-parallel rings (offsets of the cutter-centre area), but it no
longer cuts them one by one with a straight step from ring to ring.

- Ring spacing comes from the engagement limit (`offset_law`). The core is
  the deepest ring of the old tiling at `optimal_load x D`. Out from it,
  each step is the one that holds the load's contact angle on the ring's
  tightest round stretch: the constant-engagement law for a circular path
  (Stori & Wright 2000), read on the same rim as the plan's load read,
  with the spiral's climb taken off the angle. On straight walls the step
  is the full load width; round a small core it is smaller. A floor of
  15 % of the width stops an endless ladder at a point-like core.
- The core ring is cut closed (the entry's own bore cleanup, as before).
  Then each revolution morphs from one ring into the next ring out
  (`adaptive::spiral::revolution`, Held & Spielberger 2009, the HSM
  planner's own morph). The last ring of the chain (the wall ring) is cut
  as one full lap: the "wall clearing" ring that the maintainer asked for.
- A morph is used only where the two rings bound one band: adjacent
  offsets, one inside the other, the same direction, and the morph
  crosses no ring of the two offsets and not itself. Then it stays in the
  band, which is inside the cutter-centre area, so it cannot gouge. Where
  this is not true (rings that a narrow place splits, sibling rings), the
  proved chord of the ring plan stays.
- Completeness guard: a spiral that leaves more than 0.1 mm2 of the area
  unswept is not posted. The rings are then cut one by one, with a note.
- The plan is read on the HSM planner's stock (`adaptive::worst_load`, the
  same grid and contact law). Past the load plus 5 % (`PLAN_TOL`), a note
  names the load and the place. A core ring longer than one orbit of the
  tool is a full-width slot and is named too. On a round pocket the read
  holds the load (25.2 % for a 25 % load); at the sharp corners of a
  rectangle it does not (the classic offset-path corner), and the note
  says so and names Adaptive.
- Adaptive's fallback to Offset rings in the 2D Pocket uses the same
  spiral, so the fallback note "Offset rings cut ..." names what is cut.

**Three names, three behaviours.**

- Waterline: closed rings at the stepover, joined by straight steps.
- Offset rings: offsets of the wall shape, spaced by the engagement law,
  joined as one outward spiral, the wall ring last. Round stretches hold
  the load; sharp corners are not re-planned, and their load is named.
- Adaptive: every cut planned on the in-process stock (morph spiral,
  grown rings, engagement fronts, trochoids). It holds the load in
  corners and channels, and it leaves the wall corners as a named
  scallop for the finish pass.

**Why.** The maintainer's report (#3933 comment, 2026-11-24): "should be a
constant engagement spiral that ends in a 'wall clearing' ring as a final
pass". The step from ring to ring was the full-width cut: a radial move
of one step into the material, read silently. Reuse of the proven morph
keeps the change small and inside the cutter-centre area by
construction.

**Dropped.**

- Route Offset rings through the HSM planner (`adaptive::plan`) and add a
  wall lap: the two strategies would be almost the same, and the
  difference between the names would go away.
- Round the ring corners so the load holds there too (Pateloup, Duc & Ray
  2004): a new geometry law. It is a follow-up; Adaptive already holds the
  load in corners.
- The 3-D ladder's Offset rings (`machining3d`, `level_ring_plan`): the
  issue is the 2D Pocket. The 3-D per-level plan keeps the rings one by
  one, so its digests do not move in this PR.
- Pass start Inside / Outside on an area with a start side (level-curve
  pieces, `level_pieces`): the pieces are open curves of one side, not
  nested closed rings, so a morph does not apply. #3752 moves these
  planners onto one `ring_set`; the spiral can follow there.

**Cost to change later.** The spiral is one module
(`machining2d/offset_spiral.rs`, with `offset_law.rs`) over `chains_at`. When #3752
replaces `waterline_passes` with `ring_set`, the module takes its loops
and chains from there; the morph, the guards and the read stay.
