## #3190 - Fold wave 1: the `turning_*` app suites join the `turning` directory suite

**Decided.** The first fold wave of #3190 moves the 18 flat
`crates/app/tests/turning_*.rs` binaries into `crates/app/tests/turning/`.
The existing `turning.rs` becomes their root. It loads each file with
`#[path = "turning/<name>.rs"] mod turning_<name>;`.

- Each module keeps its old binary name as its module name. A test that
  was `turning_bore` / `x` is now `turning` / `turning_bore::x`. A filter
  `turning_bore::` selects the same tests as the old binary.
- Each submodule keeps its own `//!` header, Gate map and lane line.
  `xtask` joins the Gate maps of all sources of a suite (`quick::gate_maps`).
- `mod common;` in a submodule becomes `use crate::common;`. Four files
  declared `common` and did not use it. Their line is removed.
- Two `include_str!` paths get one more `../`, because the file is one
  directory deeper.

**Why this wave.** The `turning` domain has 18 flat binaries. No open PR
touches them, and none is in `ISOLATED` or `SLOW_BINARIES`. The `mcp`
domain is larger, but 9 open PRs change `mcp.rs` or a flat `mcp_*.rs`. A
fold there now makes conflicts for each of them.

**Readings.**
- Before: 19 binaries, 115 tests, 0 ignored. After: 1 binary, the same
  115 qualified names (sorted lists are equal), 0 ignored.
- Disk: the 18 old binaries took 2,306 MB in debug on Linux. The folded
  binary takes 201 MB.
- The folded suite runs in about 6 s (three runs, all green).
- Each temp file name in these tests has a prefix of its own and the
  process id. No two submodules write the same path in the shared process.

**Dropped.**
- One PR for all 607 suites. The issue asks for bounded waves, and one
  large PR conflicts with each open test PR.
- Short module names (`bore`, `thread`). They change the test paths that
  `RECOMMENDED.md`, logs and the heavy filters use.
- A new root file without tests. `turning.rs` already exists and stays
  under 400 lines (383).

**Cost to change later.** A rename of a module is a one-line change in
`turning.rs`.
