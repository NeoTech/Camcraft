## #2400 - Dogbone and T-bone inside-corner relief for 2D pockets and slots

**What was decided:**
- Add `CornerRelief` (`kind`, `tbone_direction`, and per-corner `overrides` matched by XY) to `OpParams::Pocket`, `OpParams::Adaptive2D`, and `OpParams::Offset2D`, with empty/off defaults.
- Refuse non-empty `CornerRelief` on `Adaptive2D` with `AdaptiveMethod::Trochoidal` by name in both `sane` and `generate`.
- Compute each spur in `machining2d::corner_relief` from the raw boundary loop (collapsing micro-fillet chords within `FILLET_CHORD_COS`): place the cutter center along the inside bisector (`Dogbone`) or tangent to the selected wall (`TBone`) to offset `tool_radius + stock_to_leave`, then splice `[start, end, start]` into pocket passes (`corner_relief_spur`) and seed corner-blend bans at spur anchors.
- Enforce safety guards before emission (`operation::corner_relief`): refuse unreachable corners, wall crossings, unmatched overrides, stock exits, `Exclude` / open far-side crossings, `DoNotTouch` keepouts, kept/neighbour steel hits, and multi-height thin-wall punch-throughs.
- Declare bounded `verify::CornerReliefZone` corridors (`radius <= tool_radius`, `z_bottom..=z_top`, inside stock) for `verify` and `stock_sim::credit_corner_reliefs`, reconstructed live from current geometry in `operation::corner_relief_sim::zones_for_op`.

**Why:**
- Keeping `CornerRelief::default()` as off and omitting it from parameter digests and JSON DTOs when off preserves all existing operation digests and saved project files.
- Splicing continuous spurs onto pocket passes avoids separate plunge retracts while letting pocket entry, links, and stepdowns govern the whole cut.
- Bounding verification grants strictly to cutter radius, cut floor, stock boundary, and live CAD slices ensures real gouges and wall breakthroughs still fail.

**What was considered and dropped:**
- Emitting separate plunge or drill moves for corner reliefs: dropped because discrete plunges break cut continuity and bypass pocket link/entry guarantees.
- Expanding the input `Region` with Boolean ears before pocketing: dropped because inner island/keyway geometry can still block cutter approach or create slivers that Clipper rounds away instead of reaching the true corner vertex.
- Supporting `Adaptive2D::Trochoidal` corner relief silently or via perimeter re-cuts: dropped because trochoidal loop paths do not follow corner passes cleanly; an explicit refusal prevents silent omissions.

**Cost to change later:**
- Low to moderate: planning is isolated in `crates/motion/src/machining2d/corner_relief.rs` and `corner_relief_spur.rs`, verification is isolated in `crates/sim/src/verify/corner_relief.rs` and `stock_sim/corner_relief.rs`, and saved projects skip serializing default reliefs.
