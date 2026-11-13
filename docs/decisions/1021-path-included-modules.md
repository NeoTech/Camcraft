## #1021 - quick resolves `#[path]`-included files to their parent module

**What was decided.** `cargo xtask gate quick` maps a changed source file to
the module the compiler sees. A file that a sibling pulls in with
`#[path = "x.rs"] mod name;` maps to `(parent, name)` as well as to the
module named after the file: a union, so the change can only add suites. Example: `crates/app/src/posting_index_swing.rs`
is `app::posting::index_swing`, so the suites whose Gate map says
`app::posting (index_swing)` run, `index_swing` among them. The code is
`xtask/src/path_mods.rs`, pinned by
`gate_maps_tests::a_path_included_file_selects_its_parents_submodule_suites`.

**Why.** The issue proposed renaming the maps to `app::posting_index_swing`.
But the maps were right: the module really is `posting::index_swing`. The
plan was wrong to read the file name as a module. Fixing it in the plan
covers every `#[path]` file (posting's nine siblings, stock_sim,
drill_reach, ui state, and others) without touching any suite header.

**Considered and dropped.** Renaming the maps (wrong module name, and every
other `#[path]` file would stay unmapped). Adding both spellings to each map
(noise, and the next sibling misses again).

**Scope.** Only same-directory includes are resolved, which is every
`#[path]` in the tree today. A file nothing includes keeps its own name, as
before.

**A union, not a replacement (review of PR #1033).** The first cut mapped
the file only to its parent's submodule. That selected FEWER suites wherever
a map names the file's own stem, for example `app::posting_sim` in
`wall_column_gouge` and `contour_arc_sim`, `app::posting_suppressed` in
`rest_across_setups`, and `app::posting_plasma` in `plasma_gate`. It also
narrowed `*_tests.rs` includes sharply (`drill_reach_tests` from 122 suites to
6). The plan now maps both names, so every suite selected before is still
selected (pinned in the same test). `#[path]` lines inside `//` comments are
ignored.
