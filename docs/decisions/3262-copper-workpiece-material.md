## #3262 - Copper workpiece material preset (C11000)

**Decided.**

- Add `WorkMaterial::Copper` at the end of `WorkMaterial`, `WorkMaterial::ALL`, `TABLE`, and `TURN_TABLE` (index `12`), with file word `"copper"` and UI label `"Copper (C11000)"`.
- Scope the preset to unalloyed wrought copper (C11000 ETP / C10100 OFHC, soft/annealed to half-hard, ISO 513 group `'N'`).
- Set the milling row (`crates/tools/src/cutting.rs`) to `kc11: 950.0` N/mm², `mc: 0.25`, HSS `vc: (45.0, 80.0)` m/min and `fz10: (0.03, 0.06)` mm/tooth, and Carbide `vc: (120.0, 240.0)` m/min and `fz10: (0.04, 0.08)` mm/tooth, using the metal chip-load diameter bands (`fz_scale`).
- Set the turning row (`crates/tools/src/turning_data.rs`) to HSS `vc: (45.0, 90.0)` m/min and `frev: (0.08, 0.20)` mm/rev, and Carbide `vc: (120.0, 260.0)` m/min and `frev: (0.08, 0.25)` mm/rev.
- Keep `PROJECT_VERSION` unchanged because `.camproj` files store `WorkMaterial::word()` rather than enum discriminants.
- Keep existing untabulated tool limitations (`Drill`, `Tap`, `Taper`, `Wire`, etc.) and `Unknown` row fallback behavior unchanged.

**Why.**

- Pure/unalloyed wrought copper (such as C11000 heater-block stock) is soft, ductile, and prone to built-up edge and gumming compared with free-machining brass.
- Conservative handbook speeds and a high-side `kc11` (from public Machining/Machinery's Handbook ISO N and Kienzle ranges for unalloyed copper, 700–1100 N/mm²) ensure power checks err toward warning rather than under-predicting stall, while chip loads avoid light rubbing.
- Appending `Copper` preserves existing discriminant indices used in `TABLE`, `TURN_TABLE`, and UI combo mappings, and keeps old project files valid.

**Dropped.**

- Free-machining tellurium copper (`C14500`) or age-hardened beryllium copper (`C17200`): faster or harder than pure wrought copper; users running specialty copper alloys can override feeds/speeds via per-material tool rows.
- Using the woodworking/plastics `router_scale` curve: wrought copper requires metal chip-load scaling (`fz_scale`).
- Bumping `PROJECT_VERSION`: unneeded because unknown words on older readers are already refused by name.

**Cost to change.**

- Constants live in one row each in `crates/tools/src/cutting.rs` and `crates/tools/src/turning_data.rs` plus their pinning tests in `crates/tools` and `crates/app/tests/cutting_data.rs`.
