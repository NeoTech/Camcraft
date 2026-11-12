## #920 - AGENTS.md placement rules: tests, crate edges, app scope

**What was decided.** Three rules go into AGENTS.md (test placement in
"Testing"; crate-edge and app-scope rules in "Code shape"), are named in the
PR-reviewer checklist and the work-issue skill, and PROCESS_VERSION goes to 17.
The fourth proposed rule (a new test joins an existing suite) is left out: it
waits for directory suites in xtask and for timings (#919 research, Q5).

**Why.** Maintainer direction (2026-11-11: "high cohesion, low coupling,
because every gate is costly"). The #919 research counted 36 app test files
(226 tests) that use nothing `app` defines, and 86 suites that name moved
modules as `app::<module>`, so quick misses them.

**Rule 2 small-dependency clause.** "Fewer than five items" is as proposed in
#919. Its wording is the maintainer's to confirm on this PR.

**Considered and dropped.** The fourth rule (above). A hard ban on any new
crate edge: rejected, since a real edge with a decision entry is cheaper than
duplicating types.
