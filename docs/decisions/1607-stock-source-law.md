## #1607 - A stock-source 2D operation answers for the parts in its stock, whatever its kind

**Decided:** #1603's stock-source keep law
(`operation::keep_law::parts_in_stock`, the setup's parts as the program
sim judges them: fixtures and a stock-only model left out) now covers
every 2D kind that carries a part law on a part source
(`keep_law::is_stock_kept_kind`):
- the pocket family (2D Pocket, Adaptive, Offset), as before: the solid
  law;
- Slot, Trochoidal slot and Circular (`authored_law` on a part source),
  Contour and Face (`Section` on a part source): the solid law
  (`contain_into`, nothing excused) against the parts;
- Engrave and Trace: the mark law (`mark_law::mark_into`) against the
  parts, with the same budget, `min(Top - Bottom, cutter radius)`, and
  the program sim credits that mark (`mark_law::budget_of` no longer asks
  the source to name the part).

A cutting move that enters a part is a blocking `Warning::Gouge` on the
operation, so the post is refused by the op's own moves on a turned stock
too. A setup with no part judges nothing, as before. The source still
decides the PATH (the stock is the material to remove); only the verdict
changed.

**Why:** a stock-source op had `DepthLaw::None` (`depth_law_of` returns
early when `part_solid` is false), so a cut into the part held in that
stock had 0 findings: with the stock aligned the program sim held the
post, with it turned the sim refuses to judge and the post went out.
Measured on the #1585 fixture (40 x 40 x 8 block, R15 pocket to Z4, 6 mm
end mill) in a 40 x 40 x 8 box stock over it:

| stock-source case | before: op gouges, aligned / turned | now: op gouges | now: aligned / turned |
|---|---|---|---|
| Engrave / Trace Top 8, Bottom 1 (the part's outside wall), Stock or the stock's top picked | 0, refused by the sim / **posts** | yes | refused / refused by the op's moves |
| Slot, Trochoidal slot, Circular in the pocket, 3 mm under its floor | 0, refused by the sim / **posts** | yes | refused / refused by the op's moves |
| Face to Z 5 (3 mm into the part's top) | 0, refused by the sim / **posts** | yes | refused / refused by the op's moves |
| Engrave / Trace 0.3 mm deep along the part's outline | 0, **refused by the sim** / posts | 0 | posts / posts |
| Contour round the stock to Z 1, Slot in the pocket to its floor, Face of a 10 mm stock to the part's top (Z 8) | 0, posts / posts | 0 | posts / posts |

So the two judges now agree on both stocks. The legal mark row was a
false refusal on the aligned stock before (the sim judged a stock-source
mark against the sharp model); it now posts on both.

**Why the mark law for Engrave and Trace, not the solid law:** a line
taken off the stock's outline or a pick of its top runs ON the part's
surface wherever the stock is flush with the part - exactly a whole-model
engraving of the same line. The solid law would convict every such mark,
the mark law reads it the same as #1595 does on the part. The sim's
credit follows the op's law, so a legal mark posts on both stocks.

**Tests that changed, and why:** each asserted the old "a stock source is
never judged" contract on a cut that does enter the part held in the
stock (a from-model stock of the part, the part still in the setup):
- `engrave_law.rs`: `a_cut_on_the_stock_is_never_judged_by_the_mark_law`
  is now `a_cut_on_the_stock_answers_to_the_mark_law_against_the_part`
  (7 mm down the part's wall: convicted, the budget named); in
  `the_program_sim_credits_the_mark_and_nothing_past_it` the stock-source
  0.3 mm mark is now credited (0 gouge cells) and a deep one is charged
  past its budget like the whole model's.
- `trochoid_law.rs`: `a_trochoid_on_the_stock_is_never_judged` (#1593's
  pin, which #1607 asked to keep or replace by a stated decision) is now
  `a_trochoid_on_the_stock_answers_for_the_part`: this decision replaces
  it.
- `circular_pick_law.rs`: `a_circle_picked_off_the_stock_is_never_judged`
  is now `a_circle_picked_off_the_stock_answers_for_the_part`.
- `circular_entity_law.rs`: `a_stock_source_still_cuts_the_typed_circle_unjudged`
  (a circle through the hole's wall off the Stock) is now
  `a_stock_source_circle_through_the_hole_wall_answers_for_the_part`.
- `slot_picks.rs`: `with_nothing_picked_the_typed_endpoints_rule_as_before`
  still asserts the typed slot is cut as typed, through the pocket walls;
  it now also asserts that cut is a blocking gouge.
- `posting.rs` (`a_gouging_program_is_refused_by_the_verification_gate`)
  and `stock_sim_document.rs` (`faced_doc` for the 20 mm model): a
  stock-source Face below the part's top was asserted to generate
  silent so the program sim was the only net. It is now the op's own
  gouge too; the sim's verdict and the post's refusal (led by the sim's
  clause on the aligned stock) are asserted unchanged.

**Every part, not one.** The law reads every part the setup holds
(`setup_align::parts_facets_in`), so a stock-source cut is judged
against each of them: pinned with two parts in one stock
(`a_stock_source_cut_answers_for_every_part_in_the_stock`, a cut into
either part convicted). A part-SOURCE op entering a neighbouring part is
#1621, decided below on the same branch.

**Correction to #1543 and #1593:** both say a Stock source, or a pick off
the setup's stock, "still cuts unjudged". Since #1607 it is judged against
the setup's parts; `authored_law`'s `Stock -> None` arm still stands (the
source is not part-solid, so `depth_law_of` never reaches it).

**Considered and dropped:**
- `uncovered()` refusing a None-law op when the sim cannot judge: the
  #1595 decision's reasons hold (it would refuse a legal stock-source
  Face on any turned stock, and cannot name the move).
- Making the stock source part-solid (`Section` against the stock's own
  mesh): the stock is the material to remove, and its own solid would
  convict every legal cut (the reason `part_solid` exists).
- `Section` against the parts instead of the solid law: the plane law
  reads tangency against the op's region, which here is the stock's, not
  the part's; the solid law is the test `Section` ends in (b2), against
  the real cutter and mesh - #1603's reasoning, unchanged.
- The 2D Chamfer and the hole kinds (Drill, Bore, Thread) on a stock
  source: not in #1607's list. The Chamfer's depth is derived and the sim
  credits it as a break only on the part; the hole kinds would need the
  modelled holes to excuse their walls. Left at `None` (follow-up below).

**Known cost (conservative side):** as #1603: the parts' law tightens by
the stock's fidelity excess, not each part's; a ring finished to size
against a coarsely tessellated wall is convicted once the chord sag
passes the gouge tolerance.

**Goldens:** none change (`golden_cl` and `solid_law_goldens` pass).

**Cost to change later:** one predicate (`keep_law::is_stock_kept_kind`)
and one branch in `run_job` (mark or solid law for `keep_parts`).

**Merged with #1615's review fix (4f05e9c0).** #1615 bounded a mark by
the part under the cutter (`MARK_FLOOR`, `verify::seat_skin`) and judged
the shank, holder and rapids against the part as it stands
(`verify::stack_hits`); `mark_into` now returns `Marked` and the note is
`mark_law::notes`. Both halves apply to a stock-source Engrave or Trace:
the merge passes whichever mesh the mark law reads (the source part, or
on a stock source the setup's parts) to the one `mark_into`, so the
floor and the stack check come with it. Pinned:
`mark_floor.rs::a_stock_source_cut_through_the_plate_meets_the_same_floor`
(2.9 mm and 1.9 mm off the stock through a 2 mm plate refused by name
on both stocks, 0.3 mm posts; red with the skin emptied).

## #1621 - a part-source cut answers for the setup's other parts

**Decided:** a part-source 2D operation that carries a part law
(`depth_law_of(params, src, true)` not `None`: Contour, Face, Slot,
Trochoidal slot, Circular, the pocket family, Drill without cross holes,
Bore, Thread, Engrave, Trace) is also judged against every OTHER part
the setup holds (`keep_law::neighbour_parts`: the set
`setup_align::parts_facets_in` reads - fixtures and a stock-only model
left out - less the entities its source names, in the setup's frame),
with the plain solid law (`contain_into` with `Keep`: nothing excused,
no hole walls, no mark budget), in both frame arms of `run_job`. The
source keeps its own law unchanged.

**Why the source is excluded, not judged twice:** its own law is the
right one for it - a mark lowers it by the budget, a `Section` cut
answers to its region, a hole kind excuses its own modelled rims. The
plain solid law against the source would convict every legal mark
(measured: mutation M2 below turns the legal-mark pins red).

**Why no budget on a neighbour:** a mark is on the part it marks. A
flank that enters part A while marking B is a gouge of A, however
shallow; judged with the budget it read clean (mutation M3: the probe
passed).

**Readings** (B: the #1585 pocket block on x [0, 40]; A: a 40 x 40 x 8
box on x [-40, -gap]; 80 x 80 x 8 box stock; 6 mm end mill):

| part-source case on B | before: op gouges in A, aligned / turned | now |
|---|---|---|
| Trace / Engrave 0.3 mm round B, gap 0.7 (flank 2.3 mm into A, 0.3 mm deep) | 0, **posts / posts** (the sim credits A the mark budget too) | op gouge, refused on both (by the op's moves on the turned stock) |
| Contour outside B to Z 1, gap 0.7 | 0, refused by the sim (7.00 mm) / **posts** | the same |
| Trace / Engrave 0.3 mm, whole B or B's top picked, gap 5 | 0, posts / posts | posts / posts |
| Engrave 7 mm down B's walls, gap 5 | B's own gouge, named | unchanged |
| 2D Pocket on B's picked pocket floor, gap 0.7 | clean, posts | unchanged: same path, same findings as gap 5 |

**Behaviour change to know:** a whole-model 2D pocket on B (#12: clear
the stock around the part) in a stock that also holds A clears through
A; it is now convicted on the op. That is a real gouge of A.

**Considered and dropped:**
- Judging every part, the source included, with the plain solid law: it
  convicts every legal mark (M2).
- Only the issue's three laws (Mark, Section, Keep): the hole kinds'
  `Contain` law also reads only the source, and a drill on B entering A
  is the same defect; the predicate is "carries a part law" so the two
  cannot drift.

**Known cost / follow-ups:**
- The sim's mark credit (`budget_of`, `stock_sim::credit_marks`) still
  credits a neighbour the mark budget: before this fix the 0.3 mm mark
  whose flank took 2.3 mm x 0.3 mm off A POSTED on the aligned stock
  too. The op now refuses it on both stocks (the output is safe); making
  the sim credit only the marked part, so the two judges agree, is
  #1627.
- The neighbour law tightens by the SOURCE's fidelity excess, not each
  neighbour's (the same simplification #1603 takes for the stock's).

**The other parts are NOT stamp inputs yet (#1628).** A verdict read
against parts beside the source goes stale once one of them moves, and
the op's stamp digests only its source. A stamp term over the kept set
(`keep_law::kept_models`: name, pose in the setup, geometry signature)
was built and pinned on this branch, then taken out: it re-states
established contracts - "workholding never stales" (an un-tagged vise
becomes a part), "a stock-based path never reads the model" (a
stock-source Face now reads its parts), the fidelity refusal on a
deleted stock source - that want the maintainer's call. Filed as #1628
(P1: a part moved into the cut after generation posts on a turned
stock), with the options. Not a regression: before this branch those
parts were not judged on the op at all.

**Tests that changed for #1621:**
- `area_controls.rs::a_boundary_offset_on_a_tilted_pocket_is_the_flat_offset_pocket_rotated`
  compared a tilted pocket on "part" with a flat one on "twin" in a
  world that also holds "tower", standing on the block's top: the tilted
  pocket's +3 offset puts the side-on cutter 3 mm into the tower (15
  gouges, now on the op), the flat twin has nothing beside it. The
  frame-door comparison now runs without the tower, and the tower's
  conviction is asserted - which pins the neighbour law's 3+2 frame arm
  (red with that call removed).
- `stock_source_verdict.rs` (#1094): Op1 (a Bore on the part) now
  judges the unmarked casting beside it, as the program sim does, so its
  cached verdict carries those gouges; the mark takes the casting out of
  that set, and the three tests that post or read the gate after
  marking regenerate first, as a user would (the stamp does not follow
  the mark: #1628); nothing else changed.
  Considered and dropped: never treating the stock's source model as a
  neighbour - unmarked it IS judged as a part (the #1094 rule), and the
  op would then disagree with the sim.

**Cost to change later:** `keep_law::kept_models` (the set) and
`neighbour_parts`, and one call per frame arm in `run_job`.
