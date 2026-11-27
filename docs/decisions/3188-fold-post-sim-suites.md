## #3188 - Fold the camcraft-post and camcraft-sim integration tests into directory suites

**What was decided.** The 143 flat test binaries of `crates/post/tests/` now
build as 9 directory suites, and the 95 of `crates/sim/tests/` as 8. Each
old file is a module of one suite (`tests/<suite>/<old name>.rs`) with its
text unchanged except for these mechanical edits:

- `mod recorded;` (and the other shared helpers `inverted_support`,
  `mirror_trace`, `macro_b`, `ac_trunnion_support`) became
  `use crate::<helper>;`. The suite's `main.rs` declares each helper one
  time, by `#[path]`, before its members (the `post_suite!` macro of
  `recorded` needs `#[macro_use]` there).
- Each relative `include_str!` / `include_bytes!` path and each
  `#[path = "../../app/..."]` gained one `../`. `#[path]` into the kept
  `coincident_arcs/` and `klartext/` directories gained one `../`.
- A doc command `cargo test -p camcraft-post --test <old>` now reads
  `--test <suite> <old>::`.
- A submodule directory with one owner (`agie_safe_setup`, `fanuc_post`,
  `heidenhain_cycles`, `sodick_ln2w`, `upright_after_tilt`,
  `richauto_safety`, sim `model_excess`) moved with its owner.

Post suites: `script_goldens` (every `script_*` recorded-golden suite and
`recorded_streams`), `script_cycles`, `script_probe`, `script_wcs`,
`script_rotary`, `script_edm`, `script_dialects`, `script_arcs`,
`post_engine`. Sim suites: `stock_engines`, `stock_field`, `stock_carry`,
`fixture_laws`, `reach_laws`, `gouge_laws`, `rest_credit`, `sim_timeline`.

**Why.** The issue asks for at most 12 binaries in each crate with the same
`#[test]` count. `cargo test --list` before and after lists 2353 tests, and
each old test maps to exactly one new test (its name gains the module
prefix). The `#[test]` count in the source is 1579 before and after.

The post suite names all match the `OUTPUT_SUITES` pattern of
`.github/scripts/post-merge-report.sh` (`post_engine` and `script_*`). A
red post suite on main is still classed as a wrong-output failure (P1),
and the workflow script needs no change.

`recorded::tmp` adds a per-process counter to its temp file name. Before,
each script suite was its own process, so `<script>_<case>_<pid>` was
unique. Now one process runs all script modules in parallel, and two
modules that post the same script and case would write the same file.

**Dropped.**
- Keeping the module files at `tests/` with `#[path]` from each suite: Cargo
  builds every `tests/*.rs` as a binary, so the files must move (or the
  manifest needs `autotests = false`, which the issue did not ask for).
- One copy of each helper per member (`#[path]` in each file): the helper
  would compile many times in one binary.
- Moving the data directories (`golden/`, `recorded/`, `klartext/`,
  `coincident_arcs/`): other crates, `.gitattributes` and the post
  scripts' docs read them by path.
- Editing the code comments in `crates/app/tests/`, `crates/gcode/tests/`
  and the GPU fixtures that name an old path: that would put those app
  binaries in this PR's gate for a comment. A `P7` follow-up updates them.

**Cost to change later.** A suite can be split or renamed with `git mv` and
one `mod` line; the gate reads the suites from the tree.
