## #864 - an over-budget STEP mesh: report the chord error it kept (held work, escalated)

**What was decided.** The STEP importer's refinement schedule moved into
`app::geometry::import::step_refine::refine`, which returns the mesh it kept
together with the chord tolerance of that mesh. A refinement pass that
crosses the facet budget is dropped, and the previous mesh is kept with its
own tolerance. Before this change, the halved tolerance was reported for the
coarser mesh. `step::parse_with_chord` returns the mesh and the coarsest kept
tolerance over all shells, in millimetres (scaled by the file's length unit).
A non-finite or negative value is refused as a corrupt import. `step::parse`
is unchanged for callers: same schedule, same meshes. `step::CHORD_CAP` names
the 0.005 mm cap. `step::with_facet_budget` is a test-only, thread-local
override that lets a small fixture be forced over budget.

**Why.** Every part of #864 (the UI flag, the MCP facts, the tightened laws)
needs the true chord error of the kept mesh. The `TooLarge` bookkeeping bug
made the logged value wrong exactly in the over-budget case.

**Escalated, not finished.** Tightening "every gouge law that judges
tool-against-model distance" by the excess reaches about 15 law sites in
four crates. It also has an open design question: once the excess is larger
than the 0.01 mm verdict tolerance, a uniform tighter floor convicts every
exactly finished planar face, because planar facets carry no chord error.
That makes such a model unpostable under a misleading "gouge" verdict. The
options are a named refusal, curvature-aware excess, or an outward offset of
the mesh. This is effort:max work. The inventory is on #864.

**Considered and dropped.**
- Logging only: this is the status quo the issue rejects.
- Recording the error on every STEP model: models that fit the budget would
  gain a project-file field and change saved bytes. The plan on #864 records
  it only above the cap.

**Cost to change later.** Low: `refine` is a pure function with unit pins.
