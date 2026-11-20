## #2502 - Split contour entry from shared band generation

**Chosen:** Stack on PR #2460 at `ceac40dd9` and move the single-region
entry functions, `PartRef`, and `contour_impl` into `contour_region.rs`.
Keep `PreparedBand`, `prepare_band`, `emit_band`, `finish_contour`,
`air_side`, and `unit` in `contour.rs`. Re-export the entry functions
through `contour.rs` to preserve the public API.

**Why:** The band extraction in #2460 left `contour.rs` at 514 lines.
The split follows the domain seam between single-region orchestration and
band generation shared with `contour_whole`. It leaves `contour.rs` at
396 lines and `contour_region.rs` at 132 lines without changing behavior.

**Considered and dropped:** Moving the emission body would disturb the
tab-handling lines changed by PR #2486. Keeping that body in place makes
the split smaller and avoids that overlap. Existing tests remain untouched
as the issue requires. A temporary external structural pin checks the
400-line limit without adding a repository test or review evidence.

**Cost to change later:** A different module boundary would require moving
the functions and adjusting internal imports. The public re-exports keep
callers independent of this layout; no project format or toolpath behavior
changes with the split.
