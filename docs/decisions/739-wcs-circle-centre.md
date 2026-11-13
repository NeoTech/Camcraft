## #739 - WCS page: anchor on a circle's centre, as a reusable pick; tidy the page

**What was decided.**

- The circle is read off the picked edge's **run** (one straight stretch or
  one arc of the tangent chain, `feature::edge_runs`), not the precomputed
  `FeatureGraph::arcs` list. That list is split only at feature points, so
  a filleted rim (lines and arcs tangent to each other) is one walk there
  and would never fit; runs split it into its lines and arcs, so an arc of
  a slot end anchors too. The fit itself reuses `feature_arc::fit_arc` and
  its tolerances, so the pick and the centre snap agree on what is round.
- Refusals are named variants (`CircleRefusal`): straight, too short
  (fewer than three segments), not circular (ellipse, spline, uneven
  polyline - checked against every vertex, never a best-fit guess), not an
  edge (face or vertex), gone (stale pick). Straightness is tested first so
  a straight edge never reads as "not circular".
- The axis sign is canonical (largest component positive), not the walk's,
  so a hole in a top face reads +Z whichever way the rim was walked.
- The command `SetWcsAtCircleCentre` wraps `SetWcsOrigin` (the same apply
  and undo as every other WCS edit) and resolves the circle at creation, so
  a refusal never reaches the history. The MCP tool `set_wcs_circle_centre`
  runs the same command (no follow-up issue needed).
- The page anchors with the pane's current angles (like the anchor grid and
  the vertex chip); the circle's axis is returned for later tools (#738,
  #701) and not used to orient the WCS here.
- A successful circle pick disarms the edge pick, so the viewport goes back
  to picking normally; a refused pick stays armed so the user can re-pick.
- Layout: three collapsible sections (Origin and orientation, Quarter turns,
  Anchor on); a two-column grid for the six fields; quarter turns as two
  rows of three equal buttons plus Level / Set Origin as two equal halves,
  all spanning the pane; the compass as nine equal cells (up to 72 x 30)
  placed at exact positions and centred. The Bottom/Middle/Top radios moved
  out of the label column (as a plain "Z level" row): beside a 120 px label
  they widened the section past a narrow pane and pushed the compass off
  centre.
- `pages.rs` (1894 lines) lost the WCS page to `egui_app/wcs_page.rs`, and
  `state.rs` lost the `SetWcs` dispatch arm to `state_wcs.rs`, so no file
  past 400 lines grew.

**Considered and dropped.** Snapping to the round-edge centre markers
(`PickKinds::CENTRES`) as well as edges - the issue asks for an edge pick,
and a centre pick on a deep bore could land on the far rim's centre;
orienting the WCS Z along the circle's axis - a separate, larger decision
for the alignment work (#738 / #701).

**Cost to change later.** Low: the primitive and command are additive; a
centre-marker pick or axis orientation can be added beside them.
