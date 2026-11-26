## #4027 - Path kernel K7: the law suite and its stand-in proofs

**What was decided**

1. **The suite is the existing directory suite `crates/pathkernel/tests/laws/`.**
   The issue text says "an existing directory suite"; the K0 record says
   `crates/pathkernel/tests/laws/`. The record is the approved design, and
   the crate skeleton (#4055, PR #4123) already made `tests/laws.rs` with
   its invariant tests and wrote that "the K7 law suite (#4027) joins this
   directory suite". So the two texts agree: K7 adds submodules to the one
   binary `laws`, and adds no new top-level test file. No xtask list
   changes: the gate already maps `crates/pathkernel/tests/laws/` to that
   binary.
2. **One generic check for each law.** Each check is a function over the
   stage traits (`GuidePlanner`, `OrderPlanner`, `LoadPlanner`,
   `LinkProof`, `EntryPlanner`) and the stage outputs of one run. It
   returns a `LawVerdict`: the law, held or broken, and the worst measure
   (its place, its point, its measured value and its limit).
   - L1 `check_l1`: two calls, the region given another way (loops
     reversed, each started at its next vertex), and each guide planner of
     another kind with the same choice. Curve multisets within 1 um
     (Hausdorff per curve, same count).
   - L2 `check_l2`: compile-level (a region stage is a
     `fn(MaterialSide) -> ...`), then the centre changes with the material
     side, and the order stage gives the same sequence for two regions that
     differ only in their material side.
   - L3 `check_l3`: Auto, Inside and Outside; the ordered curves against
     the guide curves, and the guide and the ordered multiset across the
     sides, within 1 um.
   - L4 `check_l4`: a guide family other than the asked one, or an entry
     into material of another kind than the asked one, needs a named
     fallback of its stage.
   - L5 `check_l5`, L6 `check_l6`: read on a stock grid that the checks own
     (`grid.rs`), written apart from the planner's stock reader. The walk
     reads each entry and each cut on the stock as it is at that step and
     clears the swept disc. L5: a descent onto material needs an
     `IntoMaterial` proof. L6: the engagement is read from the contact
     angle of the front half of the cutting edge, `ae/D = (1 - cos angle) / 2`
     (the record names the contact-angle law). Each read above
     `limit x (1 + PLAN_TOL)` plus the grid read error (0.1 ae/D) needs an
     over-limit record within one tool radius. The first version of the
     check found a real under-read of the first stand-in load planner at
     the corners of sharp offset rings (ae/D 0.75 against the limit 0.5).
     A hand estimate of the contact angle there gives about 0.77. The
     stand-in now names the corners.
   - L7 `check_l7`: the cuts in order, point by point, against the
     recorded old output, within 1 um. A case with no record is a breach,
     so that no kind moves without its golden output.
   - Link rule `check_link`: the plan's lifts against the tally (total and
     by reason), no lift on a fixture with no split ring, and no stay-down
     hop that leaves the centre area and the air.
3. **The mutation proof uses stand-in planners in the test.** No kind has
   a kernel planner yet. `standins.rs` holds one planner set that obeys
   each law (sharp offset rings, a side order, a load reader, a straight
   link, a helix entry). Each law file holds one or more planners that
   break its law in one named way, and a test that the check names the law
   and the place. The stand-ins are test code, not a design of K2-K5.
4. **The registry is the progress list.** `registry.rs` holds the 16
   phase-1 kinds of the record, each `NotYet`. A kind that moves sets its
   row to `Moved` and registers its recipe (planners and cases); the test
   `each_moved_kind_obeys_every_law` then runs `check_all` (all eight
   checks) on each case. The guard test fails with a clear message when a
   row is `Moved` with no recipe, or has a recipe and is not `Moved`. The
   test `the_law_runner_passes_a_registered_kind_that_obeys_every_law` runs
   the stand-ins through the same path.
5. **No change in `crates/pathkernel/src`.** Each law can be read through
   the public API. The suite builds a `PlanarRegion` in two places:
   `kit::rebuild` (L1's permutation and L2's flipped material side) and
   `standins::pocket_region` (the fixtures). K1 (#4025) changes
   `PlanarRegion::new` and adds a checked `region(...)`. After K1 merges,
   these two functions must take K1's constructor: the PR of #4025 or of
   #4027 that merges second takes `git merge origin/side/path-kernel` and
   fixes the two calls.

**How the two halves meet at the switch-over.** The stage-level laws are
here: they read the kernel's own outputs (`GuideSet`, `OrderedSet`,
`Controlled`, `Plan`, `LinkTally`) for each moved kind. The toolpath-level
readings (lifts, crossings, entries into material, engagement and
fallbacks of a finished toolpath, old against new) are in PR #4167
(#4154), in the tests of `camcraft-motion`. The kernel crate must not
depend on motion (a dependency cycle), and motion's `tests/common` is not
a library. At the switch-over proof (record, point 1 and 2): point 1 is
this suite green for each of the 16 rows; point 2 is the #4154 readings
table, old against new, on the disc, the dog bone and `fig8.step`. A kind
PR fills both: its row and recipe here, and its readings there.

**What was dropped**

- A new top-level test binary: the suite exists, and a new binary costs
  about the size of a test build.
- A dev-dependency of the kernel on motion, to share the #4154 measures:
  it makes a second copy of the kernel types, and the record forbids the
  edge.
- Reading the posted CL in this crate: the kernel must not depend on
  `camcraft-cl`. The checks read the kernel's `Plan` instead; the CL
  readings are #4154's half.

**Known limits, for K5 and the first kind PR**

- The entry stage has no channel to name a fallback: `EntryPlanner::entry`
  returns `Result<Entry, Refusal>`, not a `Verdict`. The record says "an
  entry where the user asked for another kind is a `Fallback`". Today
  `check_l4` reports such an entry as a breach. K5 must add the record
  (for example a `Verdict<Entry>`), and `check_l4` then reads it.
- Under `LoadControl::None` the limit is 1, so the `Controlled` constructor
  cannot name a full-width cut. The record's "full-width cuts outside a
  named slot = 0" is not checked for that control.
- The record's fixtures (the dog bone of #3741, the disc of #3934, the
  narrow neck, the open-side band) join with the first kind that moves:
  each kind registers its cases. The stand-ins use a square pocket and a
  pocket with an island.

**Cost to change later**: a check is one function in one file. A new law
adds one file and one line in `check_all`.
