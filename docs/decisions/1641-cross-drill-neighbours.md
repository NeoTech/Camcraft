## #1641 - A part-source cross drill answers for the setup's other parts

**Reproduced first; a real P1.** #1621's neighbour law
(`keep_law::neighbour_parts`) judged a part-source op against the setup's
OTHER parts only for kinds whose `depth_law_of` is not `None`; a Drill
with cross holes reads `None` (its part law is the radial `cross_part`,
not a `DepthLaw`), so the set was empty for it. The program sim does not
judge a posed (radial) chain, so nothing else caught it.

**Readings** (on da340c0f, #1640's head, before the fix; mill-turn test
bench, long-series DR-0400; B: the #117 r10 punched bar, z [-80, 20], its
r2 cross hole at (10, 0, 0) plunging -X 6 mm modelled - legal for B; A: a
box; stocks: an r20 bar and a 40 x 40 box over B's span, aligned and
turned a quarter about Z; op on source "B"):

| case | before (all four stocks) | now |
|---|---|---|
| A x [12, 18], y/z [-5, 5]: the drill runs through A, which models no hole | generates clean, **posts** | refused: "cross hole at (10, 0, 0) along (-1, 0, 0) cuts into a neighbouring part: another part the setup holds stands in the drill's line or its approach ..."; nothing written |
| A x [21, 24] outside the r20 raw bar: only the approach / retract crosses it | (not read before; same law) | refused by name, aligned and turned |
| A a tube on the line, bore r3 as a 32-gon, B's r3 hole drilled at size (DR-0600) | (not read before) | refused: A's bore is not a hole this op declares |
| A on B's far side, or 2 mm over the r2 drill on the hole's side | posts | posts |

**Decided:**
- `keep_law::kept_models` (hence `neighbour_parts`) includes a cross
  drill (`is_cross_drill`) on a part source: the setup's parts but the
  source, fixtures and a stock-only model left out - #1621's set.
- `cross_spindle::Solids` carries those parts as `neighbours`; after the
  source's own `cross_part::part_clear`, the finished chain (world frame,
  on and off the work offset alike) is swept against them by
  `cross_part::neighbours_clear` - the same radial solid law, posed along
  each move's tool axis, drill, shank, holder and rapids, with NO hole
  wall excused. A hit refuses the op at generation, naming the hole by
  the line the offending move rides.
- A stock-source cross drill passes no neighbours: it has no source
  part, and #1635 already judges every part in its stock.

**Why nothing is excused on a neighbour:** the declared cross holes are
the source's holes. A round wall A models on the same line is A's
steel to this op, and the source part's wall excuse must not leak onto
another part (mutation M3 below is that leak; the tube pin catches it).
The conservative cost: a drill meant to pass through a neighbour's
existing bore is refused - drill those parts in setups of their own.

**Why the radial law, not #1621's tail law:** the tail's `contain_into`
reads the 2D kinds' vertical-wall excuses and `DepthLaw`s; the cross
chain's own law (`cross_part`) already sweeps the posed drill and names
the hole. The tail's neighbour law is left ON for a cross drill as a
second net (no wall excused there either): dropping its use for a cross
drill turned no pin red (M4), so a guard would be unpinned code; a chain
that reaches the tail has already passed the radial check.

**Considered and dropped:**
- Folding the neighbours into `kept` (one `part_clear` call): the source's
  declared hole walls would be excused on the neighbours too.
- Making `depth_law_of` return a law for a cross drill: it would switch
  on the vertical-wall containment laws that #1635 measured convicting a
  faceted cross hole's own wall.
- Leaving it to the program sim: it refuses to judge a posed chain, on
  both stocks - the defect itself.

**Pins** (`crates/app/tests/cross_drill_neighbours.rs`):
- `a_cross_drill_on_b_into_part_a_is_refused_by_name_on_both_stocks`
- `a_cross_drill_whose_approach_crosses_part_a_is_refused_on_both_stocks`
- `a_neighbours_round_hole_on_the_line_is_not_excused`
- `a_cross_drill_on_b_clear_of_part_a_posts_on_both_stocks`
- `over_mcp_a_part_source_cross_drill_answers_for_the_neighbour_the_same`
  (built-in Mill-turn, `add_operation` with `"source": "B"`, aligned
  stock: MCP's `set_stock` takes no rotation)

| mutation (each alone, then reverted) | red |
|---|---|
| M1 cross drill out of `kept_models` (the old predicate) | into, approach, tube, MCP |
| M2 `neighbours_clear` not called | into, approach, tube, MCP |
| M3 the source's hole walls excused on the neighbours | tube |
| M4 the 2D tail's neighbour law skipped for a cross drill | none (kept on as a second net; not a guard) |
| M5 the part-source arm passes no neighbours | into, approach, tube, MCP |

`stock_source_cross_drill.rs` (#1635's pins) stays green under each.

**Not exercised:** a plain lathe with two parts (the lathe's turned
reading of a two-part setup is its own question); the 3+2 frame arm (a
cross drill is lathe / mill-turn work).

**Not changed - follow-up candidate:** a stock-source cross drill judges
every part with the declared holes' walls excused about their axes
(#1635), so a part's round bore on another part's hole line is excused
there; on a part source it no longer is. Left as #1635 decided.

**Goldens:** none change.
