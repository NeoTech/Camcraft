## #3345 - "Do not touch" support for 2D Pocket operations

**What was decided.**

- Added `OpType::Pocket` to `do_not_touch_applies` in `crates/app/src/operation/types/avoid.rs`, and updated `do_not_touch_unsupported`, `set_operation_do_not_touch` in `crates/app/src/mcp/tools.rs`, `docs/agents/mcp-tools.md`, and the UI boundary tests (`crates/ui/src/egui_app/tests/do_not_touch.rs`).
- Wired `OpParams::Pocket` in `crates/app/src/operation/generate.rs` for `(rest, &avoid_spec)` to `machining2d::pocket_avoid_entered_started`, `machining2d::rest_pocket_avoid_with_relief`, and `machining2d::rest_pocket_previous_avoid_with_relief`, leaving paths with empty `Do not touch` routed to the unchanged plain generators.
- Extracted per-level planning, relief intersection checks (`check_reliefs_avoid`), crossing-free arc enforcement (`enforce_level_crossing_free`), and shared rest emission (`emit_rest_avoid`) into `crates/motion/src/machining2d/pocket_avoid_level.rs`, unified behind `pocket_avoid_over` (`AvoidSource::Plain | AvoidSource::Rest`).
- Checked overhead protected geometry up to `p.clearance_z.max(z)` for both `level_center` and corner reliefs so initial rapid entry and final clearance lift cannot enter overhead protected volumes above `retract_z`.
- Guarded `Descent::Helix` (`descent::push`) and `entry_restart` (`descent_dive.rs`) against prior moves via `helix_clears_prior`, `find_clear_helix_raw`, `moves_clear`, and `helix_cross_fallback`, and recorded between-pass `KeepDown` chords in `LinkState.level` (`ladder_links.rs`), preventing `OffsetRings` + `KeepDown` + `Helix` crossings when keep-out changes passes between levels.
- Was stacked on PR #3353 (`base-3353`), which edits `pocket_avoid.rs`, `rest2d.rs`, `mod.rs`, and `generate.rs`. That PR has merged.

**Why.**

- 2D Pocket spans all five strategies (`Waterline`, `OffsetRings`, `Zigzag`, `Auto`, `Adaptive`), pre-drilled entry holes, corner reliefs, and both rest modes. Routing all modes through `pocket_avoid_over` guarantees consistent full-tool 3D keep-out subtraction, refusal naming, and overhead clearance across every level without duplicating level-walking logic.
- When stepped cutter geometry (flutes, shank, holder) trims inner rings at deeper Z levels, adjacent `OffsetRings` helix entries between levels share the `plunge_height` Z band; checking `moves_cross_prior` matches `machining3d::descent` and keeps every level transition crossing-free.

**Considered and dropped.**

- Maintaining separate avoid loops for plain 2D Pocket, `PassStart` 2D Offset, and 2D rest machining: dropped due to duplicated per-level keep-out, relief, and arc-crossing checks.
- Suppressing `plunge_height` above `mat_top` on lower levels to eliminate helix Z-band overlap: dropped to keep standard rapid-to-lead clearance intact; searching alternate clear helices along the pass matches 3D Pocket behavior without changing collision-free paths.

**Cost to change later.**

Low; `pocket_avoid_over` and `descent_dive.rs` isolate all avoid and helix-clearing policy from non-protected paths.
