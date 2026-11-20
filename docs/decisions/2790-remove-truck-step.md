## #2790 - Remove truck-stepio, ruststep and truck-topology; the STEP bench reads recorded truck verdicts

Part 3 of #917. It completes #917 with the scope below.

### Scope (the maintainer, 2026-11-20)

**Decided.** This PR removes three crates from the workspace:

- `truck-stepio` (the STEP reader of truck),
- `ruststep` (with its `[patch.crates-io]` git pin of #887 and the
  rationale text in the root `Cargo.toml`),
- `truck-topology` as a direct dependency (its only direct user was the
  bench's face meter, `crates/kernel/tests/step_util/truck_guard.rs`).

The non-STEP `TruckKernel` (`crates/kernel/src/truck_impl.rs`, the `truck`
feature, the tests `agreement.rs`, `blend.rs`, `heal_import.rs`) and the
truck crates that it needs stay. Their removal is #2922 (Part of #27).

**Consequence.** The first "Done means" item of #917 ("no `truck-*`
package in `Cargo.lock`") is met for the STEP crates only. Nine `truck-*`
packages stay in the lock for `TruckKernel`: `truck-base`,
`truck-derivers`, `truck-geometry`, `truck-geotrait`, `truck-meshalgo`,
`truck-modeling`, `truck-polymesh`, `truck-shapeops`, `truck-topology`.
`truck-topology` stays as a transitive package of `truck-meshalgo`,
`truck-modeling` and `truck-shapeops`; no member names it.

**Dropped.** Removing `TruckKernel` in this PR: the maintainer put it in
#2922. Keeping `ruststep` for the #1075 nesting scan: the scan only mirrors
ruststep's tokenizer rules in a table; it never calls ruststep.

### The truck side of the STEP bench: recorded readings, not deletion

**Chosen.** Each test of the bench stays. The truck side of each row is now
a recorded reading in `crates/kernel/tests/step_agreement/rows.rs`
(`Truck::Read { shells, volume, area, min, max, open_edges, chord_mm }` or
`Truck::Refused(reason)`). The live solidcore reading is judged against
the recorded truck numbers by the same rules and tolerances as before
(`verdict.rs`: `REL_TOL` 1e-3, `BOX_CHORDS` 2). Every pinned verdict
string is the same as before.

**Where the numbers come from.** The bench's own printout, one run of
`cargo test -p camcraft-kernel --test step_agreement -- --nocapture
--test-threads=1` at commit `4b694c7c7` (the head of PR #2918, before this
change), with truck-stepio 0.3.0, truck-meshalgo 0.4.0 and ruststep 0.4.0
at the #887 git pin. Volume and area are recorded at 6 decimals, the box
at full `{:?}` precision, the chord at 4 decimals. The fillet_corner and
bar_cross_holes numbers agree with `917-kernel-pcurves.md` (20651.68 and
45816.21 mm3; 121 and 1090 open edges).

**Why.** A recorded reading keeps the independent check of solidcore's
volume, area, box, solid count and unit against a second reader. A
deletion of the truck side keeps only solidcore's own numbers, and a
change of those numbers would then pass where the truck reading would
catch it.

**Dropped.** Deleting the truck side: it loses the second reader. Keeping
truck-stepio as a dev-dependency: the maintainer wants the crates out of
the workspace and the lock.

**Cost to change later.** A new STEP fixture needs a row; its truck side
can no longer be measured. Write `Truck::Refused("not recorded (added
after #2790)")` and pin the import verdict that results ("both refuse" or
"truck refuses"), or compare against a closed form in
`step_closed_form.rs`.

### The other truck users of the bench

- `step_import.rs` `corrupt_step_files_never_panic_or_hang`: the third
  column of `KNOWN_STEP_VERDICTS` (truck's verdict per corpus file) stays
  as a record and is no longer asserted. The solidcore column is asserted
  as before.
- `step_import.rs` `truck_side_refuses_a_runaway_face_and_obeys_its_cancel`
  tested the bench's own truck reader and its face meter (#1697), which
  this PR removes. It is replaced by
  `step_agreement.rs` `recorded_truck_readings_keep_their_findings`, which
  checks the recorded table: the runaway face of bspline_discontinuous is
  a recorded refusal by name, the smooth dome is a recorded mesh, the open
  meshes of fillet_corner and bar_cross_holes and the zero volume of
  twisted_flank stay recorded, every recorded mesh is finite with a sane
  box and chord, and the truck side agrees with the pinned verdict. The
  cancel half of the old test has no equivalent: the reader that it
  cancelled is gone.
- `mesh_deviation_bench.rs` (release lane, ignored): the truck column was
  printed and never asserted. It is dropped. The recorded numbers of
  `917-kernel-pcurves.md` are for two files that are not rows of this
  bench, so they cannot replace it. The test and its solidcore asserts
  stay.

### Split of step_agreement.rs (#2800)

`step_agreement.rs` had 498 lines. It is split along its domain seam: the
pinned table with the recorded readings (`step_agreement/rows.rs`), the
verdict rules (`step_agreement/verdict.rs`) and the reading and the tests
(`step_agreement.rs`). The two modules are loaded with `#[path]`, so cargo
does not see them as test targets of their own. Each `#[test]` stays.

### `#[test]` count

`cargo test -p camcraft-kernel --features truck --tests -- --list` plus
`--lib -- --list`: the count is the same before and after (see the PR body
for the numbers). Per binary: `step_import` goes from 4 to 3 and
`step_agreement` from 22 to 23 (the moved test above).

The release tier (heavy lanes) did not run for this PR. The cut gate runs it.
