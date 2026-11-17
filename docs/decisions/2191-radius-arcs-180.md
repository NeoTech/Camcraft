## #2191 - Split near-180 and near-360 R-arcs and validate post-rounding chord <= 2|R|

**Decided.**

- In `emitArcWithRadius` across all six R-capable milling posts (`fanuc.cps`, `doosan-dnm.cps`, `haas.cps`, `mazak.cps`, `tormach.cps`, `grblhal.cps`):
  1. Compute both endpoint radii (`r0`, `r1`), average them as `r = (r0 + r1) * 0.5`, and fall back to I/J/K (`return false`) when `Math.abs(r0 - r1) > Math.min(0.02, Math.max(0.002, r * 0.005))`.
  2. Split full and near-360° arcs (`sweep >= 2 * Math.PI - Math.PI / 9`) into 4 quadrant segments (`n = 4`), and split near-180° arcs (`Math.abs(sweep - Math.PI) <= Math.PI / 9`) into 2 ~90° segments (`n = 2`), interpolating the helical axis `h` linearly per segment.
  3. Validate every planned segment after output rounding (`fu(...)` relative to `[orgX, orgY, orgZ]`): `rp > 0`, `chord > 0`, `chord <= 2 * rp - 1e-7`, and `chord <= 2 * rp * 0.99` when unsplit (`count === 1`); if an unsplit arc triggers that guard, re-split into quadrant segments (`Math.max(2, Math.ceil(sweep / (Math.PI / 2)))`).
  4. If any check fails even after splitting, return `false` before calling `block(...)` so `onCircular` falls back cleanly to I/J/K without partial output or stale plane state.

**Why.** Helix half-turns snapped to the 1 µm lattice can have slightly unequal `r0`/`r1`, causing separate rounding of endpoints (`wo(...)`) and radius (`w("R", ...)`) to produce `chord > 2|R|` (`2.469 > 2.468`) or mismatched `R1.235`/`R1.234` across the two half-turns, and amplification `0.5 * tan(sweep / 2)` shifts the reconstructed center sharply near 180°. Splitting near-180° and near-360° arcs into ~90° segments keeps `useRadiusArcs: true` in R-form while bounding center drift and keeping every chord well below `2|R|`, with post-rounding I/J/K fallback as a hard safety floor (e.g., inch `R0.` or mismatched radii).

**Considered and dropped.**

- Unconditional I/J/K fallback for all arcs near 180°. Dropped: `useRadiusArcs: true` is specifically enabled for helix/pocket milling workflows that want R-form output whenever safe, and the existing post already split 360° arcs into 180° R-arcs.
- Splitting full circles into 2 × 180° arcs. Dropped: each 180° segment suffers the exact same `chord > 2R` and tangent center-shift vulnerability as an unsplit helix half-turn; 4 × 90° segments avoid both.
- Rejecting any nonzero `|r0 - r1| > 1e-6`. Dropped: recorded real `.cl` fixtures exhibit up to ~4.7 µm lattice endpoint variation (`finish3d_waterline_cone.cl`, `thread_mill_m8_multi_form.cl`) that R-averaging + quadrant splitting handles safely.

**Cost to change later.** Confined strictly to `emitArcWithRadius` in the six `.cps` post scripts and the `crates/post/tests/radius_arcs.rs` integration suite.

**PR #2202 review fix.** Fanuc and Doosan keep the original I/J/K arc when the block writes rotary words. This preserves continuous orientation and the existing whole-arc feed cap. Fixed-orientation arcs retain R splitting. Per-segment rotary interpolation was considered and dropped because it requires extra angle and feed state changes that the native arc does not need.
