## #3208 - Keep strict wire cover checks without arc provenance

**Decided.** The wire cover law (`crates/motion/src/wire_edm/laws.rs`)
restores the upstream strict `COVER_TOL` of 0.002 mm for all current
polygon profiles. Remove
the allowance that inferred meshed rounds from vertex coordinates.
An allowance requires reliable arc or mesh provenance. The current
`Loop` and wire generator inputs contain points, without that provenance.

**Why.** The review of PR #3224 found a sharp 16-sided R0.5 polygon
that passes every geometric guard of the inferred-round classifier.
Each corner turns 22.5 degrees, and its chord sag is about 0.0096 mm.
With a 0.162 mm kerf offset, the path leaves about 0.0032 mm at each
corner. This exceeds the 0.002 mm cover tolerance. The same points can
represent an authored polygon or a tessellated arc. Their coordinates
cannot establish which profile the user intended.

The original coarse-fillet acceptance remains deferred. A coarse mesh
without provenance must receive the same strict check as an authored
polygon. Fine facets that meet the existing cover tolerance can pass.

**Dropped.**

- Infer arc provenance from turn, radius, sag, or matching neighbours.
  The small drawn polygon satisfies these tests and remains unsafe.
- Increase `COVER_TOL`. This also accepts sharp corners with excessive
  material left standing.
- Add a caller-controlled allowance flag. A flag alone does not prove
  that the input came from a true arc or a model mesh.
- Add an end-to-end provenance API in this repair. The requested repair
  is limited to the review findings. Reliable provenance needs separate
  design and validation through the profile extraction path.

**Not changed.** The path remains the offset of the polygon chords.
The generator refuses a path that exceeds the existing cover tolerance.
No fixture, golden file, importer, or post processor changes are needed.

**Cost to change later.** A future allowance must carry reliable source
information through profile extraction to the cover check. Its tests
must distinguish authored polygons from supported source arcs or meshes.
