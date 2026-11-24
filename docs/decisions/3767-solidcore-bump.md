## #3767 - solidcore fork synced to upstream main, pinned at fa67b3f5

**What was decided.** The fork NeoTech/solidcore `main` was 0 commits ahead
and 300 commits behind tangency-labs/solidcore `main`. It took the upstream
commits with GitHub's merge-upstream call, which was a fast-forward (no
force-push, no merge commit needed). `crates/kernel/Cargo.toml` pins all six
solidcore crates at the full rev `fa67b3f560b7a8d59e99cb728f72f85e91dee2da`
(was `2016386442dfee3b2a0c3a1bfdd4017c5254dac8`). `Cargo.lock` changed only
for the six solidcore packages. The workspace line `truck-polymesh = "=0.6.0"`
is removed (#3742): no crate manifest uses it. `truck-polymesh` stays in
`Cargo.lock` as a transitive dependency of `truck-meshalgo`.

**Why.** The maintainer asked for the update (2026-11-23). A fast-forward
keeps the fork's history equal to upstream, and a full rev keeps the build
reproducible.

**Moved pins, re-triaged to the new solidcore.** Each file is still
refused or still measured exactly; no assert was widened.

- `twisted_flank_off.step` (kernel `step_agreement` row, kernel, geometry
  and app `step_twisted_flank`): still refused as `#117`. The reader now
  builds the pcurves, and the vertex check refuses the solid ("a vertex
  sits 3.430e-1 mm off its edges (limit 1e-3)"). The refusal no longer
  names face #93; the app pin drops that one assert.
- `edge_gap_021_stated_5mm.step` (kernel and geometry `step_edge_gap`): was
  kept by the reader and refused by the mesh at the chord. Now refused at
  the read by the kernel's gap check, with the 0.02 mm cap and the stated
  5 mm named. An earlier refusal of the same file.
- `cube-chamfer-all-12-d2` (kernel `blend`): volume 802.667 -> 808.000.
  Upstream #21269 made the chamfer-corner event the only chamfer corner:
  three equal chamfers meet in one mitred vertex, with no flat bevel
  triangle. The closed form drops the d^3/12 tetrahedron per corner and
  matches the new solid exactly.
- `provoke_face_left_out_edge_not_sampled_and_lenient` case 3 (kernel
  `machining_mesh_shortfalls`): the self-crossing inner loop now gives
  `NotWatertight { open: 6 }` in place of `Lenient`. Still a refusal. A new
  provocation for the `Lenient` arm is #3800.

**One code change: the STEP syntax locator.** The new Part 21 parser holds
a text's counts to a density of the text's own length (one parameter for
each 4 bytes). `camcraft_geometry::step_diag::record_parses` probes each
DATA record alone in a short frame, so a dense, legal record (the
`step_nesting` record 64 lists deep, or a long list of `0,`) passed that
density and the locator refused the whole import as a syntax error. The
probe now reads `TooManyParams`, `TooManyRecords` and `TooManyEntities` as
"no syntax fault in this record". The parse of the whole file still
applies the limits to the file. Dropped: padding the probe to 4 times the
record (costs memory on a large B-spline record, same result). Pinned by
`step_diag::a_dense_legal_record_is_not_named` and the two `step_nesting`
pins that went red.

**Considered and dropped.** A merge commit on the fork: not necessary, the
fork had no commits of its own. A branch reference in place of a rev: the
issue asks for a full rev. A tag on the fork for the pin (as the older
`camcraft-pin-2026-11-11`): not necessary, the rev is on `main`.

**Cost to change later.** One rev string in six lines and `cargo update -p`
for the six solidcore packages.
