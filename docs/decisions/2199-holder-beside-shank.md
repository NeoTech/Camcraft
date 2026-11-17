## #2199 - Tool body: judge the stated shank below a holder foot drawn beside it

**What was decided**

1. `HolderProfile::enclose` keeps widening `shank_d` when a holder band stands strictly inside `(flute_len, flute_len + shank_len)`. It switches from inclusive `reach_over` to strict interior `span_reach` (`above < hi - 1e-9 && above + len > lo + 1e-9`), so a band touching only `shank_top` from above no longer widens the judged shank (#2180).
2. A new helper `camcraft_tools::neck::shank_with_body(stored, body, neck)` computes the shank bands for `[flute_len, flute_len + shank_len]`. When no body band enters the interior of `(flute_len, shank_top)`, it returns `neck` unchanged. When one does, it splits `[flute_len, shank_top]` across the boundaries of `stored` (or `neck`) and `body`, taking `max(base_r, body_r)` on each sub-span.
3. `operation::tool_gate::job_tool` and `fixture_struck::op_hits` call `shank_with_body(&stored, body.as_ref(), neck)` using the un-enclosed `stored` tool record alongside the judged `(tool, body)` pair.
4. `HolderStack::shank_bands` uses the same `- 1e-9` interior slack on `beside` so floating-point rounding at a holder foot boundary cannot smear the holder radius into the shank band beneath it.

**Why**

Single-cylinder fallback laws (`verify::stock_top`, `verify::traverse_solids`, `drill_reach`) read only `ToolGeom::shank_radius`, and `BodyBands::of` checks body bands against `ToolGeom::shank_radius` beside the shank. Keeping `HolderProfile::enclose` conservative on `shank_d` preserves every safety bound on those single-cylinder readers while `NeckBands` carries the stepped `[flute_len .. holder_foot @ stored_shank_r, holder_foot .. shank_top @ holder_r]` profile to `pocket_reach`, `verify`, `post`, and workholding fixture checks.

**Considered and dropped**

- Dropping the `beside` widening in `HolderProfile::enclose`: unsafe for single-cylinder readers and breaks `holder_profile::tests::the_enclosing_cylinder_never_shrinks_the_body`.
- Extending `BodyBands::of` below `shank_top` instead of splitting `NeckBands`: forces every `BodyBands` caller (including `drill_reach::stack_bands`, `body_judged::falls_back`, `probe_op::stylus_cutter`, and `sim_tool_body` while #2179 is open) to reinterpret bands below `shank_top`, and would still need `NeckBands` when a holder collar ends at or below `shank_top`.

**Cost to change later**

Low and local. Only `tools::holder_profile`, `tools::neck`, `app::operation::tool_gate::job_tool`, and `app::fixture_struck::op_hits` participate. No project schema or stored data format changed.
