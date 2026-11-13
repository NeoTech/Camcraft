## #543 - Sinker EDM: rough and finish with undersize (multi-electrode)

**Decided:**
- **Sibling ops, not a pass list.** A rough/finish pair is two Sinker Burn ops of one cavity, each mounting its own electrode the normal way. Every Sinker Burn gains `pass` (`single` / `rough` / `finish`, default single = #541's law unchanged) and the finished cavity's `cavity_x` / `cavity_y`. One op stays one electrode, so the tool change, the stock sim's box atom (one tool per planned op), the post (#542) and the tool law carry multi-electrode work without a second tool slot per op.
- **The undersize comes from the condition.** A rough or finish reads its condition's `undersize` (and `gap`) through #531's `Edm::condition(machine, id, need)` by id, after the sinker-kind check (`sinker_op::condition`, now the one door the gap and the undersize share). The law per axis (`motion::sinker_undersize::judge`), electrode `e`, cavity `c`, undersize `u`, gap `g`:
  - `e + 2g > c` is refused by name for either pass: the wall would be burned past size (a gouge).
  - for a finish, `e + 2g < c` is refused by name: "sinker finish leaves the cavity under size".
  - `e` must be `c - 2u` (within a micron): the electrode the condition calls for. A mismatch is refused by name - a wrong electrode in the holder is the likely mistake.
  - the tip stops `u` above the finished floor (the electrode stands `u` inside the finished surface on every side, its face included), so a rough leaves `u - g` on the floor as on the walls. `SinkerParams.stand` carries it; `None` keeps #541's one-gap stand.
- **The order law is a warning, by name.** A finish looks back (program order, same setup, not suppressed) for the nearest rough of the same cavity at the same burn point. If the finish's undersize is not smaller than that rough's, `Warning::SinkerFinishOrder` names both numbers. It is advisory (does not hold the post): the finish's own size law already holds the cavity to size, so the inverted pair is a process mistake, not a gouge. Read "the finish electrode is not smaller than the rough" as "its undersize is not smaller" - a correct finish electrode is physically the larger one.
- **Staleness:** the op's stamp digests its condition's undersize and, for a finish, the rough it follows (name and undersize), so re-stating a condition or adding/removing/re-sizing the rough marks the finish stale. A single burn's digest does not move.
- **File and MCP:** additive DTO keys (`pass`, `cavity_x`, `cavity_y`), written only for a rough or finish, so #541 files read unchanged. `add_operation` / `edit_operation` / `generate` / `undo` drive the pair through the same undoable commands (pinned in `sinker_undersize.rs`); no new MCP tool.
- **UI:** the Sinker Burn page's Passes tab gains "Electrode pass" and the finished cavity X / Y (the full sinker pages stay #548's).
- **Verification:** the program verdict (`simulate_document`, the box atom of #541) pins the pair: after the rough the cavity is not clean (stock stands), after the finish it is clean (no rest, no gouge). Golden: `tests/golden/sinker_rough_finish_pair.cl` (rough then finish). Synthetic conditions only; not exercised on a real EDM control.

**Why:** #290 D4/D7. The undersize is condition data, so the electrode is checked against it rather than trusted; the size refusals are the Prime Directive's (an overburn gouges, an under-size finish is a silently wrong part).

**Considered and dropped:**
- A pass list (electrode + condition per pass) inside one op. Every consumer (tool law, stock sim plan, post tool changes, stamp) assumes one tool per op; a second tool slot would have touched all of them.
- Refusing the inverted order. It is not unsafe once each pass's size law holds, and the issue asks for a warning.
- Letting the electrode's measured size define the undersize (ignoring the condition's). The issue takes the undersize from the conditions; a mismatched electrode is exactly what the check should catch.
- Applying the undersize to Z through a separate floor stock parameter. One per-side undersize on every face keeps the floor and walls finishing together, as the issue states "per side".

**Cost to change later:** low. `pass` and the cavity are additive keys; an orbiting finish (#544) changes only the `e + 2g` term to `e + 2(g + orbit)` in `judge`.

### Refresh over main (#1131, #544 orbit, #545 side burn merged), 2026-11-12

**Decided:**
- **Main's tip refusals read the stand-off.** #1131 added two refusals on `tip = z + gap` (tip at or above the approach plane; gap not smaller than the burn). This PR places the tip at `z + stand(p)` (the undersize stand-off for a rough/finish), so both refusals now read `stand(p)`, naming it "undersize stand-off" when one is set and keeping main's "spark gap" text otherwise. Since the stand is never less than the gap, this only refuses more. The side burn's tip uses the same `stand(p)`.
- **One condition door.** Main's `condition_gap_as(label)` (#545) and this PR's `condition(need)` became `condition_as(world, setup, label, op, need)`; `condition_gap`, `condition_gap_as` and `condition` are thin wrappers, so every refusal text stays as both sides had it.
- **Validate early (NeoTech review at b93e8fbe):** `SinkerBurnOp::pass_refusal` refuses a rough or finish without both finished cavity sizes above 0 by name at `CreateOperation` / `EditOperationParams` (the sinker gate), and `is_sane` repeats it; generation runs it again for a file-loaded op.
- **No orbit on a rough or finish yet.** `judge` sizes the burn as `e + 2g`; an orbit widens it by `2r`. Rather than judge an orbited pass wrongly, it is refused by name (creation and generate), and #1355 owns counting the radius. A single burn orbits as #544 shipped.
- **The side burn's burn is a single pass.** It sizes its electrode by its own undersize (#545), so a rough/finish pass on it is refused by name.

**Considered and dropped:** counting the orbit in `judge` now (it also moves the floor stand - design work for #1355, not a merge); splitting `job.rs` (main is already at 415 lines; this PR adds 8, a split is not trivial and belongs to its own refactor).

### Refresh over main (#1157 wear compensation, #546), 2026-11-13

**Decided (coordinator M5, posted on PR #1136):**
- **Wear compensation reads the stand-off.** #546's `burn_compensated` placed every layer's tip at `top - d + gap - r*d` and ignored `SinkerParams::stand`. Calling it from a rough or finish would burn to floor + gap and silently through the finish allowance. Both `burn_compensated` and `deepest_tip` now read `sinker::stand(p)` (`s`) where they read the gap.
- **Compensate the depth actually burned.** `tip = top - d + s - r*max(0, d - (s - gap))` and `deepest = z + s - r*(top - z - s + gap)`. Burning starts one gap above the top, so this is exactly where `worn_tip` (the stock sim's as-burned reading) puts the worn face, on floor + s. The wear bound and the electrode-length check read the same deepest tip. With no stand-off set, `s` is the gap and every move is byte-identical to main.
- **One door.** The law lives in motion's `burn_compensated`; `generate_sinker` runs `pass_refusal`, `wear_refusal`, the undersize judgement, then `burn_compensated`.
- **Why:** the planned burn is the burn the sim verifies, and no burn silently enters the allowance a later pass is meant to remove.

**Considered and dropped:** keeping `r*d` (over-deepens by `r*(s - gap)`, which goes into the finish allowance and away from the sim's reading); a separate app-side check (two doors to keep in step on burn depth).
