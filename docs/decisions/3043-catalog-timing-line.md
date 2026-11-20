## #3043 - The catalog timing line is one write; the contract test runs its cases in order

**Decided.** `within` (`crates/app/tests/example_catalog/support.rs`) makes
the whole timing line into one `String` (`timing_line`) and writes it with
one `write_all` on the locked stderr. The contract test
`two_broken_cases_report_both_names_and_times` runs its inner test binary
with `--test-threads=1`. Its asserts do not change.

**Why.** The post-merge run 44780029308 failed at `contract.rs:59`. The
inner run started the two broken cases in parallel. Stderr has no buffer,
so the formatted timing line went out in pieces. The default panic hook of
the other case thread wrote its message between the pieces:

```
catalog 2d/contour-tabs regenerates
thread '<unnamed>' panicked at crates/app/tests/example_catalog/support.rs:68:5:
2d/first-part: missing model/
 elapsed=0.000s budget=60.000s
```

The assert looks for the contiguous text `catalog {case} {action}
elapsed=`, so it failed. This is a race. It occurs more on a loaded Linux
runner. The one-write change also keeps the timing lines whole in normal
parallel gate logs. The one-thread inner run makes the contract test
deterministic.

**Dropped.**
- A retry of the inner run: it hides the race and does not remove it.
- A looser assert (find `catalog {case} {action}` and `elapsed=` apart):
  it weakens the contract that each case reports its name and its time on
  one line.
- Only one of the two changes: each change alone removes this failure, but
  the one-write change protects the real gate logs, and the one-thread run
  protects the test from other writers to stderr.

**Cost to change.** Low. Two test files only; no product code.
