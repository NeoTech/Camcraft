## #2188 - Route cam.post and MCP post to per-operation files when per_op is enabled

**What was decided.**
- In `crates/app/src/script/api.rs` (`v_post`, which backs both `cam.post` and the MCP `post` tool), when `posting::post_options(&env.world).per_op` is `true`, derive `(dir, program)` from the target path (`super::report::per_op_target`) and post through `posting::post_per_op_with_script_options` or `posting::post_per_op_script_with_options`, returning `super::report::posted_per_op_view`.
- Document `operations?: PostedOpInfo[]` on `PostResult` in `crates/app/src/script/mod.rs` and `assets/scripts/cam.d.ts` so multi-operation `per_op` script runs return each file's path, operation key, line count, and optional program text consistently with `mounts?: PostedMountInfo[]`.
- Refuse combined-program posting APIs (`posting_mounts::single`, `posting_mounts::post_mounts_with`, and `posting::stream_records`) by name via `posting_mounts::refuse_per_op` whenever `setup_post_options::in_force(world).per_op` is `true`.

**Why.**
- `posting_setup_change::stop_of` suppresses between-setups stops when `per_op` is `true` because each operation is meant to be posted into its own standalone program file.
- `v_post` previously routed only through `post_mounts_with_script_options` / `post_mounts_script_with_options`, which produced a single combined program whose setup-change stops had been stripped by `stop_of`.
- Routing `v_post` to `post_per_op_*` gives MCP `post` and `cam.post` the same standalone per-operation output the GUI produces, while `refuse_per_op` prevents any single-file or per-mount posting entry point from ever emitting a combined program with missing setup-change stops when `per_op` is enabled.

**What was considered and dropped.**
- *Refusing `per_op` in `v_post` instead of calling `post_per_op_*`:* would leave MCP and scripting users unable to post documents whose `per_op` option is enabled without toggling the option off, even though `post_per_op_*` already exists and works headlessly.
- *Changing `op_streams_by` in `crates/app/src/posting.rs` to parameterize setup-change folding:* `op_streams_by` is actively edited by open PR #2136 (which carries merge conflicts), and combined-program callers (`post_setup`, `post_script`, `post_setup_unwritten`, `post_mounts`, `stream_records`) already cannot represent multiple per-op files, so refusing `per_op` at those combined-program doors enforces the safety invariant without overlapping #2136.
- *Delegating `post_mounts_with` directly to `post_per_op`:* `post_mounts` returns `Vec<PostedMount>` (one entry per machine/post mount) and is called by the UI's file-path command, which assumes per-mount semantics rather than per-op semantics.

**What it would cost to change later.**
- Low. `v_post` dispatch is a single branch in `script/api.rs`, `posted_per_op_view` is isolated in `script/report.rs`, and the `refuse_per_op` guards are single-call checks at the combined-program entry points.
