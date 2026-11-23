## #3706 - quick runs one changed submodule; the heavy filter reads flat suites through their modules

**Decided.**

- Quick: a change outside the test fns of one submodule file of a directory
  suite (`tests/<suite>/<sub>.rs`) runs `--test <suite> -- <sub>::`. The
  whole suite runs when:
  - `main.rs` changes outside its test fns;
  - the submodule holds an item that other modules can use: a `pub` or
    `pub(..)` item, `macro_rules!`, `macro_export` or `macro_use`, now or at
    the merge base;
  - the submodule is cfg-gated or not precise for the scanner;
  - the text at the merge base is not available;
  - more than one submodule of the suite changes;
  - another rule picks the suite whole (a Gate map, an asset, a fixture).
- The filter is a libtest name filter. A test whose path contains `<sub>::`
  runs. A name filter can only run more tests than the submodule holds,
  never fewer.
- Exact fn picks inside the filtered submodule are dropped: the filter runs
  them.
- `plan::may_run_ignored` reads every suite through its module files, flat
  suites too. These keep the heavy step (the safe answer):
  - `ignore`, `include!`, `#[path`, `#[cfg_attr(.., path = ..)]` in any file;
  - an unreadable file or a module walk error;
  - a declared module whose file is absent;
  - a `mod` token in code that the line scan did not read as a declaration
    (`mod a; mod b;` on one line, `#[cfg(test)] mod b;`, a declaration split
    over lines). `braces::code` blanks strings and comments for this count.
- `quick.rs` was at 400 lines. `describe` and `by_fn` moved to
  `quick_describe.rs`. The submodule rule is in `quick_subs.rs`.

**Why.**

- The owner's standing word: take the recommended option. When in doubt,
  the whole suite runs, because a gate that runs less by mistake is the risk.
- The cut plan drops 174 heavy steps (273 to 99). A check outside the
  module walk read each dropped suite: its entry file, plus `tests/<m>.rs`
  and all of `tests/<m>/` for each `mod <m>` of the entry. 174 suites, 0
  with the word `ignore`.

**Dropped.**

- Per-source Gate maps in `map_modules` (a src change that names one
  submodule's Gate map). The joined map still runs the whole suite. This is
  the safe side, and no suite on main needs it yet.
- A reference analysis of which items other modules use. The `pub` test is
  coarse but safe.

**Cost to change.** Small. `quick_subs::filter` holds the whole rule;
`may_run_ignored` and `suites::modules` hold the heavy guards.
