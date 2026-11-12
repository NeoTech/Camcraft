## #1217 - The hole-wall excuse credits tessellation only

**Decided.** `swept::chord_dip`, the containment kinds' hole-wall
excuse, still computes a facet's chord dip into the swept disc
(`rho - sqrt(rho^2 - l^2/4)`), but credits at most `HOLE_CHORD_CREDIT`
= the program sim's `DEV_TOL` (0.01 mm). A dip past it is a polygonal
hole's flat - real material - and a cut into it is a Gouge.

**Why.** From the mesh alone an octagon with its corners on r 5 is
exactly what an 8-segment tessellation of an r 5 circle looks like, so
"the vertices lie on a circle" cannot tell a flat from tessellation;
only the size of the dip can. A round wall meshed within the mesh
tolerance dips at most that much, so capping there keeps every honest
round hole clean (the STEP importer meshes at half of it; the goldens'
32/128-gon holes and a 96-gon r 5 hole stay clean) while every coarse
polygon's flats are judged. Same tolerance #1099 chose for its thread
grant (`CHORD_CREDIT`), so the two excuses agree.

**Considered and dropped.**
- Checking the facet's vertices against the region's declared rim
  circle: the solid law gets no rim (only `hole_walls: bool`), and a
  coarse polygon's corners lie on that circle too - it would not
  convict the issue's octagon.
- Dropping the excuse: false gouges on every round hole whose tool is
  matched to it.

**Cost to change.** One constant. A coarse STL of a round hole (dip >
0.01 mm) cut to size now convicts - the honest answer for a mesh that
coarse; a looser credit would need a declared mesh tolerance per part.
