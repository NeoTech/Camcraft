## #2011 - EDM Wire Tool and Sinker Electrode UI Visibility

**What was decided.**
- Added `ToolKind::Wire` (`"wire"` / `"EDM wire"`, non-spinning `rpm == 0`, `corner_r == 0`, `diameter` in `[0.02, 1.0]` mm, `material_group` for wire material such as `"brass"`, no per-material cutting rows) in `crates/tools/src/wire.rs` and starter catalog entry `"EDM-W025"` (`0.25` mm brass wire) in `assets/tools/catalog.json`.
- Updated `OpType::WireContour` to mount a `ToolKind::Wire` library tool (`tool_law::WIRE`, `mounts_tool() == true`) and made `wire_op::condition_of` read `wire_diameter` from the mounted wire tool (`wire_op::wire_diameter_of`).
- Relaxed `wire_op::one_condition` so `edm.conditions[]` entries only require `gap` (`wire_diameter` on a condition is optional); when a condition or `edm.wire_diameters` states a diameter, it must match the mounted wire tool within `WIRE_MATCH_MM` (`0.001` mm), and `gap < wire_diameter * 0.5` is refused by name across main, skim, and tab cut-off conditions.
- Kept tool digests (`stamp_tool::write`) and wire condition digests (`wire_op::stamp_condition`) together in `current_stamp` and `stale_reason` so editing or swapping the mounted wire tool marks the Wire Contour stale.
- Gated tool kinds offered in the Tool Library catalog, Tool type filter, and New Tool Type dropdown via `machine_modes::tool_kind_offered`: a tool kind appears only if at least one `KindInfo` entry accepts it and admits a machine kind in `MACHINE_MODES` or already mounted on the document/setup. This hides `Sinker electrode` / `EDM-E10X6` (and unreferenced `Waterjet nozzle`) while keeping `EDM wire` / `EDM-W025` visible and unlocking `Sinker electrode` automatically when `SinkerEdm` becomes selectable or mounted.

**Why.**
- Wire EDM operators expect to select a wire (diameter, material) like a tool in CAM software, and existing `ToolKind::Wire` + catalog `"EDM-W025"` eliminates empty-rack ambiguity without inventing a separate wire-spool registry or project entity schema.
- Enforcing `WIRE_MATCH_MM` agreement against `edm.wire_diameters` and any condition-stated `wire_diameter` prevents silent kerf mismatches between a mounted spool and a machine's technology table while remaining compatible with `.mch` files that state `wire_diameter` on conditions and allowing future machines to omit it and state only spark gaps.
- Filtering offered UI tool kinds off `KindInfo::ALL` + `MACHINE_MODES` + mounted machines fixes the Sinker EDM / Sinker Electrode half-exposed state with one shared rule across catalog and tool editor, without hardcoding special-case lists per widget.

**What was considered and dropped.**
- *Inventory-only wire selection (Option 2 in #2011)*: leaving `WireContour` with `mounts_tool() == false` and introducing a separate wire-spool field/command would have fragmented tool management (`AddTool`, `ChangeTool`, setup tool lists, and staling) and contradicted `Wire Contour`'s `Tool` row UX.
- *Ignoring condition-stated `wire_diameter`*: trusting only `tool.diameter` even when a condition explicitly stated a different `wire_diameter` would allow a 0.25 mm condition table entry to run under a 0.20 mm wire with wrong spark-gap assumptions.
- *Hardcoding `Kind::SinkerEdm` exclusion in the UI*: would diverge again when PR #1753 adds `SinkerEdm` to `MACHINE_MODES` or when users mount custom sinker machines.

**What it would cost to change later.**
- Removing condition-level `wire_diameter` from `.mch` files completely (if desired later) only requires deprecating that field in `edm_conditions.rs`; `wire_op` already treats it as optional.
- Migrating legacy pre-#2011 project files that stored `WireContour` with an empty `tool` string, if encountered, would only need accepting empty `op.tool` for `WireContour` in `project/validate.rs` (since `tool_problem` already flags a missing tool cleanly).
