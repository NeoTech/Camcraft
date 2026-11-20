## #2201 - Pick highlight frame cost: input-keyed caches only

**What was decided.** The refresh of PR #2256 against main keeps only the
caches whose key is the input itself (crates/ui/AGENTS.md, "Frame cost",
#2238):

- `frame_cache::FrameCache::world_verts_cached`: each body's world-space
  vertices, keyed by the facet allocation (`Weak<[Triangle]>`, compared by
  pointer, plus the length) and the pose bits.
- `pick_highlight::HighlightCache`: the resolved outlines of the shown
  picks, the viewport hover and the Geometry-row hover. The key is the
  same as the key of `pick_line_memo`: the picks by value, and for each
  picked body its display-off flag, its entity, its `Arc<Features>`
  allocation and its view pose.
- `pick_highlight_mesh`: the batched screen meshes of these outlines,
  in three slots (selected, viewport hover, row hover). The key is
  `MeshKey`: the outline id, the camera matrix, the pane mapping, the
  clip, the pixels per point, the widths and the colours. Each slot
  (`MeshSlot`) has its own build counter. A hit sends a clone of the
  `Arc<Mesh>`, not a deep copy.
- `geometry::feature`: the `[Option<Pos>; 3]` digest (the same hash as the
  `Vec` form) and one `patch_lattice()` call for `patches` and
  `patch_rings`.
- `model::toolpath_nonempty`, in `model/toolpath.rs` (model.rs is past
  400 lines). `toolpath_warning_count` had no caller and is removed.

**Why.** Main now has input-keyed memos for the same work:
`model::features_of` (Weak memo), `model::resolve_chain`, `model::patches`,
`model::patch_rings`, `frame_memo::scene_scope` and `pick_line_memo`. The
PR's other caches keyed on a document counter (`WorldStamp`,
`revision_token`). The frame-cost rule forbids a counter as the only key.

**Hand-resolved hunks (same logic on both sides).**

- `model::features_of` / `features_of_shared`: main's side. The PR's
  `SHARED_FEATURES_CACHE` is dropped; main's Weak memo does the same work.
- `resolve_chain` call sites (state, interact, pick_highlight): main's
  `model::resolve_chain`. The PR's `feature/index.rs` is deleted.
- `world::collect`: main's `frame_memo::scene_scope`. The PR's
  `scene_cache_key` and `collect_shared` are dropped. `world_verts` keeps
  the PR's vertex cache under the new key.
- `pick_ink::paint`: the PR's batched selection and hover meshes, and
  main's start and exit marks as plain strokes between them.
- `shell.rs`: the PR's `paint_row_hover`, inside main's
  `frame_cost::time("ui/viewport.row_hover")`.

**Dropped.** The scene, region and toolpath-line caches, the six panel
caches (`panel_cache.rs`) and `commands::revision_token`. They were keyed
by a document counter, and the review asked for counter and changed-input
tests for each. Main's memos cover the scene and the pick lines. A panel
cache comes back only with a measurement and an input key.

**Hand-resolved hunk of the second refresh (main at `ed4a9940f`).**

- `egui_app/tests.rs` module list: both sides added one module at the
  same place (`highlight_perf` and main's `pocket_no_adaptive`). Both are
  kept. #2484 (shared pick names) merged without a conflict: its
  `geo_label` status lines in `interact.rs` and its chip names in
  `state.rs` stay as main has them.

**Counters split.** The outline cache counted the viewport hover and the
row hover in one counter. Each now has its own (`hover_builds`,
`row_hover_builds`), so each slot has its own pin.

**Cost to change later.** Low. Each kept cache is one module with a counter
test and changed-input tests:

- `frame_cache_tests.rs`: the world-vertex cache.
- `pick_highlight_tests.rs`: the outline cache (row hover and viewport
  hover, each with its own counter).
- `pick_highlight_mesh_tests.rs`: the screen-mesh cache. For each of the
  three slots, a still frame tessellates once, and an orbit, a pane
  resize and an ink change each tessellate again with moved or recoloured
  painted vertices.

**Not done here (review non-blockers, left open).** The vertex cache keeps
one entry for each pose of a live body (bounded at 32), the vertex cache
hit still clones the vertex vector, and the law tests of
`interact::hover_features` / `row_hover_segments` use
`pick_highlight::project_segments`, not the production projection in
`pick_highlight_mesh`.
