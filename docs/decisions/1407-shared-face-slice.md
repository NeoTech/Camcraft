## #1407 - a soup of closed solids that meet a plane 4-way slices as their union (2026-11-13)

**What was decided.** `slice::slice` keeps its plain read for every mesh
that slices today. Only when that read refuses with `NonManifold` (chord
ends met 3+ ways) does it retry: the facets are split into bodies by
shared-edge connectivity (corners welded within 0.1 um by spatial hashing),
and if EVERY body is a closed, consistently wound 2-manifold (every
undirected edge used once in each direction) with positive signed volume,
each body is sliced on its own and the sections are united by the 2D
boolean union. Anything else keeps the refusal. The refusal is reworded to
say what is wrong with the model and what to change, and the generators
map `Open` and `NonManifold` slice faults to a new `GenError::Model`
("the source model cannot be sliced: ...") instead of `GenError::Invalid`
("invalid machining parameters: ...").

**Why.** The laid-flat sample is three closed solids; two are stacked so
their cylindrical walls overlap in Z on the same vertical lines (both walls
carry the same seam vertices). Every plane in that band meets both outlines
at 93 shared points, four chords each. The chords there run the same way
(both bodies' material on the same side: the solids overlap in that band),
so a "cancel opposite coincident chords" weld does not apply - the chords
are not even coincident, the two bodies subdivide the polygon edges
differently. The section of a union of solids is the union of their
sections (regularized set operations, Requicha, ACM Computing Surveys
12(4), 1980), which is the law `material_above_of` already applies to
bodies picked from separate entities (M13.11). Connectivity is a safe
source of body identity here only because each body must prove itself a
closed outward solid first.

**Safety guards (each pinned, each mutation-checked):**
- the closed/consistent edge test - two solids WELDED along a shared edge
  or face share it four ways, cannot be told apart, and stay one
  non-manifold shell: refused (M13.11's reason for not using connectivity
  is kept as a refusal, not overridden). An open body is refused the same way.
- the outward (positive volume) test - an inward cavity shell sliced alone
  reads as material, and the union would fill the cavity: refused.
- the 0.1 um corner weld - nanometre seam drift of imported STEP shells
  must not split one body into open pieces.

**Considered and dropped.**
- Cancelling coincident oppositely-oriented chords (the B-rep "glue" of
  two solids across a shared face): correct for solids butted face to face
  with identical tessellation, but this model's chords are same-direction
  and differently subdivided; it would not fix the reported case.
- Body ranges from the importer (one range per `MANIFOLD_SOLID_BREP`): the
  facet soup carries no body ranges today, and adding them changes the
  project file and every `Material` consumer - larger than this fix, and
  the connectivity proof gives the same bodies for this model.
- Always slicing per body (not only on refusal): would re-normalize every
  multi-shell mesh's regions and move goldens for no gain; the fallback
  leaves every mesh that sliced before byte-identical.
- Keeping `GenError::Invalid` and only rewording: the prefix "invalid
  machining parameters" is what misled the reporter.

**Cost to change later.** Importer body ranges could replace the
connectivity split in `slice_bodies::closed_solids` without touching any
caller; `GenError::Model` is matched nowhere by name except the two
`from_slice` maps.

Provenance: Requicha 1980 (regularized union, sections); Botsch et al.,
"Polygon Mesh Processing", A K Peters 2010, ch. 1 (closed oriented
2-manifold test); Zhang & Chen, ICIP 2001 (signed volume by the divergence
theorem); Teschner et al., VMV 2003 (spatial-hash welding). No vendor
material consulted.
