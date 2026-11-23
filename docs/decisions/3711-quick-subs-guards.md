## #3711 - the quick submodule filter reads trait impls and scoped pub as shared

**Decided.**

- `quick_subs::filter` treats these as shared, so the whole suite runs:
  - `impl <Trait> for <Type>` (also generic and `unsafe impl`);
  - `pub` after attributes on the same line;
  - `pub(crate)`, `pub(super)`, `pub(in ..)`.

  An inherent `impl Type { .. }` of a private type keeps the filter.
- `braces::code` reads `cr".."` and `cr#".."#` as raw strings. An escaped
  char literal (`'\''`, `'\\'`) ends at its last quote.
- Two scanner defects that were already on main are fixed in
  `scan::test_fns`, because the new pins found them:
  - A `#[test]` below a line like `#[allow(dead_code)] pub fn helper() {}`
    made `test_fns` loop forever: the attribute walk took the fn line, and
    the loop did not move on. The memory use went up without a limit. Now
    the walk stops at a line that holds a fn, and the loop always moves on.
  - `fn_re` did not read `pub(crate) fn` or `const` / `unsafe` /
    `extern "C"` fns. Such a line was read as an attribute of the next test,
    so an edit there ran only that test.
- `run_step` (the fixture harness) returns libtest's stdout. The real run of
  a `<sub>::` step reads "4 filtered out".

**Why.** When in doubt, the whole suite runs. A quick tier that hangs or
runs less by mistake is the risk.

**Dropped.** A reference analysis (which module uses which item). The
pattern test is coarse but safe.

**Cost to change.** Small: one regex in `quick_subs.rs`, one in `scan.rs`,
and two match arms in `braces.rs`.
