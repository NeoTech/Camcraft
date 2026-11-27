## #3560 - The heavy reach of a library file follows the Gate maps

**What was decided.** `cargo xtask gate heavy` maps a changed library file
`crates/<d>/src/<rest>` to these targets:

- the `lib`, `bins` and `doc` targets of `d`, when `d` is a heavy crate;
- each heavy integration binary whose Gate map names a module of the file.

The module names come from the quick plan (`quick::src_modules`). The
matching uses the quick plan's rule (`quick::suites_naming`). A
`#[path]` file has two names, and one named module is enough.

The crate reach (each heavy lane of each dependent crate) stays in these
cases:

- the crate is shared (`geometry`, `cl`), where quick runs batch;
- the file is a crate root (`lib.rs`, `main.rs`, `bin/`) or is not under
  `src/`;
- no Gate map names any module name of the file.

PR #3737 already narrowed `#[cfg(test)]` and binary-only files. That part
does not change.

**Why.** The maintainer decided on 2026-11-22 to run only the heavy targets
that a change touches. The Gate maps are a seam that the project already
trusts: each test file names the modules that it covers ("A test lives with
the code it tests"), and the quick tier selects its suites by them. The
heavy reach now uses the same seam. Example: `motion/src/drilling/boss.rs`
goes from 115 steps to 11 steps.

The full heavy lanes still run at each cut, in the cut gate. A test that
the narrow reach misses at review is still run before a release. AGENTS.md
"Heavy lanes at review" also runs no full heavy lane at review, so the
narrow reach is a tool and not a merge condition.

**Considered and dropped.**

- A per-module `use` graph. It is not sound without guards for glob
  imports, re-exports, trait impls, method calls and registries (see the
  comment on #3560 from the PR #3737 worker). It also adds a parser.
- "Every module name must have a Gate map" for `#[path]` files. The own
  name of such a file is never in a Gate map, so each `#[path]` file then
  kept the crate reach. Quick uses the union of the names, and so does this
  change.
- Skip the `bins` and `doc` targets of `d`. Doc tests in the changed file
  and unit tests in a binary can call the module. The two steps are short.

**Cost to change later.** The change is in `xtask/src/heavy.rs`
(`gate_map_reach`). To go back to the crate reach, make that function
return `None`. A sharper seam can replace it under the same name.
