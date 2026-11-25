## #2811 - 3D Finish switch to stop the waterline ring pass on steep flanks

- **What was decided**:
  1. Added `waterlines: bool` (default `true`) across `Finish3DParams`, `OpParams::Finish3D`, `ParamsDto::Finish3D`, and MCP `add_operation` / `edit_operation` / `list_operation_types`.
  2. Applied `waterlines: false` in `rings_and_tail_with_trim` (shared close for `Parallel`, `ParallelMulti`, `Scallop`, `SteepShallow`, `Ramp`, `Radial`, `Spiral`, `MorphSpiral`, `Contour`, `Project`, `Flow`) and `swarf_multi`: skip `emit_rings_with_trim`, refuse all-steep or no-shallow-cut models with `GenError::Empty`, count steep flank facets (`|m_z| < SPLIT_MZ`) in the depth window, and emit a `WATERLINES_OFF_NOTE` path warning surfaced by `entry_status_lines` in the UI status banner.
  3. Refused `waterlines: false` by name for ring-dependent patterns (`Waterline`, `Blend`, `Morph`), and hid the control in the UI for patterns without a steep-flank waterline pass (`Waterline`, `Blend`, `Morph`, `Flat`, `Pencil`, `Corner`, `Geodesic`).
  4. Serialized `waterlines` unconditionally in `ParamsDto::Finish3D` with `#[serde(default = "default_true")]` (no `PROJECT_VERSION` bump) and used an append-only conditional stamp byte (`0x4E` when `false`) in `stamp_mill3d`.
  5. Exposed the UI control as `Steep flanks` `radio_pair` (`Waterlines` / `Leave uncut`) with an inline `consequence` warning when `Leave uncut` is selected, and split `finishing3d/validate.rs` out of `finishing3d/params.rs` to stay under the 400-line limit.
- **Why**: Keeps every existing project and default toolpath stamp byte-identical, avoids `list_operation_types` / `patched()` drift for bool defaults, makes uncut steep flanks visible in both UI and MCP output, and prevents `Waterline`/`Blend`/`Morph` from silently ignoring the switch.
- **What was considered and dropped**:
  - Applying `waterlines: false` to `Blend` or silently ignoring it on `Waterline`/`Morph` was dropped because `Blend` relies on the ring pass across the 30-45 degree band and silent no-ops violate the refusal law.
  - Using `skip_serializing_if = "is_true"` was dropped because `list_operation_types` discovers fields from serialized `seed()` JSON and `climb`/`ramp`/`entry_ramp` already serialize unconditionally.
  - Leaving `params.rs` unsplit at >420 lines was dropped to obey the 400-line file rule.
- **Cost to change later**: Low. Each layer checks `waterlines` directly; changing per-pattern eligibility or note wording touches one match arm and one helper.
