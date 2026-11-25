## #3514 - The slow motion tests move to the heavy lane with light twins (slice A)

**What was decided.**

- The issue is worked in three slices by area. Slice A takes the six slow
  tests of `crates/motion/src/machining2d/adaptive_troch_tests.rs`. Slices
  B (`adaptive/tests.rs`, `pocket_hsm_ring.rs`) and C (the three
  `tests/adaptive_troch_*.rs` binaries) follow in their own PRs.
- The six full tests and their six light twins go into a child module,
  `adaptive_troch_heavy_tests.rs` (`mod heavy` of the `tests` module). The
  light gate (`xtask/src/light.rs`) refuses a twin whose own file has no
  ignored full test, so a twin must sit in the same file as its full test.
  The parent file goes from 388 to 219 lines; the child file stays under
  400 lines.
- The issue says "`cargo xtask gate heavy` runs the full tests". This is
  met by one targeted release run of each moved full test, by name. The
  full heavy lane runs at the cut (AGENTS.md "Heavy lanes at review",
  #3556). A PR does not run the full heavy lane.
- A twin asserts the same law as its full test on a reduced case (one
  level in place of two, one region in place of two, a smaller pocket, or
  a part of the full test's calls). The full test keeps every assert and
  every value; only its module path changed (`super::super::` became
  `super::super::super::`).

**Why.** The six tests took 76 s to 334 s each in debug (#3427), more
than the 60 s limit of AGENTS.md "Testing". The light lane keeps a fast
check of each law at review time.

**Dropped.** A sibling module next to `tests` in `adaptive_troch.rs`
(option b of the context pack): it needs the private helpers made
`pub(super)` and a change to a library file. Twins in a separate file
from their full tests: the light gate refuses them.

**Cost to change later.** Low. The module is test code only.
