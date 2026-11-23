## #3521 - The quick plan selects the suites that post the shipped examples

**Decided:** `xtask/src/example_suites.rs` adds nine whole app suites to the quick plan when the diff
touches `assets/posts/`, `assets/machines/`, `examples/`, `crates/post/src/` or `crates/motion/src/`:
`example_catalog`, `example_catalog_adaptive`, `example_catalog_edge_work`, `first_part_programs`,
`first_router_part`, `first_4axis_part`, `continuous_4axis_part`, `tilt_basis_compat` and
`catalog_groove_part_off`. Each one posts a shipped example, compares a shipped `.nc`, or reads a posted
catalog case. The list is fixed; a test pins that each name is an app test binary.

**Why:** PR #3478 and PR #3464 passed review with a red `example_catalog` and `first_part_programs`.
The content scan of `assets.rs` cannot find these suites: they read `examples/` and name no post
script. A fixed list is the smallest rule that covers them.

**Dropped:** `agie_native` (it reads an example only as input). The UI suites that render
`examples/` projects (`catalog_render`, `egui_app`, `gpu_render`): no post output, and a UI build for
each post change is too slow for quick. A list discovered from a scan of `examples/` in the test text:
it also selects the input-only readers, and the wider selection problem is #3557.

**Cost to change:** one line in `SUITES` or `INPUTS`. The heavy reach (`heavy.rs`) still uses only
`map_catalog` and does not change.
