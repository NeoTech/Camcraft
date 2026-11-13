## #692 - Circle and arc centres as opt-in feature points

**What was decided.** The shared pick service offers the centre of a round
feature edge only to a picker that asks for it (`PickKinds::centres`, never
part of `PickKinds::ALL`). A centre pick is stored as the round run it is
derived from (`FeatureSel::Chain(run)`) and read back through #759's
`circle_of_run`; a point slot (`pick::points` three points, `pick::axis`
two points) that gets an edge reads that circle's centre, and a straight or
non-circular edge is refused by name (`pick::centre_point`, wrapping
`CircleRefusal`). In the GUI only the three-points draft (in Vertex mode)
and an axis by two points opt in; the plain pick modes, the plane row, an
axis by edge, fixture locate and the WCS role draft do not. A centre is
offered only when its EDGE is in sight (up to 8 segment midpoints sampled
around the run), not by its own position, so a hole's centre floating in
the air is offered while the rim is visible and a hidden rim's centre is
offered only with Pick-through on. The MCP point picks (`lay_part_on_points`,
`set_part_axes` point pairs) accept a circular edge chain id for its centre
through the same functions.

**Why.** No new `FeatureSel` variant: a stored centre that is just its run
keeps project files, the DTO and every `match` on `FeatureSel` unchanged,
and the WCS "Circle centre" anchor (#739) already stores a run and reads
its centre the same way. A point slot never accepted an edge before, so a
round edge there has one meaning - its centre - and nothing that used to
work changes.

**Considered and dropped.** A `FeatureSel::Centre` variant (a project-file
change touching every consumer); a `centre:0x…` MCP id form (a second
spelling of the chain id for the same thing); opting the fixture-locate
mates and the WCS role draft in now (their datums read vertices by their
own rules; they can opt in later with the same `centre_point`). Also
changed: the snap sort compares ray distance to the nanometre, so two
centres stacked on one axis tie and the nearer one wins instead of float
noise picking the far rim.

**Cost to change later.** Low: opting another picker in is one `PickKinds`
line in `feature_click::ask` plus accepting `FeatureSel::Chain` through
`pick::centre_point` in its consumer.
