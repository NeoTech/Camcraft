## #1962 - AC trunnion `.mch` schema, rotary chain order, and explicit non-RTCP capability

**Decided.**
1. **Parent-child chain alongside `ctrl_coord`-sorted `axes`.** `Machine.axes` keeps its existing `controller.synced_configuration` sort order so no existing `axes` caller changes. `AxisLimits.chain_order: Option<u32>` preserves the depth-first parent-child nesting order from `kinematics.default.parts` on each rotary itself (so callers/tests that rename a rotary in `axes` stay consistent), `AxisLimits.parent: Option<String>` records each rotary's nearest enclosing rotary ancestor, and `Machine::rotary_axes()` returns rotaries sorted by `chain_order` (falling back to `axes` order when unstated on legacy or hand-built structs).
2. **Optional rotary trunnion fields (`#[serde(default)]`).** `AxisLimits` adds `pivot: Option<[f64; 3]>` (mm, unit-scaled), `offset: Option<[f64; 3]>` (mm, unit-scaled), `zero_offset: Option<f64>` (degrees, never length-scaled), `rollover: Option<bool>`, and `sign_convention: Option<RotarySignConvention>`. Absent or `null` stays `None` under the unstated-limit law; `[0, 0, 0]` positions and `0.0` `zero_offset` are preserved as stated values. Invalid types, vector lengths, non-finite / inch-overflow values, or rotary keys on linear parts are refused by named key through one shared helper (`crates/post/src/machine/trunnion.rs`) called by both `parse_mch` and `validate_text`.
3. **One explicit sign convention (`tool_rh` vs `table_rh`).** Added `RotarySignConvention::{ToolRh, TableRh}` (`"tool_rh"` | `"table_rh"`), accepted on `kinematics.default`, `controller`, or individual rotary parts (refusing by name if `controller` and `kinematics.default` disagree). Unstated defaults to ISO 841 / RS-274 `tool_rh` in `Machine::effective_sign_convention()` and `script_props` (`machine.signConvention` and `machine.rotaries[].signConvention`) so the checker and posts always agree.
4. **Explicit `controller.rtcp` guard and project version bump.** `Machine.controller_rtcp: Option<bool>` records `controller.rtcp`. When `false`, `mirror()` and `script_props()` set `machine.rtcp = false` even when a rotary tilts off Z, while mirroring the full rotary chain (`parent`, `pivot`, `offset`, `zeroOffset`, `rollover`, `signConvention`) for future CAM-side IK posts (#1965, #1968). `doosan-dnm.cps` refuses multi-rotary moves without controller RTCP even when `rotaryBrake` options are present, matching `fanuc.cps`. `PROJECT_VERSION` bumps (63 -> 64, stacked on #2126) so older builds refuse `.camproj` files rather than discarding `controller_rtcp: false` and emitting `G43.4`.

**Considered and dropped.**
- Re-sorting `Machine.axes` by tree order instead of `ctrl_coord`: dropped because existing UI/sheet/test callers expect `Machine.axes` in `controller.synced_configuration` order.
- Making `sign_convention` required on every `.mch`: dropped because all existing shipped `.mch` files omit it and follow ISO 841 (`tool_rh`) per `assets/machines/AGENTS.md`.
- Keeping `PROJECT_VERSION` unchanged: dropped because `MachineDto` serializes `Machine` with serde defaults and older readers would silently drop `controller_rtcp: false` and infer `rtcp = true` from `A`'s tilt vector.

**Cost to change.** Local to `crates/post/src/machine/trunnion.rs`, `post/machine_rotary.rs`, and `project/file.rs`.

### PR #2162 review correction

**Decided.** Move the value-only Break-through helper and its unit tests to `camcraft-motion`. App keeps the setup reads and passes the values and `Z_TOL` to the helper. App re-exports the helper. The existing dependency supplies the API; no dependency was added.

**Considered and dropped.** Keeping the helper in app violates the domain ownership rule. The helper reads no document state.

**Cost to change.** Local to the motion helper and the app generation call.

### PR #2162 merge refresh

**Decided.** Merge `upstream/main` (`4e0b9abef`) bringing #2126's merged `camcraft_motion::break_through` (internal `Z_TOL = 1e-6`, 5-arg `judge`), #2015's v64, and #2133's v65. Take `upstream/main`'s `break_through` module, test, and `generate.rs` call instead of altering #2126's merged signature. Advance `PROJECT_VERSION` from 65 to 66 so v65 and older readers refuse files carrying `controller_rtcp: false` rather than dropping it.

### PR #2162 compatibility metadata correction

**Decided.** Record the project format v65 -> v66 compatibility change in a new commit's `BREAKING CHANGE:` footer and in the release-note Changelog text. v66 preserves the new trunnion machine fields, including explicit non-RTCP capability; v65 and older readers refuse v66 files.

**Considered and dropped.** Rewriting existing commits would discard the reviewed history. Leaving the bump unmarked would hide the incompatible file-format change from release tooling.

**Cost to change.** Commit metadata and release-note text only.
