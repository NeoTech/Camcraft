## #1635 - A stock-source cross drill answers for the parts in its stock

**Reproduced first; not a P1.** #1635 asked whether a stock-source cross
drill (a Drill with cross holes, source Stock) that enters a part posts
unjudged. It does not: `cross_part::part_clear` read the source's mesh,
which off the stock is the stock itself, so EVERY stock-source cross
drill was refused at generation - into the part and legal alike - with
"cross hole ... rapids into the finished part: the finished part stands
in the drill's path". The stock was read as the solid to keep. Nothing
reached the post; the defect is a false refusal of legal programs (and a
misleading message), P3 by the AGENTS.md table. The issue keeps its P1
label until the maintainer lowers it (agents only raise).

**Readings** (measured on 881f7726 and on this branch; the #117 punched
bar, r10, z [-80, 20], one r2 hole plunging -X from (10, 0, 0) 6 mm deep;
DR-0400 long series; stocks: an r20 bar and a 40 x 40 box over the
bar's span, each aligned and turned a quarter about Z; mill-turn and
lathe test benches; stock-source op "S1"):

| case | before (all four stocks) | now |
|---|---|---|
| hole where the bar models none | refused: "rapids into the finished part" (the stock) | refused: "cuts into the finished part: the drill meets steel at the mouth - the model has no hole there" |
| hole into the bar's modelled hole | refused, the same words | generates the part source's chain move for move; posts |
| hole in stock-only material (r3 core) | refused, the same words | generates; posts |
| part source, the same holes | into-part refused, legal posts | unchanged |
| lathe, box stock, legal hole | refused: "this part cannot be read as turned" (the box) | generates |

A first cut that only swapped the judged mesh (the parts instead of the
stock) still refused the legal holes, for a rapid into stock: with the
stock as the source the standoffs were measured about the stock as if
it were the part, so the mouths stayed at the declared r10 inside the
r20 raw bar. Hence the whole reading below, not only the judged mesh.

**Decided:** a stock-source cross drill is the chain a part-source cross
drill drills in the same raw stock (`cross_spindle::Solids`):
- the setup's parts (`keep_law::parts_in_stock` - fixtures and a
  stock-only model left out; `keep_law::is_cross_drill` joins
  `is_stock_kept_kind`) are the solid it measures (`source`) and keeps
  (`kept`, judged by `cross_part` - each declared cross hole's round
  wall excused about its own axis, nothing else);
- the source stock is the raw material (`stock`) its mouths stand off
  and its rapids are proven clear of (#77, `cross_stock`);
- on a plain lathe, the parts are the turned work `lathe_reads` reads
  (a box raw stock is not a turned part).

A setup with no part keeps nothing (`part_clear` returns early on an
empty mesh); the mouths still stand off the stock.

The vertical-wall laws of the 2D tail (#1607's solid law against the
setup's parts) skip a cross drill (`parts_crossed`): the radial law
already judged its chain against the same parts, and the solid law
with no wall excuse convicts a drill at the size of a faceted cross
hole (measured: mutation M5).

**Why the part source's law, not a new one:** the issue's "Done means"
and #1631's reasoning - a hole off the stock is legal in the part only
where the part has that hole, which is the question `cross_part` asks
on a part source. Reusing it makes a stock-source cross drill read
exactly as the same drill on the part (pinned: the chains are equal).

**Considered and dropped:**
- Judging the parts but keeping the stock-as-source standoffs: still
  refuses every legal hole (rapid in stock, M3).
- `contain_into` with `DepthLaw::Contain` against the parts (#1631's
  hole-kind law): it excuses vertical hole walls only (`HoleAxis`
  vertical); a cross hole's wall runs along X or Y.
- Refusing every stock-source cross drill by name: the legal program
  (drill the part's own cross hole off the raw bar) is common.

**Pins** (`crates/app/tests/stock_source_cross_drill.rs`):
- `a_stock_source_cross_drill_into_the_part_is_refused_by_name_on_both_stocks`
- `a_stock_source_cross_drill_into_the_parts_own_hole_or_stock_only_material_posts`
  (and equals the part source's chain)
- `a_stock_source_cross_drill_at_the_modelled_holes_size_is_excused_its_faceted_wall`
- `a_stock_source_cross_drill_with_no_part_in_the_setup_keeps_nothing`
- `a_lathe_reads_the_turned_part_in_a_box_stock_not_the_box`
- `over_mcp_a_stock_source_cross_drill_answers_for_the_part_the_same`
  (built-in Mill-turn, MCP's default Stock source, aligned stock only:
  MCP's `set_stock` takes no rotation)

M1-M4 and M6 were run before the faceted-wall pin was written (which
M5 then needed); their red lists name the five pins of that run.

| mutation (each alone, then reverted) | red |
|---|---|
| M1 the old reading (source = kept = the stock, no raw stock) | all five then written |
| M2 no part kept (`kept` empty) | into-part, lathe, MCP |
| M3 parts judged, standoffs still off the stock as the source | legal, no-part, lathe |
| M4 `is_cross_drill` out of `is_stock_kept_kind` | into-part, lathe, MCP |
| M5 the 2D tail's solid law also applied to the cross chain | faceted wall |
| M6 the lathe reads the box stock | lathe |

**Not exercised:** the 3+2 frame arm's `parts_crossed` guard (a cross
drill is lathe / mill-turn work and does not reach that arm); a posted
program on a plain lathe (the test bench mounts no turning post).

**Not changed - follow-up:** a part-source cross drill still answers
only to its own source, not to #1621's neighbouring parts
(`depth_law_of` is `None` for it, so `neighbour_parts` is empty).

**Goldens:** none change.
