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

**Cost to change later:** one predicate (`keep_law::is_stock_kept_kind`)
and one branch in `run_job` (mark or solid law for `keep_parts`).
