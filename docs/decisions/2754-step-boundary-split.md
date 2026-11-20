## #2754 - STEP: divide the boundary edges of faces curved two ways finer

**Decided:** the importer divides each shell edge before the tessellation
pass by the largest inside bound (`step_fit::Curved::inside`) of the faces
that it bounds: 1.5 for an edge of a cone, sphere or torus face, 1 for all
other edges. A cylinder face then gives the largest divisor of its edges to
all of its edges, until no edge changes (`step_fit::edge_divisors`). The
division happens once, in the shell's own edge polyline
(`CurveGuard::finer`), so the two faces of an edge get the same points and
the shell stays closed.

**Why:** #2731 made the report true: the kept height is the highest facet
edge times the inside bound (#2718). `fit` never splits a boundary edge,
and the kernel held the boundary edges only to the pass tolerance. Every
model with a cone, sphere or torus face thus read 1.5 x the tolerance
(fig8: 0.00749 mm), past `CHORD_CAP`, and became reduced fidelity. With a
boundary edge at tolerance / 1.5, the report is the tolerance again (fig8:
0.005 mm), and it stays true. The 1.5 bound and #2731 stay as they are.

The cylinder step: a cylinder meshes rim to rim. When only the rim next to
a cone is divided finer, the wall facets slant, and each slice height cuts
a different chord of the wall. The fig8 outline then had 70299 points in
place of about 2500, and the adaptive region and its order changed.

**Dropped:**
- A divisor with a margin (2.0 = 1.5 x 4/3). It gave the same chord, but
  the fig8 adaptive path then ended with an extra ring at the stock edge
  after the wall pass (`fig8_outside_containment_with_em0800_and_clearing_kinds_works`
  failed at "ends at selected face", dist 31.406). The bound with no margin
  is exact for a circle on its own surface. Where the curve's parameter
  midpoint and the face's differ, the report goes over the tolerance and
  says so; it is never under-stated.
- No cylinder step: the slanted wall facets above.
- Option A (report only the inside edges), C (raise `CHORD_CAP`), D
  (relax the test expectations) and a revert of #2731: each makes the report
  false again or weakens a fidelity guard.
- A split of the boundary edges per face, with a repair of the T-junctions
  on the neighbour face: more code, and a risk to the closed shell.

**Cost to change later:** the divisor is one function
(`edge_divisors`); a new curved kind adds one `Curved` arm. Note: the
adaptive path order on fig8 is sensitive to small mesh changes (the dropped
margin above); a later mesh change can show it again.
