## #1530 - a flipped setup's fixtures are judged by their shape against the carried stock

**Decided.** The carried-workholding clause (`fixture_carried`) keeps the
fixture's world box as the broad phase and, for a fixture the box flags,
reads its closed world mesh column by column: the Z spans the mesh puts
on the column's vertical line (`FixtureMesh::steel_on_line`, new in
`sim::fixture_mesh::column`) against the column's carried material.
A column convicts when that overlap exceeds `CARRIED_CONTACT_TOL` =
0.01 mm on the centre line AND on four lines 0.01 mm beside it (so a
face lying on a column centre is contact). Between the columns the mesh
is read over whole rectangles, not lines (review rework, below). The mesh only acquits: an
open or refused mesh, a coarse-fidelity fixture (#864), a line no nudge
reads or that winds inconsistently, a rectangle it cannot read, a spent visit budget (`MESH_VISIT_BUDGET`, 50 M facet visits per
fixture) and a footprint narrower than a cell keep the box depth.

**Why.** A vise's box spans the part it grips and a nest pocket lies
inside its jaws' box, so both were refused as "standing inside" the
carried part (4.48 / 11.43 mm deep in the report). The real steel is
what the machine holds the part with.

**Tolerance.** 10 microns: below any fit a jaw or nest is modelled to,
far above the float dust of a carried body (the old box law kept
`FIXTURE_HIT_TOL`, 1e-9 mm, which still governs the box fallback). A
gripping face or a seated floor is exact contact in a model; 0.01 mm of
modelled interference is still contact, 0.03 mm is refused.

**Considered and dropped.** A full mesh-vs-dexel volume intersection
(new engine, no gain at the carried grid's pitch); per-body splitting
of multi-body fixtures (the line reading handles any number of shells,
overlapping or touching); reusing `FixtureMesh::contains` per sample
point (many points per column for the same answer, no depth).

**Cost / bounds.** Lines are read only where the box convicts, and a
column whose box depth cannot deepen the worst depth already found is
skipped, so a fixture that truly penetrates stops after its first
convicting column. Re-measured after the review rework (release,
M-series Mac): a 400 k-facet fixture (a 100 k-segment cylinder whose cap
fans every line and rectangle near the centre meets) takes 16.6 s
(clears) / 19.4 s (penetrates) with no limit; with the 50 M-visit budget
both stop at 0.34 s and keep the box verdict, logged. Inside the material nothing is sampled (rectangles are read whole); at a side face or step the resolution is the carried grid's pitch, as before.

### Review rework (M3 review on 279ac89f)

**Decided: the mesh reads the material between the columns whole, not
on lines.** Besides the per-column lines, every rectangle spanned by
four neighbouring material columns, trimmed by the 0.01 mm contact
tolerance, is judged against the material all four share (highest
floor to lowest top) by `FixtureMesh::steel_in_prism`
(`sim::fixture_mesh::prism`). The rule and why it cannot miss steel:
- Clip every facet to the rectangle (Sutherland-Hodgman). If a clipped
  facet's Z range reaches into the material slab trimmed by the
  tolerance, that facet has a point in the slab: steel stands in the
  material, refused.
- If no facet meets the rectangle, the fixture has no steel over it (any
  steel point's vertical line leaves the bounded solid through a facet
  at the same XY).
- Otherwise the slab holds no surface point, so it is wholly inside or
  wholly outside the solid, and one line through its middle decides.
No step samples a line, so a feature of any size inside the rectangle
is seen: M3's 0.19 x 0.19 post (3 mm) and 0.19 x 50 lip (4 mm) are
refused at pitch 0.375. Cost: a BVH walk per rectangle the box flags,
on the same facet-visit budget.

**Why "what all four share", not the cell's own material.** The carried
body is known at column centres only, so a side face of the part lies
somewhere between two centres. Judging a cell's whole area against its
own column's material refuses every vise jaw whose face is not on a
grid line (the cell holding the part's edge reaches past it). Where the
four corners disagree, only what they share is material there: steel
within one pitch of a side face or a step, short of the next centre,
passes - the same resolution the box law has always had at a face. It
is the only steel this reading lets through, and it lies within a cell
of a material boundary, never inside the material.

**Considered and dropped.** A per-cell Z envelope against the cell's
own column (refuses gripping jaws, as above); falling back to the box
wherever a mesh vertex or edge lies off the sampled lines (a jaw face
always does: same over-refusal); more lines per cell (any finite set
misses a thinner feature).

**Inconsistent winding keeps the box.** A line whose winding goes
negative, or does not close at zero, reads `None`: an inside-out shell
overlapping another (one flipped body in an imported STL) no longer
reads its steel as air. A flipped shell wholly inside another still
reads as a cavity - along a line it is indistinguishable from a real
one. With the guard, nonzero winding and odd parity agree, so the old
parity clause is gone.

**Nudge capped.** The nudge step is a millionth of the mesh's scale,
now capped at 0.1 micron (0.56 micron for the farthest nudge), so a
line far from the origin is not moved off a feature by its
coordinates (19 microns at X 5000 before).
