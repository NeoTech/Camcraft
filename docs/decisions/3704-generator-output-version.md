## #3704 - A project saved before PR #3684 keeps the old start of a tabbed 2D Contour until the user regenerates it

**What was decided.** Rule option 1, as a generator-version term at the salt
(`crates/app/src/operation/stamp.rs`): **a change of generator output that
fixes a defect makes old caches stale.** The named constant
`GENERATOR_OUTPUT_VERSION` is written into every operation stamp. The PR
that changes what a generator emits - a moved start, dive, lead, level or
entry, any emitted move - bumps the number in the same PR. Every cache a
project saved under the old number reads Stale once and clears on
regenerate. PR #3684 (the first tabbed window enters where the levels above
cut) is exactly such a change: with the term, the project saved before it
reads Stale instead of posting the old 18 mm plunge at the first tab
window. The bump is manual by rule; the code never auto-detects an output
change.

**Why.** The stamp's written rule asked for a new salt only when a verify
law changes, so #3684's changed start stayed invisible to the stamp and a
saved project posted the old wrong move while reading as current. The
emitted path is what the user machines; a cache of an older output must
never read current (the Prime Directive: never post a silently wrong path).

**Considered and dropped.**
- Option 2 (old caches stay current; the release notes tell the user to
  regenerate): the wrong path posts silently, which the Prime Directive
  refuses.
- Auto-detecting output changes (a canary regeneration per PR): heavy, and
  the detection can miss; a manual bump on the PR that owns the fix is the
  rule.
- A per-kind version term (only the tabbed contour stales): finer churn,
  but every future fix would re-decide the scope, and a wrong scope re-opens
  this bug. One number is the lean rule the issue asked to decide.

**Cost to change later.** Bumping the number is one line. Narrowing it to
per-kind terms later keeps the constant as the default and the rule text at
the salt.

**Pins.** `crates/app/src/operation/stamp_tests.rs`:
- `a_bump_of_the_generator_output_version_moves_the_stamp`
- `an_unchanged_generator_output_version_keeps_a_saved_cache_current`
- `a_tabbed_contour_saved_before_the_bump_reads_stale`
