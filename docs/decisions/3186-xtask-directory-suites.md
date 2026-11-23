## #3186 - xtask reads directory suites in asset selection, the catalog check and the heavy filter

**Decided.**

- `assets::suites_for` reads the entry file and each module file of a suite
  (`suites::sources`). A directory suite whose child module names an asset
  is selected. A flat suite whose `mod x;` file names an asset is now also
  selected. This can only add suites to a quick plan.
- `catalog::map_catalog` finds the catalog suite with
  `suites::entry(root, "app", "example_catalog")`.
- `plan::may_run_ignored` walks all module files of a directory suite
  (`tests/<b>/main.rs`). A `mod x;` line alone no longer gives a heavy step.
  `include!`, `#[path]`, a walk error, an unreadable file and a declared
  module whose file is absent (`suites::sources_and_missing`) still give a
  heavy step.
- A flat suite keeps the old rule: a `mod x;` line gives a heavy step.
- The `quick.rs` part (a `--test <suite> <submod>::` filter) is not in this
  PR. It is #3706. The PR says `Refs #3186`.

**Why.**

- A flat-suite walk removes about 175 heavy steps of the cut plan
  (`cargo xtask gate release --dry-run`: 552 lines before, 202 after). Each
  of these suites has no `#[ignore]` in its files, but a change of that size
  to the cut gate needs its own review. The issue names only directory
  suites. With the flat rule kept, the heavy plan loses only
  `motion heavy: rest_adaptive`.
- A missing module file can be a cfg-gated module. Its text is unknown, so
  the gate keeps the step ("the gate never drops a binary by accident").
- The quick submodule filter needs a new pick kind in `Picks`, `describe`
  and `steps`. That is more than half of this work.

**Dropped.** A flat-suite walk in this PR (recorded in #3706). A count of
`mod` lines in place of the missing-file count from the module walk: the
walk already resolves `mod.rs`, nested and inline module paths.

**Cost to change.** Small: one branch in `may_run_ignored`.
