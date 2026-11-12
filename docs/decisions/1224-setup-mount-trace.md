## #1224 - Trace of a setup's own machine and post to the posted files

**What was decided.** The seven trace rows are pinned by tests in the crate
that owns each stage. App rows are in `crates/app/tests/setup_mount_trace.rs`
(new), plus two edge tests in `setup_mounts_post.rs`. The MCP leg is in
`mcp_setup_mount_trace.rs` (new). The UI leg extends the #1206 pin in
`crates/ui/src/egui_app/tests/setup_mount_pages.rs` so it posts through the
shell's own Post command. Existing tests that already cover a row are named
in the PR table rather than copied. No product code changed.

The fixture uses shipped files whose difference shows in the output.
Machine A is the VMC 3-axis, post A the built-in Fanuc. Machine B is the
Tormach 770M (a 10000 rpm spindle, so the spindle limit tells the machines
apart), and post B is Siemens 840D (`;` comments, N words, G710, M2 against
Fanuc's O-word and M30). The tests assert dialect markers, the machine named
in each header, the operations and their order, and each file's work offset.

**Failures found.**
- #1232 (`P1`): a turning operation on a lathe setup whose own post is a
  milling dialect (Haas) posts a milling program instead of being refused.
  Pinned by `an_own_milling_post_on_a_lathe_setup_is_refused_by_name`,
  kept `#[ignore = "#1232: ..."]`.
- #1233 (`P5`): no MCP tool sets a setup's own post. The MCP leg therefore
  covers the machine pick only, and the post half is pinned app-side.

**Why.** The issue says to trace, not to fix. A failing row becomes its own
issue so the fix gets its own review at the safety-floor tier.

**Considered and dropped.**
- A shared fixture module under `crates/app/tests/`. Dropped: the quick gate
  treats any shared test input as "run every app suite". The fixture stays
  inside the one trace file, and the lathe edges reuse
  `setup_mounts_post.rs`'s existing mill/lathe fixture.
- Haas as post B. Dropped in favour of Siemens 840D, whose dialect differs
  from Fanuc on every line. Haas still appears as the document's new post
  in row 5 and in the UI leg.

**Limits.** These are headless tests. Nothing was looked at on screen, and
no file was run on a control.
