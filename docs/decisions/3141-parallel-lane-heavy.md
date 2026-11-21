## #3141 - Move the wide-pocket parallel-lane test to the motion heavy lane

**What was decided.** Mark
`wide_rectangular_pocket_is_cleared_by_parallel_lanes` in
`crates/motion/tests/adaptive_troch_parallel.rs` with
`#[ignore = "heavy lane: ..."]`. The release tier and `cargo xtask gate
heavy` run it with `--ignored` in the step `motion heavy:
adaptive_troch_parallel`. The test keeps its two pockets (30 x 24 mm D6 and
44 x 36 mm D10), all its shape assertions and its 90-second budget. The
budget is now a guard on the release run. Add the light twin
`wide_rectangular_pocket_is_cleared_by_parallel_lanes_light`, which checks
the 30 x 24 mm D6 pocket only, in release. This replaces the rule of
decision #2990 "Apply this assertion in every run" for this test.

**Why.** The debug test ran in 36, 64, 106, 140 and 145 seconds on
unchanged motion code (#3116, #3141). The post-merge runner was loaded on
the red runs. Wall time on a loaded runner is not a function of the code,
so the debug budget made main red with no defect. AGENTS.md "Testing" says
that a test above about 60 s in debug moves to the heavy lane. In release
the same work is much faster, so the 90-second budget still catches a real
slowdown of the planner. The light twin keeps one parallel-lane shape check
at review and merge time. The `corner_cleanup_reduces_34x16_rest_to_at_most_hsm_rest`
test stays in the default debug run.

**Considered and dropped.**
- A larger debug budget. Any number can fail on a loaded runner, and a
  large number does not catch a slowdown.
- A smaller fixture. Decision #2990 already measured the smaller D10
  rectangles: a 38 x 33 mm pocket failed the rest-area assertion. A smaller
  pocket weakens the case that a D10 tool needs a short side above 32.36 mm.
- Removing the budget. It is the only guard on the planner time of this case.

**Cost to change later.** The batch gate no longer runs the two wide-pocket
checks in debug. If the planner becomes fast enough in debug, remove the
ignore and keep the budget only with measured evidence from a loaded run.
