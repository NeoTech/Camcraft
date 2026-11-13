## #1336 - a coarse stock of deleted source: the fidelity refusal blocks first, and that is intended

**What was decided.** The test was out of date, not the gate. The order
of the refusals is unchanged and stays: `posting::verification_report_by`
runs the fidelity clause (#864) first, and since #1056 point 3 a fidelity
refusal silences the sim's own lines ("the gate shows the refusal alone").
What changed is the fixture's input: until PR #1127 (#1056 point 1) a
`FromModel` stock lost its chord once its source was deleted, so the
fidelity gate saw an exact stock and only the rapid law (#1091) blocked.
Since #1127 the stock copy carries the casting's 2.5 mm chord itself, and
the fidelity clause refuses the post by the casting's name. That is the
earlier and the safer refusal: it is the one that is true of every move
judged against that mesh, not one rapid.

`stepped_stock_rapid::a_coarse_stock_whose_source_is_gone_is_never_acquitted`
now pins all three facts, both for a part-sourced and a stock-sourced Bore:

- the coarse, deleted casting's post is blocked by the coarse-import
  refusal, by name (`model 'casting'` + `model_fidelity::TOO_COARSE`);
- the rapid law still convicts that dive (`Warning::RapidInStock` on the
  op): the stock-top acquittal is still refused on an unknown chord;
- the same program with the casting exact and deleted - no fidelity
  refusal ahead of it - is blocked by the rapid law, by name
  (`rapid move <at> `).

**Considered and dropped.**

- *Put the rapid law ahead of the fidelity clause.* Reverses #864 and
  #1056's recorded order; a rapid-law line read off a mesh the gate calls
  too coarse is exactly what #1056 point 3 removed.
- *Assert only "the post is blocked".* Would pass on any refusal and no
  longer prove which law holds the program.
- *Clear the stock's chord by hand to reach the rapid law on the coarse
  fixture.* The exact-and-deleted twin is the same program without the
  coarse refusal and needs no reach into the stock's flag.

**Cost to change later.** None: a test-only change.
