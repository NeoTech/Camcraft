# 3189 - Fold the camcraft-motion integration tests into domain directory suites

**Decided.** The 153 flat, non-slow test files of `crates/motion/tests/` move
into ten domain directory suites: `adaptive2d`, `contour2d`, `machining2d`,
`holes`, `entry_links`, `machining3d`, `finishing3d`, `multiaxis`, `turning`
and `edges_edm`. Each suite has a `main.rs` that declares one module for each
former flat file. The crate now has 15 test binaries: the ten suites, the
three `SLOW_BINARIES` (`finish3d`, `pivot5d`, `flow`, still flat), and the two
directory suites that were there before (`rest_adaptive`, `rotary_flow`).

**Why.**
- Each former file moves with `git mv` and keeps its text, except for one
  or two lines in 14 files:
  - five `mod common;` lines become `use crate::common;`. The suite root
    declares `common` once with `#[path = "../common/mod.rs"]`;
  - the three `adaptive_troch_*` files share `common/adaptive_troch.rs`. The
    `adaptive2d` root declares it once as `fixtures`, and each file has
    `use crate::fixtures;` (clippy's `duplicate_mod` refuses three copies in
    one binary). Their ignore reasons now name the `motion heavy: adaptive2d`
    step;
  - `arc_turn_kinds` (`#[path]`), `adaptive_orbit_seam` (`include_str!`) and
    `rotary_adaptive` (`#[path]` on `rotary_adaptive_fixture`) get a `../`
    prefix. `tests/fixtures/` and `tests/rotary_adaptive_fixture/` stay in
    place, because the lib's unit tests read them too;
  - two regenerate commands in doc comments name the new suite.

  Git's rename detection can then carry the edits of open PRs onto the
  moved files.
- A support directory of a file (`deburr/`, `multiaxis/`,
  `pass_start_levels/` and others) moves next to the file. A `#[path]`
  attribute is relative to the directory of its file, so these attributes
  do not change.
- The Gate map paragraphs stay in the moved files. The gate reads them
  through the suite root (`quick::gate_maps`, #3186). Quick selection now
  runs a whole suite, or one module of it, for a change.
- The suite names follow the issue's list. `drilling` is `holes`, because
  the group also holds Bore and Circular. Chamfer, V-carve and the two EDM
  files form `edges_edm`, so that the count stays below 16 with room for one
  more suite.

**Dropped.**
- `autotests = false` with `[[test]]` entries that point at the old files.
  The xtask scanner finds binaries from the directory layout, not from the
  manifest, so it would see each old file as a binary.
- Folding `rest_adaptive` and `rotary_flow` as well. They are directory
  suites already; a move gives no saving.
- Duplicating `common` in each module (`#[path]` on each `mod common;`).
  That compiles the helpers five times in one binary, and clippy refuses it.
- Splitting the slow folded suites further. In one debug run (6 test threads, one binary at a time)
  `entry_links` took 186 s, `adaptive2d` 155 s and `machining3d` 141 s.
  Batch runs the suites in parallel lanes, and no single test moved. If a
  suite proves too slow on the runner, it can split along its module list.

**Cost to change later.** A file moves back with `git mv` and the reverse of
its line edit. A new motion test goes into one of the ten suites (AGENTS.md
**New tests join a directory suite**).
