# #1602 - thread strips mesh at full fidelity: per-direction division, a stretched parameter plane, a second parameter search

Context: two of the user's AP214 parts each carry three `B_SPLINE_SURFACE`
faces that are a thread's flanks and root: one rational strip, degree 1
across, degree 2 along in 36-degree arcs, about fourteen turns long. truck
meshed each into about 2 M facets (6.1 M and 5.0 M a part; 31-130 s in
release), with a 0.5-3 mm hole along the flank that PR #1576's closed-shell
check refuses. Three causes, each fixed in the importer's wrapper around the
face (`step_guard::Metered`), none in truck.

## 1. B-spline and NURBS faces divide per direction (`step_division`)

- **Chosen:** our own tensor-grid division. Per cell, one jittered interior
  point is tested against the chord of its iso-v curve (splits u), of its
  iso-u curve (splits v), and, only when both hold, against truck's own
  bilinear patch (a twist: splits both). Every cell of the result passes
  truck's acceptance test.
- **Why:** truck's `parameter_division` flags both intervals of a failing
  cell, so a strip curved only along v gets 2^sweeps intervals across u.
  That was the facet runaway: 523 K facets for a four-turn synthetic flank,
  1,024 with this division.
- **A jump still refuses.** A division still splitting after 64 sweeps
  spends the face's whole evaluation budget, so the pass is refused by name
  as before (#1536). Without this a jump split only across itself grows
  one column a sweep and would come back as a grid with a wall in it.
- **Dropped:** a larger evaluation cap or facet budget (#1568, #1599): they
  bound the cost and leave the cause. A fixed cap on u intervals: has no
  basis in the geometry. Patching or forking truck: a dependency we do not
  own.

## 2. The mesher's parameter plane is stretched (`step_stretch`)

- **Chosen:** an affine reparametrisation of a B-spline or NURBS face. The
  mean speeds along u and v are sampled (16 x 16), and the faster direction
  is multiplied by their ratio, so a parameter unit is about as long on the
  face either way. A face within 4:1 is left as it was, as is one past
  1e6:1 (a sliver whose sample is not trusted).
- **Why:** truck's trimmed-face triangulation is a constrained Delaunay
  triangulation in (u, v), and its empty-circle rule assumes the two
  directions are comparable. A thread flank is 350 times shorter in u than
  in v. Once the division stopped adding u lines, nothing constrained the
  plane next to the boundary, and Delaunay joined boundary points to the
  strip's far ends: triangles a quarter of the face long.
- **Dropped:** forcing interior grid lines (the region next to the
  boundary stays free and still fans out). Writing our own trimmed-face
  triangulator (effort:max work, not needed once the plane is isotropic).
- **Threshold 4:** every testdata model and sc_logo.step keep their
  facet counts except sc_logo (36,728 -> 34,496 from the division, the same
  chord). A lower threshold would re-mesh faces that were already sound.

## 3. A second parameter search for boundary points (`step_nearest`)

- **Chosen:** an answer from truck's parameter search that is off the
  surface (farther than truck's 1e-6 acceptance) or outside the knot domain
  is compared with Newton started from the 8 nearest samples of a grid with
  up to 4 samples in every knot span (at most 4,096 evaluations; section 4). The
  nearest answer inside the domain wins when it is on the surface or nearer
  than truck's. A pole is settled by `step_pole` first (#1548), so its
  answer is inside the domain and kept.
- **Why:** truck's no-hint presearch is a 51 x 51 grid over the whole
  domain - about three samples a turn on a 14-turn strip - and Newton then
  settles on the neighbouring turn, 1.7 mm off (the hole). On a root band
  the straight rulings, extrapolated, meet the band's other turns, and
  Newton finds the point exactly at u = 7 on [0, 1].
- **Dropped:** running the global search for every boundary point (cost;
  truck's local answer is right almost always). Raising truck's presearch
  density (a crate constant we cannot set).

## 4. What the second search refuses (review of PR #1638, items 1-3)

Taken unattended in the takeover of the review items; each takes the option
that refuses more.

- **The sample grid is capped by a refusal, not by striding.** A face with
  more knot spans than `MAX_SAMPLES` (4,096, the bound the search already
  named) covers at one sample a span gets no grid: when one of its boundary
  points needs the second search, the face is refused by name ("more knot
  spans than the search may sample"). No new number was chosen.
  **Dropped:** striding the knots (spans go unsampled, which is the miss
  the search exists to prevent, and nothing would say so); skipping the
  search and keeping truck's answer (the unchecked answer is what made the
  hole).
  **Cost, owed a reading:** a dense fitted surface (more than about
  64 x 64 spans) whose edge curves lie more than 1e-6 off it imported
  before this PR and is now refused. Not measured on real files; if it
  bites, the bound or the trigger distance is the maintainer's to move.
- **The second search is charged to the face's evaluation budget (#1536)
  before it runs:** 1 + 4,096 + 8 x 101 = 4,905 evaluations a search, the
  most it can take (the grid, 8 Newton runs of 100 trials, the checks),
  whatever it actually took. About 30,000 searches a face a pass fit in
  the 150 M cap; past it the face trips and is refused as any runaway. A
  face already refused in the pass runs no further search.
  **Dropped:** counting the evaluations truck's Newton really makes (its
  search runs on the bare surface; wrapping it would meter truck's own
  first search too, which main never did); a model-scale trigger distance
  instead (loosens when the search runs - not ours to choose here).
- **An answer outside the knot domain is never passed on.** When no search
  finds the point inside the domain and truck's answer is outside it, the
  search answers nothing and the face is refused by name ("a boundary point
  lies off the surface, outside its knot domain"). An answer INSIDE the
  domain that no search betters is kept as truck gave it, as on main.
  **Dropped:** a logged warning with the answer kept (the mesh would still
  carry the hole); clamping into the domain (a silent move of a boundary
  point).
  **Cost, owed a reading:** a face whose boundary runs past its surface's
  edge by more than a millionth of the knot span imported before (truck
  extrapolates) and is now refused. The fixtures of the eight importer
  suites still import.
- A faulted face is never "continuous": no coarser pass's mesh is kept for
  it (`step_refine`), since the coarser pass searched the same boundary.
- **Sheet-jump bound (review item 4).** When truck's answer is inside the
  knot domain, it followed the previous boundary point, so it is kept
  unless it is more than `JUMP` = 1e-3 model units (1 um in a millimetre
  file) off the point AND the second search's answer is at least 10 times
  nearer. A fold of the surface within 1e-3 of the point never moves a
  boundary point; a thread's wrong turn (one pitch off, against an edge
  error of microns) still does. An answer outside the domain is replaced
  as before, by any in-domain answer on the surface or nearer.
  **Dropped:** the reviewer's narrower option, a second answer only when
  truck's answer is outside the domain (it gives up the in-domain wrong
  turn that `a_nearer_answer_replaces_a_far_one_off_the_surface` pins); a
  bound on the chord tolerance (truck does not pass the tolerance to the
  search); a ratio alone (a point
  exactly on the other sheet would always jump).
  **Cost, owed a reading:** `JUMP` is in model units, so an inch file's
  bound is 25 um and a metre file's 1 mm. Not measured on real files.

## Readings (release, local; the user's files stay local)

| File | Before (upstream main) | After |
|---|---|---|
| 10208.STEP | 6,095,454 facets, chord 0.0157 mm (reduced), 31-72 s; open under #1576's check | 13,662 facets, chord 0.005 mm, 1.9 s; closed under #1576's check |
| 32050.STEP | 4,970,604 facets, 62-130 s; open under #1576's check | 8,128 facets, chord 0.005 mm, 1.8 s; closed under #1576's check |
| sc_logo.step | 36,728 facets, 0.19 s | 34,496 facets, 0.19 s |
| testdata/models/*.step | - | facet counts unchanged |
