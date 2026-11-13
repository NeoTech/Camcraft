## #1057 - A declared rim is read at the plain gouge tolerance on a reduced-fidelity model

**The threshold, explained.** A Z drill, bore or thread mill verifies in
`Mode::Boundary` against a region whose rim loops are our own polygons of
each hole's declared circle (`material_source::rim_loop`). Those polygons
are built to `RIM_SAG_BUDGET` = 0.005 mm - half the 0.01 mm `GOUGE_TOL` -
with at least 48 sides, so a 2 mm-radius rim stands 2 (1 - cos(pi/48)) =
0.00428 mm inside its circle at each edge's middle. A matched drill's
centre therefore reads `r - 0.00428` off the rim. #864 made law (b) read
every loop at `GOUGE_TOL - excess` (excess = the model's chord error less
the importer's 0.005 mm promise). Once the excess passed 0.01 - 0.00428 =
0.00572 mm - a model flagged above 0.0107 mm - the tolerance left no
longer covered the rim's own sagitta and the matched drill was convicted
at its first plunge (`Gouge { at: 1 }`). A matched bore was convicted on
its rings the same way. Past 0.015 mm the excess uses the whole tolerance,
`excess_floor` reads untightened and the program gate refuses by name,
which is why 0.03 mm read clean.

**What was decided.** `VerifyContext` carries `rims: &[(Point2, f64)]`,
the declared circles of a containment kind (filled in `generate` from the
Drill (Z holes) and Bore params when the depth law is `Contain`,
empty everywhere else). Law (b) - both the law-point reading and the
Boundary whole-move wall reading - reads a region loop that traces one of
those circles (every vertex within 2 um of it: the 1 um lattice snap) at
the plain `gouge_tol`; every other loop, including the model's own
silhouette loops beside the rims, stays at `tightened(gouge_tol)`. The
solid law that sweeps the tool against the model's facets
(`contain_into`) is untouched and still tightened.

**Why.** The excess measures how far a coarse model's facets stand inside
its true surface. A rim is not a facet: it is drawn by us from the hole's
centre and radius, to a sag budget we chose, and a coarse model does not
make it any less accurate. A hole's radius fitted from a coarse mesh is
fitted to vertices on the true surface (or smaller), never larger, so the
declared circle is never wider than the real hole because of the excess.
Reading the rim at the plain tolerance is exactly today's full-fidelity
verdict on it - nothing a full-fidelity model convicts is acquitted - and
the real gouges on a coarse model still convict through the laws that
read the model: an oversize drill against its rim, a drill or bore offset
into the part's wall (the solid law), and a drill past the floor (the
solid law, tightened: 0.005 mm past is clean at full fidelity and a gouge
on a model flagged 0.0149 mm).

**Considered and dropped.**
- *Build the rims finer as the excess grows.* The tolerance left goes to
  zero at the bound, so no finite side count reaches "anywhere below the
  bound", and a 4096-gon rim makes law (b) slow.
- *Read rims on their true circle (exact distance) at the tightened
  tolerance.* Stricter than today at full fidelity in the wrong direction
  (it would remove the sagitta our own polygon adds at full fidelity too,
  changing settled verdicts), more code in the arc-wall reading, and it
  still applies a model property to a non-model loop.
- *Pass `model_excess = 0` to the containment kinds' verify.* Loosens the
  model-derived silhouette loops and the reach and rapid laws, which do
  read the model.
- *Recognise rims structurally in `sim` (a regular polygon on a circle).*
  A coarse model's own hole silhouette is such a polygon; only the
  operation knows which circles it declared.

**Left open: Thread declares no rim** (#1073 review). Since #1071 the
thread's region reads the major circle (`thread_rims::major_rims`), not the
declared minor rim, so a minor declaration traced no loop and the thread
mill stayed tightened anyway. Thread is therefore left out of `rims_of`: a
thread mill on a coarse model keeps the tightened reading, the conservative
side - it may convict near the refusal bound where full fidelity is clean.
Declaring the major circle would need the tool's pitch and the region's
measured minor (`major_rims` shrinks it to a flat model rim); that is a
follow-up, not this fix. Pinned by
`coarse_rims::a_thread_mill_on_a_coarse_model_is_never_read_looser_than_at_full_fidelity`.

**Cost to change later.** One field with an empty default; dropping it
restores the old reading.
