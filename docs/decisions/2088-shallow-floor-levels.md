## #2088 - Include shallow flat floors in the 3D Pocket ladder

**What was decided.** Add distinct heights from upward-facing, exactly flat
triangles, plus `stock_to_leave`, strictly between Top Z and the first
scheduled level below Top Z. Keep the regular levels. Use one helper for
plain, offset-shell, open-side and rest generation. The air-split ladder of
the rotary wrap keeps the plain level law: there a flat face is a turned
skin, and a window whose only open level is the wrap top stays air
(`rotary_offset::the_height_window_names_the_stations_and_refuses_an_empty_one`). The rest pass
uses its own half-stepdown. Keep the existing material targets, protected
areas, entry checks and no-gouge checks. Refuse a level count above
`MAX_PASSES`, while preserving an existing Top Z stock pass.

This change was stacked on PR #2149 because both changes touch the same
functions. PR #2149 merged to main. The branch then took a merge of main.
That merge keeps the shared `cut_levels` helper and main's `AdaptiveMethod`
argument on each ladder call. The protected-face integration is main's own.
Use `issue/2088-shallow-floor-levels` for this change. Preserve the older
`issue/2088-pocket3d-shallow-blind-floor` branch from closed PR #2141.

**Why.** The regular ladder can put its first level below a shallow blind
floor. The material checks then correctly reject every level in that hole.
A level at the floor cuts the hole without entering the solid below it.
Adding levels inside the first interval does not increase regular level
spacing. The existing target checks still decide where each level can cut.

**Considered and dropped.** Restoring a Top Z air pass would leave the
shallow hole uncut. Clamping Bottom Z would change the requested depth for
other areas. Retrying only after an empty result would miss a shallow hole
beside an area that already cuts. General discovery of sloped surfaces
would require a separate algorithm and more safety coverage.

**Cost to change later.** Discovery currently covers exactly flat upward
faces. A wider surface rule needs independent safety tests for slopes,
overhangs, stock allowance and shell motion. Keep the shared helper and
the per-level material checks when extending that rule.

**Validation.** All five blind-floor tests pass. Three mutation checks fail
with the corresponding change removed and pass after restoration. The quick
gate passes 611 steps. Clippy passes for `camcraft-motion`.
