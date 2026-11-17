## #2248, #2249 - the selection walk runs once per input for the display

**What was decided.** `operation::selection_account` and
`operation::contour_auto_side` (app, `operation/selection_memo.rs`) read
one memo of the selection walk. One miss runs the walk one time, with the
notes on, and keeps three answers: the notes, the door refusal and the
Auto side. The account and the side of the same operation share the
entry, so the Geometry tab and the Contour fields cost one walk together.

The key is the input itself (rule 2 of
`docs/decisions/2238-frame-cost-rule.md`, first and second forms):

- the setup name, the picks and the parameters, by value;
- for each picked entity and the setup's stock: the entity, the
  allocation of its `Facets` (an `Arc` that each landing replaces), its
  `Location`, and whether a setup alignment moves it (`is_part`);
- for the setup: the effective setup, its `SetupDef`, its machine kind,
  its WCS pose and word;
- the document unit.

These are all the world reads of the walk (`selection_geometry_impl`,
`stitch_open_runs`, `open_band`, `auto_side_of_picks`, `material_of`,
`setup_problem`). A hit is the answer that a fresh walk gives.

`operation::selection_walks()` counts the misses.
`contour_auto_side_fresh` and `selection_account_fresh` walk without the
memo, for the pins.

**Safety line (rule 6).** Generation reads the side from its own walk in
`job` (`selection_geometry_impl`), not from this memo. The pick doors
(`selection_refusal`) and the region walk (`selection_geometry`) also walk
for themselves. `region_of` reads the side through `contour_auto_side`,
and `region_of` is a display answer (the viewport outline, the circular
fit echo, the machined footprint). No gate calls it.

**Why.** On fig8 with ten top chains, the walk cost 4.2 ms for the
account and 4.1 ms for the side on each still frame in release (#2238
probe, state b).

**Dropped.**
- A key on `doc_version`. Rule 2 forbids it, and the app layer does not
  see it.
- A key on the operation entity and a revision that
  `SetOperationGeometry` bumps. A move of the part or an edit of the setup
  changes the answer without that command, so the key must hold the
  world reads.
- A cache of the walk's geometry (`SelectionGeometry`). The display needs
  only the three answers; `GenError` is not `Clone`, and a smaller entry
  is cheaper to compare.
- A cache for `region_of`'s own walk (`selection_geometry`). It is a
  different answer (the region, not the account), it has its own probe
  piece, and it gets its own cache by the same rule.

**Cost to change later.** Small. The memo is a thread-local list of at
most 8 entries. A new world read in the walk must be added to `Reads`;
the pin `contour_side_and_account_walk_once_per_input` compares each hit
with a fresh walk.
