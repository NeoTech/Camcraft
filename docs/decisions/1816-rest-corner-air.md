## #1816 - Clip rest-pocket concentric rings to cutter reach and re-anchor tip starts

- **Clip waterline offsets of `plan.center` against `cut_zone = plan.rest.offset(r + REST_REACH * 0.5)` instead of feeding closed loops around cleared floor.**
  `plan.center` (`rest.offset(r + 0.05).intersect(own_center)`) correctly defines every tool center that can touch rest stock, so offsetting `plan.center` produces closed rings whose inner legs walk `r + 0.05 mm` away from rest material across cleared floor (and outer ring hole loops on wide wall-stock cases walk pure air). Clipping each offset polyline (`k = 0..`) against `rest.offset(r + 0.025)` keeps pocket walls (`0.025 mm` inside `cut_zone`) and rest-touching passes intact while dropping every cleared-floor inner leg and air hole lap, leaving a `0.025 mm` margin inside `plan.center`'s `0.05 mm` boundary. Pieces are sorted interior-out and chained (`chain_pieces`) only along chords clearing `cut_center = cut_zone.intersect(&plan.center)` without crossing planned pieces.
  *Considered and dropped:* shrinking `plan.center` directly (which would shrink the dive certification region for `Helix`/`Ramp` entries) or switching rest pockets to open zigzag/adaptive lanes (tracked separately in `#1490`, omitted to keep this fix orthogonal).
  *Cost to change later:* localized to `rest_passes` in `crates/motion/src/machining2d/rest_rings.rs`.

- **Re-anchor open pinched-tip first passes into nearby dive room (`anchor_first_pass`) and bound `links_center` to `&cut_center` while keeping `center: &plan.center`.**
  When clipping opens an outer crescent ring, its start point sits at a pinched corner tip where no nearby dive candidate inside `r` stays inside `rest.offset(r + 0.005)`, causing `descent::plan` to plunge/helix into the middle of the crescent and feed across cleared floor to the tip. When no reachable dive point near `first[0]` keeps its dive link within `rest.offset(r + 0.005)`, `anchor_first_pass` selects an anchor on `first` near `dive_room` of its connected component, walks to the tip, and doubles back around the corner without touching cleared floor; closed first passes rotate to the vertex nearest dive room. Passing `center: &plan.center` preserves entry clearance proofs while passing `links_center: &cut_center` prevents keep-down pass links from cutting across cleared floor.
  *Considered and dropped:* changing `descent::plan` globally across all 2D operations (unnecessary blast radius on non-rest pockets) or trimming off corner tips (which leaves wall corner cusps uncut).
  *Cost to change later:* self-contained in `crates/motion/src/machining2d/rest_anchor.rs` and `rest2d.rs`.

- **Extract shared pass emission into `crates/motion/src/machining2d/pocket_emit.rs` rather than duplicating the emission pipeline or adding rest-clip branches to `pocket_over`.**
  Both `pocket` and `rest_pocket` need identical arc fitting, `blend_turns` corner filleting, crossing enforcement, level-walk descent/linking, and refusal checks. Moving emission into `emit_pocket_passes` (`pub(super)`) shares one path with zero behavior change to `pocket` and avoids tangling `pocket_over` with unrelated open PRs (`#2035`, `#2037`).
  *Cost to change later:* single internal module in `crates/motion/src/machining2d/`.

- **Stack on open PR `#1817` (`merge-ready`) while targeting `main`.**
  The fixture (`crates/app/tests/rest_pocket.rs`, `crates/app/tests/golden/pocket_rest_corners.cl`) and `rest_pocket` pipeline live in `#1817`. Stacking cleanly avoids duplicating or conflicting against `#1817`.
