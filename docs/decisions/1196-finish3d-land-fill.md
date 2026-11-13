## #1196 - 3D Finish land model: fill closed openings on the drive surface

**What was decided.** 3D Finish gets one optional parameter,
`fill_openings` (mm, a diameter; absent = off, every older file and every
fresh op). When stated, the generator plans the path on the model's facets
plus horizontal lids over every *closed* opening a flat disk of that
diameter cannot enter, so the finish flies over bolt holes and small
pockets at their rim instead of diving down their walls. The fill rule:

- Sample the model's top-down height field `H` on a grid (cell = width/12,
  coarsened to at most 4M cells; a width too fine for the model at that
  cap is refused by name). Cells over no facet read as the model's lowest Z.
- Grey-scale morphological closing of `H` with a flat disk of the stated
  diameter (Serra 1982; Soille 2003). Cells raised by more than the finish
  tolerance are opening cells.
- A connected group of opening cells is filled only when it is *closed*:
  none of it on the grid border, and every cell bordering it is material
  standing at the lid's level (within one cell's 45-degree rise plus the
  tolerance). A slot or valley running out to the part's edge, or a lid
  that would spill over a lower rim, is open and never filled.
- Lids cover the group plus its ring of neighbouring cells, each at its
  own closed height (never below `H`), merged into row rectangles.

**Safety argument.** The drive facets are the model's own facets,
unchanged and first, plus lids: every finishing law keeps the ball off
every facet it is given, so a path planned on the superset can rise but
never bite the model deeper. The verifier is not given the lids - it still
judges the real model (`material.facets`), so a gouge under a lid is
convicted (pinned by `a_blind_pocket_is_lidded_and_a_gouge_into_its_floor_is_convicted`).
The path carries a `land: N closed opening(s) ...` note naming what stands
as land on purpose.

**Considered and dropped.**
- A "land Z" / ignore-below-Z plane (the reporter's first shape): on the
  reported part the top folds 8.4 -> 4.5 mm, so no single Z separates the
  holes from the surface; a width threshold follows the part.
- Topological hole capping on the mesh (find the rim edge loops): needs
  a clean manifold and fails on the imported STEP meshes this is for; the
  height-field closing needs nothing but facets.
- Filling every closed depression regardless of size: a large width
  fills more, but the grid follows the width (cell = width/12), so a large
  width samples coarsely and can miss a thin rim - the opening then reads
  open and is not filled (safe: nothing is lidded). An unbounded "all"
  would need its own rule (a cell clamped by tolerance or tool radius) and
  is left until asked.
- Geodesic: the lids are new open boundary for its march, so the
  combination is refused by name rather than silently changing its seed.

**Pencil and Corner are refused with the fill on (review of PR #1286).**
Both walk crease loci off the mesh's shared-edge index; the lids share no
edge with the model, so the creases inside a lidded opening were still
found and cleaned while the path note said the opening was "flown over,
not cut" - a note claiming a fill the path does not do. Chosen: refuse the
pair by name at generation, as Geodesic is ("Pencil cleans the creases
inside openings, so it cannot fill them - clear 'Fill openings' or pick
another pattern"). Considered and dropped: making the crease walk honour
the lids (drop every locus under a lid) - new crease logic under the
safety floor for a combination nobody asked for, and a pencil pass whose
whole job is the creases inside openings has little to gain from a fill.
Flow was checked and is NOT refused: its seeds and march ground every
column on the same ball support as Parallel (`plunge_support` over every
drive facet) and its flanks ride the waterline rings, so it flies over the
lids (pinned by `flow_honours_the_lids_and_never_descends_into_a_lidded_pocket`).
When nothing was filled the note now says "no closed opening narrower than
W mm found - the path follows the model" instead of "0 closed opening(s)
... flown over".

**No project-file version bump.** The key is optional and absent means
off, so every older file reads unchanged; an older reader ignores the key
and cuts the model itself (diving into holes) - a different but valid
finish, not an unsafe one, so no reader must refuse it. The stamp digest
gains bytes only when a width is stated, so no existing op goes stale.

**Cost to change later.** The rule lives in one file
(`crates/motion/src/finishing3d/fill.rs`); the parameter is one optional
key. A finer lid (true rim polygons instead of grid rectangles) would
change only the lids' emission.
