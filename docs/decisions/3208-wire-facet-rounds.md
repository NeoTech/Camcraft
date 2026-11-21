## #3208 - Wire corner check accepts the facet mitre of a meshed round

**Decided.** The wire cover law (`crates/motion/src/wire_edm/laws.rs`)
gives an allowance at each vertex that is a facet of a meshed round. The
allowance is the exact mitre excess of the polygon offset at that vertex,
`k (1 / cos(θ/2) - 1)`, where `k` is the kerf offset and `θ` the turn.
The allowance fades linearly along each chord. `rounds.rs` finds the
facets. A vertex is a facet when all of these are true:

- it turns 30° or less;
- the circle through it and its two neighbours has a radius of `k` or
  more, and the chord stands 0.02 mm or less off that circle;
- a neighbour vertex is a facet of the same circle (2 % of the radius
  plus 1 µm), with a turn of the same sign.

**Why.** A model's mesh draws an r0.25 fillet in four 22.5° chords. The
path is the polygon offset of the chords, so it stands 3.2 µm too far at
each facet vertex for a 0.162 mm offset. `COVER_TOL` (2 µm) absorbs only
15° facets. The true round is legal, so the refusal was wrong. The test
reads only the mesh, so it must recognise a round from its vertices. The
linear fade is safe: the excess along a chord is convex and falls to
zero within `k tan(θ/2)` of the vertex, which is shorter than the chord
for each radius of `k` or more.

**Dropped.**

- A larger `COVER_TOL`. It also lets a sharp kink or a radius a little
  under the minimum pass. The tolerance stays 2 µm.
- Finer tessellation of the profile in the app. The silhouette comes
  from the model's mesh, and the true arc is not available there.
- Rebuilding the arcs and offsetting them as true arcs (an arc-fit
  offset). It is new geometry for each wire path, at a much larger cost.

**Not changed.** The path is still the offset of the chords. Against the
true arc it leaves the chord sag (about 5 µm for r0.25 in 22.5° chords)
standing at mid-chord. This was already true for every meshed round. The
mesh resolution sets it, not this law.

**Cost to change later.** The two limits (30°, 0.02 mm) are constants in
`rounds.rs`. A finer arc-fit offset can replace the allowance without a
change to the callers.
