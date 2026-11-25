## #3747 - 2D Offset "Pass starts at": rings link in the plane, a lift needs a named reason

**Decided.** Round a part (`StartSide::around_part`, Inside, Outside, and
Auto once resolved), the rings stay at depth and link in the cut plane.
Two places change, both link code:

- The planner's join law (`level_order::linkable`) accepts a chord whose
  samples stand in air as well as over cut material. A centre is in air
  when it stands at least 0.1 tool radii outside the stock footprint: its
  disc then cuts at most 0.9 radii into the stock, the same depth as the
  old law allows next to a cut (0.9 radii, `SWATH`). Where the straight
  chord fails, the planner tries an edge route (`ring_links::edge_route`):
  a chord to the edge of the cutter-centre area (the nearest point, or a
  vertex of a walk that stands on that edge), along the edge either way,
  and a chord back to the next piece. Every chord of the route obeys the
  same join law. The route goes through each walk vertex on the edge, so
  a piece that ends on the edge is touched at a shared vertex, never
  crossed (the verifier's crossing law reads a shared vertex as a touch).
- The ladder's pass link (`ladder_links::pass_link`) of a ring area uses
  the ring law (`ring_links::link`) whatever the pass link of the op is
  (`Links::PLANAR` has `Retract`). The straight link is fed when it is
  within the stay-down limit, inside the certified area, crosses no cut
  of the level, and each sample is in air or over a column that the path
  has cut. Else the tool lifts to the retract plane, and the tally names
  the reason: the stay-down limit, the part (the link leaves the area that
  keeps the tool off the part), a fixture (a "Do not touch" keep-out on
  the link), a crossing, or uncut stock. The reasons show as `link:` notes
  on the path, which the op page shows as warnings.

**Readings** (`pass_start_levels::ring_links`, 80 x 50 box, D8, step 2,
one level). Disc part: Inside 20 lifts with no reason before, 0 now;
Outside and Auto 0. Dog bone (#3744): Inside 15 before, 0 now; Outside 0.
A clamp across the band: Inside 3 lifts, Outside 26 lifts, each named
(the clamp cuts each outer ring into a C, and the climb order needs the
link across it).

**Why.** The maintainer's rule (#3741): each ring goes around the part in
the plane and keeps its depth; a lift is permitted only where an in-plane
link would cut the part or a fixture, with the reason named. The old
links lifted at each break between pieces with no proof and no reason:
the planner joined only through cut material, so each piece that the
stock edge cut began a new pass, and `Retract` lifted there.

**Scope kept small.**
- A pocket (`around_part: false`) keeps its join law and pass link byte
  for byte; #3752 moves 2D Pocket Inside and Outside to the shared law.
  A 2D Pocket with Outside containment (`around_part: true`) takes the
  ring-link law (#3820, #3822): every in-plane link stays in the certified
  cutter-centre area or lifts with a named reason.
- The level link, the entry and the re-entry after a named lift (the
  dead-end ramp, the solid restart) do not change; #3746 owns the entry.
- No new `Links::RINGS` seed. The ring law lives in the motion layer for
  any pass link, so an old project file (stored `Retract`) gets the same
  path as a new one. #3750 owns the seed and its project-file record.
- A chosen pass link of `Clearance` or `KeepDown` on a ring area also
  takes the ring law; a named lift goes to the retract plane, as a
  failed keep-down link always did.

**Dropped.**
- A full shortest-path search through the cut floor for each break (a
  visibility graph over all walks): too slow for each break of each level,
  and a route over cut floor crosses the rings that the verifier's
  crossing law protects. The edge route covers the breaks that the stock
  edge makes.
- Reversing a piece to join it (climb and conventional would mix).

**Cost to change later.** The law is in `ring_links.rs`; #3750 moves it
into the shared `ring_links` module of #3743 with its pins.
