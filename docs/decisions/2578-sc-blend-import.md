# #2578 - Solidcraft `.sc` import of a body with a blend face

- **Chosen:** no code change. The solidcore pin moved past solidcore commit
  `676f5e2c9` (`Surface::Blend`) in PR #2797, and again in PR #3802
  (`fa67b3f5`). On upstream/main `b13f790ec` a `.sc` design with a blend
  face imports, keeps its blend face, has the modeller's volume, and meshes
  for machining within 0.005 mm. The pin test
  `blend::a_design_with_a_fillet_imports_its_blend_body`
  (`crates/kernel/tests/solidcraft_import/blend.rs`) passes without a fix.
- **Why:** the refusal came only from the old pinned decoder. The kernel,
  the record build and the machining mesh accept the blend kind at the new
  pin.
- **Fixture:** the test builds the fixture. It replaces the sample design's
  final body with a 10 x 20 x 30 block that has one variable round (2 mm to
  5 mm). A constant round on two planes is an exact cylinder and keeps no
  blend definition, so it does not pin the `blend` kind.
- **Dropped:** a committed `.sc` file from a SolidCraft 0.44 release. No
  SolidCraft build is in this environment. The built fixture uses the same
  encoder (`solidcore_record::native::encode`) at the pinned commit.
- **Open acceptance checks:** add a fixture from a current SolidCraft
  release and compare the imported body with its STEP export. The generated
  fixture compares with the modeller's volume and box and pins the decoder
  regression. The PR uses `Refs #2578` for this partial coverage.
- **Noted:** the kernel's box of the filleted block is 2.8e-4 mm too large
  on one side (the blend's net). The test permits up to 1e-3 mm, as for
  trimmed cylinders (#939).
- **Review decision:** replace PR #3949 with a branch from current
  `upstream/main`. Keep the regression test and use only `Refs #2578`
  in the replacement commits. The old history has a closing reference
  that would close the issue before the two open checks are complete.
- **Alternative dropped:** merge PR #3949 and reopen the issue. A
  missed manual step would lose the open acceptance checks.
- **Cost to change:** the replacement PR must complete the release
  fixture and STEP comparison before it can close issue #2578.
