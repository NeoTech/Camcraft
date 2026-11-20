## #2569 - A light heavy tier, and a local full rerun before the cut gate restarts

**What was decided**

- `cargo xtask gate light` is a tier of its own. It runs the light twins of
  the heavy tests in release: one release build of the crates that hold a
  twin, then one step for each test target with a twin (`<crate> light: lib`,
  `<crate> light: <binary>`).
- A light twin is found from the source. Its ignore reason starts with
  `light lane:` and its name is `<full test>_light`. The planner refuses a
  twin by name when its file has no `#[ignore]`d full test.
- The first twin is
  `the_slot_keeps_its_verdict_under_sub_micron_perturbations_light` in
  `crates/motion/src/machining2d/adaptive/tests.rs`. It plans the slot 2
  times (opening 180 nm deeper, slot 125 nm narrower). The full sweep keeps
  its 20 rows and does not change.
- A red light run prints a note for the PR. The note names each red step and
  the command of its full heavy step
  (`cargo xtask gate release --only "<crate> heavy: <target>"`).
- AGENTS.md has two new named rules: **The light lane** and **Local full
  rerun**.

**Why**

- A marker in the source needs no list in `xtask` that can go stale. The
  known-fail markers (#2567) use the same method.
- The name suffix makes the libtest filter safe. A step passes the twin
  names as filters after `--ignored`. A name that ends in `_light` does not
  select a full heavy test by accident.
- The refusal keeps the rule "the golden asserts stay in the ignored set". A
  twin cannot take the place of its full test.
- Only the crates that hold a twin are built. The tier stays at a few
  minutes while there are few twins.
- The two slot rows are rows that dropped the lifted plan before the #982
  fix on Linux x86-64. The 32 nm row is already a debug test.

**Considered and dropped**

- A flag on the release tier (`gate release --light`). Dropped: the release
  tier has the lanes of the cut, the known-fail issue check and the debug
  suites. A light run needs none of them.
- A fixed name filter (`--ignored light`). Dropped: a substring such as
  `light` also selects tests named `highlight_...`.
- `--exact` with the full libtest path. Dropped: `xtask` would need the
  module path of each test. The suffix gives the same safety.
- A mapping from a light log directory to `gate release --failed`. Dropped:
  the step names differ (`light:` and `heavy:`). The note prints the
  `--only` commands instead.
- The wiring into `/review-prs`, `/merge-ready` and `pr-test.yml`. Dropped
  from this PR: issue #2568 owns when the heavy tests of a PR run.
- The text for the `gate` and `release-cut` skills. Not in this PR: the
  worker session could not write files in `.claude/skills/`. The PR body has
  the text. The rules are in AGENTS.md until a session with that permission
  adds the skill text.

**Cost to change later**

- Low. The marker text and the suffix are two constants in
  `xtask/src/light.rs`. A new twin needs no change in `xtask`.
- The release tier runs the twins too, because its heavy steps run each
  ignored test. That costs about 10 s for each twin at a cut.
