## #1864 - 2D Contour on DXF sketches: layer/chain picking, per-chain overrides, and sketch verify

Maintainer request from #1452 ("We should be able to load DXF file to do profile cuts on."), continuing #1862 and #1863.

### What was decided and why

- **Extend `FeatureSel` with `SketchChain(ChainId)` and `SketchLayer(String)` (`crates/app/src/operation/types/sel.rs`).**
  `FeatureSel` drops `Copy` and derives `Clone` so `SketchLayer` can carry the layer name directly across UI state, commands, MCP IDs (`sketch_chain:<hex>` / `sketch_layer:<name>`), and `PROJECT_VERSION = 74` (`crates/app/src/project/file.rs`).
  - *Dropped*: interning DXF layer names into numeric IDs or resolving layers into fixed chain lists at pick time. Expanding a layer pick dynamically during resolution (`crates/app/src/operation/contour_sketch_pick.rs`) keeps layer picks accurate when a DXF sketch is reloaded.
- **Additive per-chain overrides on `OpParams::Contour` (`ChainOverride` in `crates/app/src/operation/types/params.rs` and `crates/app/src/operation/types/sel.rs`).**
  Each override keys on `(entity, chain: ChainId)` with optional `side: Option<ChainSide>` and `reverse: bool` (`#[serde(default, skip_serializing_if = "Vec::is_empty")]`). `PROJECT_VERSION` steps from `73` to `74` (main took v73 for #2055) so older builds refuse files carrying sketch picks or per-chain overrides instead of silently dropping them, while v73 files load unchanged.
  - *Dropped*: attaching `side`/`reverse` onto `GeoSel` itself (which would break layer picks and prevent plain selection equality) or minting a separate sketch-only operation variant. Storing `chain_overrides` on `OpParams::Contour` reuses the standard `EditOperationParams` undo/redo and `GenStamp` staleness checks, while `SetOperationGeometry::with_chain_overrides` lets MCP callers update picks and overrides in one undo step.
- **Natural nesting when uniform, ordered per-chain generation when overridden (`crates/app/src/operation/contour_sketch.rs`).**
  Whole sketches (`SrcRef::Entity`) and layer/chain picks with unmodified `Auto` sides and uniform `climb` check `can_group_natural` and cut as one nested `Region` (holes Inside, outers Outside). When per-chain `side` or `reverse` overrides diverge, closed chains generate in hole-before-outer order so inner bores clear while the part still holds them. Preserved closed polylines and sampled arcs keep true CL `ArcMove` geometry so `posting` emits `G2`/`G3`.
- **Explicit open-chain safety (`validate_plan` in `crates/app/src/operation/contour_sketch.rs`).**
  Open DXF chains cut along the centerline only. Operation-level `Side::Inside`/`Side::Outside`, any per-chain `ChainSide` override on an open chain, and control/wear compensation (`G41`/`G42`) refuse by name, naming the sketch chain; `reverse: true` remains allowed to flip centerline direction.
  - *Dropped*: guessing an inside/outside side from winding or open endpoints, which causes silent undercuts or wrong compensation.
- **Meshless generation and verification (`crates/app/src/sketch_model.rs`, `crates/app/src/operation/material.rs`, `crates/app/src/operation/job.rs`).**
  `sketch_model::generation_problem` admits `OpParams::Contour`, and `material::named_material` returns the placed sketch stock-top height with empty facets for `Sketch` entities (`part_solid = false`). Unresolved `TopOfModel`/`BottomOfModel` heights still refuse via `sketch_model::NO_SOLID_HEIGHT`. Stock removal (`stock_sim::simulate_document`), crossing, stock-rapid, and fixture collision checks run against setup stock and fixtures without requiring a 3D solid mesh.
- **Ray-to-curve picking and UI workflow (`crates/app/src/pick/sketch_pick.rs`, `crates/ui/src/edge_pick.rs`, `crates/ui/src/egui_app/geo_page.rs`, `crates/ui/src/egui_app/mill2d_page.rs`).**
  `pick_feature` projects sketch primitives at their setup pose, finds the closest curve within pick radius, and respects front solid occlusion unless `PickQuery::through` is set. `edge_pick` appends Shift-clicked `SketchChain` picks instead of running 3D loop extension, `geo_page` adds per-layer toggle buttons, `mill2d_page`/`fields.rs` edits `Per-chain overrides`, and `pick_highlight`/`interact` render hovered and selected sketch chains in world space.

### What it costs to change later

- A future sketch operation kind (such as `2D Pocket` on DXF profiles) can reuse `SketchContourPlan` resolution in `contour_sketch_pick.rs` and `named_material` stock-top handling while adding its own clearing generator and `sketch_model::generation_problem` arm.
- Adding per-chain lead or tab overrides on top of `ChainOverride` requires adding defaulted fields to `ChainOverrideDto` and another `PROJECT_VERSION` bump.
