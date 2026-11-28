## #4010 - Read the star cell through the offset's rounding noise

**Decided:** `adaptive::star::Star::new` drops each vertex of an opened row
loop that stands less than 1e-5 mm from the last vertex kept (`denoised`).
The star laws (`disc_in_kernel`, `radial`) then read the cleaned loop.
The bore's `BORE_PIN` moves to `(1642, 1614, 300)`, measured on macOS arm64
in release. The fallback band is centred on it with the same half-widths.

**Why:** The Z 15 and Z 5 levels of the user's bore hand the planner a
328-vertex disc outline. Its opening at the corner radius (R2.5) keeps 20
edges that are one or two Clipper units long (1.4e-6 to 2.2e-6 mm). Each
turns back about the pole by integer rounding. `radial` reads such an edge
as a shadow. `disc_in_kernel` reads its line, in a random direction, as a
wall through the area. So the star cell was refused, and the growth rings
planned the disc: 999.92 mm of feed against 334.31 mm. The 128-vertex
outline of Z 10 and Z 0 has no such edges, so those levels kept the morph
spiral. The mesh, not the planner, chose which levels lost. Before the
PR #3802 mesh, Z 10 took the long plan; that reading was not probed
again, but it is the same pair of plans. With the fix each level
takes the morph spiral. Z 15 and Z 5 cut 330.07 mm, reach the wall at
7.498 mm in -Y, and verify clean.

A 1e-5 mm limit is 10 Clipper units. Noise edges are at most a few units
long. A real edge of 1e-5 mm or more keeps its direction within the
rounding, so it cannot turn back. The loop moves at most 1e-5 mm, which is
far below the plan's sagitta and the verifier's tolerance. The plan is
still proved after this: the arc proof, the crossing law and the verifier
read the posted walk.

**Considered and dropped:**
- A note when a level plan is much longer than needed (the issue's second
  option). It would name the symptom and keep the 3x feed length. The cause
  is a false refusal, so the fix removes it.
- A tolerance in `radial`'s grazing test or in `disc_in_kernel`. A
  tolerance there accepts a real small shadow too. The noise is in the
  vertices, so the vertices are cleaned.
- `Region::offset` cleaning its output in `camcraft-geometry`. That
  changes every offset in all operations and their goldens. The star cell
  is the one reader that is this sensitive to the direction of a tiny edge.

**Cost to change later:** Low. `denoised` is one function in `star.rs`,
pinned by `the_users_z15_bore_is_one_star_cell` and
`the_users_z15_bore_plans_the_short_spiral` on the recorded outline
`crates/motion/tests/fixtures/bore_z15_center.txt`. Windows and Linux
equalled macOS before the fix and are not measured after it. Their next run
confirms `BORE_PIN` or adds a platform triple.
