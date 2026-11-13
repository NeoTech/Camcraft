## #544 - Sinker EDM: orbit patterns (circular, vector, planetary) as explicit moves

**Decided:**
- **No new machine schema.** The `.mch` `edm` section from #530 already states what the orbit law needs: `edm.orbits[]` (`kind` = one of `circular` / `vector` / `planetary`, plus the control's `word`) and `edm.max_orbit_radius` (mm). #1141 (#542, the sinker post) reads the same fields for its words. This PR reads them as they are and adds no key.
- **The orbit is an optional field of the Sinker Burn** (`SinkerBurnOp.orbit: Option<SinkerOrbit>`): pattern, radius, `vectors` (vector only, default 4) and `turns` (planetary only, default 1). `None` is #541's plain plunge, unchanged, with the same CL and the same params digest. The file key `orbit` is additive, so older files load. It goes through the existing params path (`CreateOperation` / `EditOperationParams`), and MCP reaches it through the params JSON door (`add_operation` / `edit_operation` with `"orbit": {...}`, or `null` to drop it). No new command and no new MCP tool.
- **The orbit runs at the burn floor, after the plunge**, at the servo feed. All of it is explicit Feed chords (#290 D3), and it returns to the centre before the rapid retract:
  - circular: out along +X to the radius, then one full circle;
  - vector: out to the radius and back along `vectors` evenly spaced directions, starting at +X;
  - planetary: an Archimedean spiral from the centre to the radius over `turns` turns, closed by one full circle.
- **No overshoot, by construction.** Every vertex lies on or inside the circle of the stated radius (a rounding that would land one ulp beyond is pulled back in). A chord between two points of a disc stays in the disc. The chords per turn keep the sagitta under 0.001 mm, and their count is a multiple of four, so the ±X / ±Y directions are vertices and a rectangular electrode's walls move out by exactly the radius.
- **The machine law, refused by name at generation** (`sinker_op::orbit_law`): a pattern the machine does not list under `edm.orbits`; a machine that states no `max_orbit_radius` (absent is not unlimited); a radius above it. A radius equal to the maximum is admitted. The job and the staleness stamp read it through `burn_gap` (gap and orbit law together), so a re-stated orbit list or maximum marks the op stale. A garbled orbit (a non-positive radius, 0 or more than 72 vectors, 0 or more than 20 turns) is refused at the door (`is_sane`), and the generator says the same by name.
- **Verification** is #541's own: the orbit's chords are straight Feed moves, so the box atom (`simulate_box`) sweeps each one, corner joins included, and the program verdict convicts an orbit wider than the part's cavity as a gouge. No new sim law.
- **Posting is left to #542.** This PR emits no `EdmOrbit` PARAM. The orbit posts as the explicit moves it is. A canned orbit word would be motion the sim never saw. Posting it compactly is D3's later sub-issue, and only after the expanded path verifies clean.

**Why:** #290 D3 and D6 (the Prime Directive): what posts is what the sim verified, and a pattern or radius the machine does not state is refused instead of guessed.

**Considered and dropped:**
- G2/G3 arcs for the circle. The box atom refuses arcs under a rectangular electrode (#541), and an arc posted to a control that orbits in its own way is the dialect's business.
- Orbiting while sinking (a helical or 3D orbit). Public texts describe both, but the floor orbit is the plain case that "Done means" asks for, and it keeps the depth laws of #541 exact. It can be added later as a new pattern.
- A start angle for the vector pattern. It is one more field with no stated need. Directions start at +X.
- Emitting `PARAM EdmOrbit` / `EdmOrbitRadius` (which #1141's post reads). That would post the control's canned orbit on top of the explicit moves, a double orbit.

**Cost to change later:** low. The orbit is one optional field and one motion module (`motion::sinker_orbit`). A new pattern appends to `OrbitKind`, and to `ORBIT_KINDS` in the post crate.
