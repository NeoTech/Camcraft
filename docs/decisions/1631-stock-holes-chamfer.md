## #1631 - A stock-source 2D Chamfer, Drill, Bore or Thread answers for the parts in its stock

**Decided:** #1607's stock-source part law (`keep_law::parts_in_stock`,
the setup's parts in its frame - fixtures and a stock-only model left
out) now also covers the four kinds #1607 left at `DepthLaw::None`
(`keep_law::is_stock_kept_kind`):
- **Drill (no cross holes), Bore, Thread** (`keep_law::is_hole_kind`):
  the **containment law** they answer to on a part source
  (`contain_into` with `DepthLaw::Contain`) against the setup's parts.
  It excuses exactly the round wall of each hole the operation
  declares, measured about that hole's own axis and no deeper than the
  facets' sagitta (#1205, `verify::hole_wall`), and a thread mill's
  declared bore (#1071, `thread_op::bores`). Nothing else: a hole where
  the part models none, wider than the part's hole, or deeper than its
  floor puts the drill, bar or mill in the part's solid - a blocking
  `Warning::Gouge` on the move.
- **2D Chamfer**: the **solid law** (`contain_into` with
  `DepthLaw::Keep`, nothing excused) against the setup's parts.

A setup with no part judges nothing, as before. The source still
decides the path; only the verdict changed. Both frame arms of `run_job`
carry it (the 3+2 arm in the plane's frame).

**Why the containment law for the hole kinds, not a new one:** a hole
off the stock is legal in the part only where the part has that hole -
the very question `Contain` asks of a part-source drill: its declared
hole's modelled rim is the finished wall, anything else is steel the
part keeps. Re-using it means a stock-source drill into the part's own
hole reads exactly as the same drill on the part, and a hole through
stock-only material meets no part facet at all. The solid law alone
(nothing excused) was measured as the alternative (mutation M7 below):
it convicts a drill or bore at the size of a modelled 32-gon hole,
whose flats stand 0.012-0.024 mm inside the circle - a false refusal
the containment law exists to avoid.

**Why the solid law for the Chamfer, not a mark-style budget:**
- On a part source the Chamfer's cut is legal by construction: the
  edge it breaks is the part's own (its depth is derived from width and
  angle, the program sim credits the declared break, #303). Off the
  stock the edge it follows is the STOCK's. Where that edge stands
  clear of every part the cone meets no part facet and the solid law
  finds nothing; where it lies on or inside a part the break is cut
  into the part with no part edge that declared it.
- A mark-style budget (lower the part by the chamfer depth under the
  cone, #1595's shape) would grant a V-groove of that depth anywhere a
  stock edge crosses a part's face - the very gouge #1631 names - and
  the depth is a derived number, not one the user stated as permission.
- The cost is a false refusal in one case (below), with a workaround
  that is the right source anyway.

**Readings** (measured with a probe on c74d888d and on this branch; 30 x 30 x 20 plate, one modelled round hole at the origin;
square box stock flush with its top, aligned and turned a quarter about
Z; stock-source op "S1"):

| case | before: op gouges, aligned / turned | now: op gouges | now: aligned / turned |
|---|---|---|---|
| Drill Ø6 at (0, 9), no hole there | 0, refused by the sim / **posts** | yes | refused / refused by the op's moves |
| Drill Ø6 at the r 3 hole, 3 mm under its floor | 0, refused by the sim / **posts** | yes | refused / refused |
| M6 tap at (0, 9), and 3 mm past the Ø5 hole's floor | 0, refused by the sim / **posts** | yes | refused / refused |
| Bore r 5 at (0, 9); Bore r 6 in the r 5 hole | 0, refused by the sim / **posts** | yes | refused / refused |
| Thread M8.8 x 1 at (0, 9.5); 4 mm under its hole's floor | 0, refused by the sim | yes | refused / never generates (#992, below) |
| 2D Chamfer 1 mm on a 30 x 30 stock flush with the part | 0, **posts** (the sim credits the declared break) / **posts** | yes | refused / refused |
| Drill, tap, bore, thread at the part's hole to size; the same at (25, 0) in stock-only material; Chamfer on a 70 x 70 stock | 0, posts | 0 | posts / posts (thread: aligned only) |

**The tap's core re-read (#266) now reads every part law.** A Tap
cycle's gouges are re-judged at its thread core and any the core run
does not raise are dropped. That run read only the source's own
`Contain` law, so the parts a tap answers to beyond its source - the
stock-source parts here, and #1621's neighbours on a part source - were
acquitted unread: a stock-source M6 tap into solid part posted, and a
part-source tap fed through a neighbouring lid that models no hole
carried no gouge (measured before this change, mutation M5). The core
run now applies the same part laws as the full-diameter run.

**Behaviour to know - the flush-stock chamfer is refused.** A box stock
exactly the part's footprint, chamfered off the stock, breaks the part's
outer edge; the program sim credits that declared break (it does not ask
which source declared it), so on an aligned stock it posted before. It
is now a gouge on the operation. Chamfer the part's edge with the part
as the source - the same path, legal there by construction.

**Thread on a turned stock (not changed here).** A stock-source Thread
never generates on a turned stock: #992's way-in law reads the program
ahead on the column grid, which cannot model the turn, and refuses the
thread by name ("is not drilled yet ... or the program ahead cannot be
read"). That is a named refusal, so the into-part case is safe there;
the legal case cannot post on a turned stock either, part source
included. Pinned as it stands; not this issue.

**Considered and dropped:**
- The cross drill (a Drill with cross holes): its chain answers to
  `cross_part` against its source and its moves lie along each hole's
  own axis; the containment law here excuses vertical walls only. Left
  as it is (follow-up filed).
- Refusing every stock-source hole op that enters a part: drilling the
  part's own hole off the stock is a common, legal program.
- Matching the declared hole against recognised part holes
  (`detect_holes`) and refusing a mismatch by name: a second law that
  could disagree with the solid one; the containment law already reads
  diameter and depth against the mesh itself.

**Pins** (`crates/app/tests/stock_source_holes.rs`; aligned and turned
stock unless said):
- `a_hole_kind_off_the_stock_into_the_part_gouges_and_never_posts`
- `a_hole_kind_in_the_parts_own_hole_or_in_stock_only_material_posts`
- `a_chamfer_off_a_stock_flush_with_the_part_gouges_and_one_clear_of_it_posts`
- `a_tap_through_a_neighbouring_part_is_not_acquitted_at_its_core`
  (part source, #1621's neighbour)
- `a_stock_source_hole_or_chamfer_with_no_part_in_the_setup_judges_nothing`
- `over_mcp_a_stock_source_hole_or_chamfer_answers_for_the_part_the_same`
  (MCP `add_operation` - whose default source is the stock - `generate`,
  `verify`; aligned stock, MCP's `set_stock` takes no rotation)

| mutation (each alone, then reverted) | red |
|---|---|
| M1 the four kinds out of `is_stock_kept_kind` | into-part, chamfer, MCP |
| M2 the hole kinds judged by the plain solid law (no containment branch) | into-part (the tap acquitted), legal, MCP |
| M3 Chamfer out of the set only | chamfer, MCP |
| M4 the stock-source parts out of the tap's core re-read | into-part (tap cases) |
| M5 #1621's neighbours out of the tap's core re-read | tap-through-lid |
| M6 no thread bore granted against the parts | legal (thread in its hole) |
| M7 `Keep` (no wall excuse) instead of `Contain` against the parts | legal, MCP |

**Not exercised:** the 3+2 frame arm with a stock-source hole or
chamfer (same code shape as #1607's, which `tilt.rs` covers only with
no stock); the cross drill (above).

**Goldens:** none change.

**Cost to change later:** one predicate (`keep_law::is_hole_kind`, and
`Chamfer2D` in `is_stock_kept_kind`), one branch per frame arm in
`run_job`, and two lines in the tap's core re-read.
