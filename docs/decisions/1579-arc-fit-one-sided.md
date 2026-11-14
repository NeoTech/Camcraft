## #1579 - one-sided arc fit on contour walls

**Chosen.** A contour walk that knows its part side (closed rings, open
chains and the lead reseat; `air_left` = `climb`) fits arcs one-sided: the
circle stays through the run's two end vertices, and its centre slides
along their bisector until no interior vertex is on the part side. The
offset nearest the two-sided circle is taken. A closing ring keeps its
circle only when every vertex is already on the air side; otherwise the
greedy fit keeps the longest open run. The two-sided fit
(`fit_arcs`) is unchanged for every other caller.

**Why.** The two-sided fit let a vertex stand half the operation
tolerance (0.05 at 0.1) off the circle in either direction. Toward the
part that is a cut past a verified point, and the verifier forgives only
`GOUGE_TOL` (0.01), whatever the operation tolerance. So a coarse fit on
a tessellated round wall posted arcs that verify then held as gouges,
and the result depended on the walk direction.

**Considered and dropped.**
- Cap the vertex share at `ARC_FIT_TOL` (0.005): the Ø30 cylinder wall
  then fit no arcs at all at tolerance 0.1 - the offset wobble of the
  facets is wider than that.
- Prove each arc against the legal region after the fit (as the adaptive
  plan's `arc_proof` does) and drop the refused ones: it keeps chords
  where a slightly larger circle is safe, and costs a region test per arc.
- Raise the verifier's allowance to the operation tolerance: that hides
  a real cut past the wall, against the Prime Directive.

**Not changed.** The compensation walk (`contour_comp`) and the pocket,
ring and waterline fits stay two-sided; their sag laws are their own.
