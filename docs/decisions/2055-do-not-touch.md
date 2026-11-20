## #2055 - "Do not touch" protected faces and edges on 2D Offset and 3D Pocket

### What was decided
- **Operations covered in this first slice**: `OpType::Offset2D` (`2D Offset`) and `OpType::Pocket3D` (`3D Pocket`) obey `"Do not touch"` across all strategies (`Waterline`, `Zigzag`, `Auto`, `OffsetRings`, `Adaptive`, `shell`, `rest_rough`, `feed_opt`), all `Entry3D` modes, all `Links` modes (including `KeepDown` level checks and elevated rapid traverses), and tilted 3+2 planes (`Offset2D { tilt: Some(...) }`). Unsupported operation kinds refuse non-default `"Do not touch"` by name across commands, UI, MCP, and generation so nothing is silently ignored.
- **Storage on `AreaControls` (`crates/app/src/operation/types/area.rs`, `avoid.rs`)**: Added `pub do_not_touch: DoNotTouch` inside `AreaControls` (with `has_boundary_edits()` separating 2D region boundary clipping from `"Do not touch"` and preserving exact legacy behavior/stamps when `DoNotTouch` is default), avoiding edits to 185 `Operation` literals while automatically propagating through undo/redo, paste, stale tracking, and UI drafts.
- **Independent 3D verification & generator keep-outs (`crates/sim/src/verify/avoid.rs`, `avoid_links.rs`, `avoid_poly.rs`)**: Added `Warning::AvoidHit { at, item, part, short_mm }` (`Copy`, `blocks_post() == true`). Per-Z-interval keep-out regions (`AVOID_PAD = 0.02 mm`) dilate Z-clipped triangles and 3D edges by each tool stack band (`Flute`, `Shank`, `Holder`) plus `clearance + max(stock_to_leave, 0)`, while `avoid::hits_named` (`AVOID_VERIFY_TOL = 1e-4 mm`) sweeps every move independently across all stack bands and midpose chords.
- **Per-level planners & rapid link elevation**: Extracted `pocket_avoid` (`crates/motion/src/machining2d/pocket_avoid.rs`) and `avoid`-aware `pocket3d_avoid` (`ladder.rs`, `shell.rs`, `entry.rs`), checking `prev_center` on `KeepDown` links and elevating obstructed horizontal rapid moves via `elevate_rapid_links`. Rest-roughing skips safely with a note if only protected columns remain unmachined after a clean roughing ladder.
- **Commands, MCP, UI, live post gate, and project format v64**:
  - `SetOperationDoNotTouch`, `EditOperationParams`, `SetOperationGeometry`, `ChangeOperationKind`, and `EditOperationLinks` validate picks (rejecting vertices) and refuse overlap between protected items and the machining selection or `Include` boundaries by name.
  - Added `get_operation_do_not_touch` / `set_operation_do_not_touch`, `"do_not_touch"` on `get_operation_geometry`, `"Do not touch"` controls under `Geometry` -> `Boundary` -> `Boundaries`, and persistent viewport highlighting (`avoid_viz.rs`) while an operation is selected.
  - Bumped `PROJECT_VERSION` `63 -> 64` (stacked on `#2077`) so older builds refuse `.camproj` files that carry `"do_not_touch"` protection rather than silently dropping it.

### Why
- Separating `has_boundary_edits()` from `is_default()` on `AreaControls` lets `Offset2D` and `Pocket3D` use `"Do not touch"` without altering 2D boundary containment/extension semantics or existing stamps/goldens.
- Using one independent 3D full-stack verifier (`sim::verify::avoid`) across generation (`verify_avoid`) and posting (`live_avoid_problems` + `blocks_post`) prevents any generator quirk, rapid hop, or post-generation part move from ever emitting unsafe G-code.
- Extending `PROJECT_VERSION` prevents pre-v64 readers from silently discarding protection and cutting protected geometry.

### Considered and dropped
- Storing `do_not_touch` directly on `OpParams` or `Operation` instead of `AreaControls`: rejected because it touches hundreds of struct literals and duplicates copy/paste/draft plumbing that `AreaControls` already handles.
- Rolling out across all 2D/3D/multi-axis/turning operations in one PR: rejected in favor of a tight, end-to-end verified first slice (`2D Offset` and `3D Pocket`) that avoids stacking conflicts with open PRs `#1817`, `#1822`, `#2037`, and `#2093`.

### Cost to change later
- Extending `"Do not touch"` to additional operation kinds (`Pocket`, `Adaptive2D`, `Contour`, `Finish3D`, etc.) is additive: add the `OpType` variant to `do_not_touch_applies` and pass `avoid_spec` to that generator arm; `verify_avoid`, `route_links`, UI, commands, MCP, and v64 project format already work unchanged.

### Merge refresh against main v66

Main now writes project v66 for the trunnion schema (#1962), following
cutting rows (v64) and the tilt basis (v65). The protected-pick format must
therefore write v67. Reusing v64 or v66 would let an existing reader accept
the file and silently discard `do_not_touch`. All earlier migrations and
refusals remain. The project test pins a version beyond v66, protection
round trips, future-version refusal, and blocked posting after a moved
protected item. Reusing main's version was rejected because it would weaken
that protection. Main's carried-stock replan and previous-stock mask are
kept together with the protected-item planners.

### Merge refresh against main v70 and review fixes (#2149)

Main advanced to `PROJECT_VERSION` 70 (v68 for Drill `break_through_from`, #2441; v69 for Pocket Adaptive retirement, #1517; v70 for library holder reference, #2016), added `cut_levels_under` (#2004), `pocket3d_open_prev` (#1950), `Field::new` in `crossing_hop_tests`, and `crates/ui/AGENTS.md` Frame cost rules (#2238) + pick-highlight caching (#2201). On the merged branch:
- `PROJECT_VERSION` steps `70 -> 71` for `"do_not_touch"`, keeping v68, v69, and v70 migrations and refusals intact.
- `pocket3d_avoid`, `pocket3d_open_avoid`, and `pocket_avoid` keep `cut_levels_under` and `air_stock` while preserving `avoid` keep-outs, `Field::new` is restored alongside `Field::new_with_avoid`, and `Warning::AvoidHit` coexists with `Warning::DrillUndersize`.
- `avoid::hits_named` sweeps continuous 3D chords `Move::segments3(ARC_SAG)` widened by `Bulge::of(...).deviation()` across all arc planes (including XZ/YZ vertical arcs) instead of stationary `solid::pieces` samples.
- `avoid_viz` memoizes 3D protected geometry and projected screen marks keyed on picks, entity handles, hidden state, `Weak<Features>`, `Weak<[Triangle]>`, setup pose, and MVP bits, and `shell.rs` wraps painting in `frame_cost::time("ui/viewport.avoid_marks", ...)`.

### Example-version review fix against main v72 (#2149)

The branch merges current main and keeps its v72 linked-model fields,
checks, and migration notes. Protected projects use v73 so v72 readers
refuse the file instead of discarding protection. The committed continuous
4-axis example is regenerated at v73. Its test compares the saved project
directly with that example and keeps the posted-program checks.

Keeping the example at an older version, or migrating it inside the
comparison test, was rejected because either choice hides a stale committed
example. Reusing v72 was rejected because it weakens the protection barrier.
A later format change must update the writer, reader-floor pin, migration
notes, and committed example together.
