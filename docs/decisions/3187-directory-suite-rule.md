## #3187 - a new integration test joins an existing directory suite

**Decided.**

- `AGENTS.md` "Testing" has a new rule, **New tests join a directory
  suite**. A new integration test goes into an existing domain directory
  suite (`tests/<suite>/<submod>.rs`) before a new top-level test binary is
  made.
- Exempt: the suites in `ISOLATED` (`xtask/src/lanes.rs`) and
  `SLOW_BINARIES` (`xtask/src/plan.rs`). They must stay separate binaries.
- A new top-level integration test file needs a one-line reason in the PR
  body.
- `AGENTS.md` "Code shape" has one line that points to the rule. It does
  not repeat it.
- `.claude/agents/PR-reviewer.md`, step 6, Placement bullet, has check (d).
  A PR with a new top-level file under `crates/*/tests/` and no reason in
  its body, outside `ISOLATED` and `SLOW_BINARIES`, gets a requested change.

**Owner's decision** (live answer in the question dialog, 2026-11-23,
relayed by the main session and recorded by the coordinator in a comment on
#3187): "YES, add it." The question was: add the rule "a new integration
test joins an existing directory suite", with the exemptions `ISOLATED` and
`SLOW_BINARIES`?

**Why.** Each integration test binary of `camcraft-app` takes about 100 MB.
Workspace test binaries went from 412 to 1,020 in ten days. A test in a
directory suite shares one binary and one link step. The directory-suite
support in `cargo xtask` is on main (#3186, #3706), so the rule can take
effect.

**Dropped.**

- Converting existing flat test files to suites. This is separate work.
- A gate check that refuses a new top-level file. The reviewer check is
  enough for now. A gate check can follow.
- A fixed list of allowed reasons. The PR body line is free text.

**Cost to change later.** Low. The rule is two process-file edits. No
code depends on it.
