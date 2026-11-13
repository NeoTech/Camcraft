## #1530 - a flipped setup's fixtures are judged by their shape against the carried stock

**Decided.** The carried-workholding clause (`fixture_carried`) keeps the
fixture's world box as the broad phase and, for a fixture the box flags,
reads its closed world mesh column by column: the Z spans the mesh puts
on the column's vertical line (`FixtureMesh::steel_on_line`, new in
`sim::fixture_mesh::column`) against the column's carried material.
A column convicts when that overlap exceeds `CARRIED_CONTACT_TOL` =
0.01 mm on the centre line AND on four lines 0.01 mm beside it (so a
face lying on a column centre is contact). The mesh only acquits: an
open or refused mesh, a coarse-fidelity fixture (#864), a line no nudge
reads, a spent visit budget (`MESH_VISIT_BUDGET`, 50 M facet visits per
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
convicting column. A 400 k-facet fan (every line meets ~25 k facets)
that the mesh would acquit took 3.6 s unbounded in release; with the
budget it stops at 0.39 s (5.8 s debug) and keeps the box verdict,
logged. Lateral resolution is the carried grid's pitch, as before.
