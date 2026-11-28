## #2578 - A Solidcraft release fixture with fillets, compared with its STEP export

**What was decided.**
- The release check of #2578 uses a real design, not a synthetic body:
  `testdata/models/filleted_block.sc`, saved by Solidcraft 0.48.1, and
  `filleted_block.step`, the same design's STEP export from the same
  release.
- The design is a 40 x 30 x 20 mm block with a constant 3 mm round on one
  top edge and a variable round (2 mm to 5 mm) on the opposite top edge.
  The variable round is the procedural blend face that Solidcraft writes
  for its fillets. The modeller's volume is 23858.637 mm3.
- The test `release_fixture::a_release_design_with_fillets_imports_the_body_of_its_step_export`
  (in the `solidcraft_import` binary) checks that the `.sc` keeps a blend
  face, and that the `.sc` and STEP bodies have the same faces, volume,
  box and machining mesh.
- To track the next Solidcraft surface kinds: at a new Solidcraft release,
  save the design again with that release, export its STEP, replace the
  two files and run the test.

**Why.** The synthetic test of PR #3969 encodes its body with the pinned
solidcore crates. It does not prove that a file written by a Solidcraft
release imports, which is what "Done means" asks for.

**Considered and dropped.**
- A larger part with many fillets (the QA parts p06 and p07). They are not
  in the repository, and a small block keeps the test at about 10 s in
  debug.
- Constant fillets only. Solidcraft can write those as cylinders or tori,
  so they do not prove the blend surface kind.

**Cost to change later.** Small: two fixture files and one test.

## The STEP bench row

- Chosen: `filleted_block.step` gets a row in the STEP agreement bench
  (`step_agreement/rows.rs`) with `Truck::NotRecorded`, because the bench
  fails on a fixture with no row. The truck reader left the workspace in
  #2790, so no truck reading is made up. The pinned verdicts are "no truck
  reading; unit agrees" and "watertight, within tolerance".
- Dropped: a subdirectory outside the bench, because the file is a plain
  release fixture and the bench is the right place to judge it.
