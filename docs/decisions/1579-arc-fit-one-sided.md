## #1579 - one-sided arc fit on contour walls

**Chosen.** A contour walk that knows its part side (closed rings, open
chains, the lead reseat and the cutter-compensation part contour;
`air_left` = `climb`) fits arcs one-sided:
- A vertex may stand at most `ARC_SIDE_SLACK` = 1e-3 mm on the part side
  of its arc. When the two-sided circle already meets that, it is kept
  as it is.
- Otherwise an open run keeps its two end vertices, and the centre
  slides along their bisector until no interior vertex is on the part
  side (the offset nearest the two-sided circle). A closing ring has no
  chord to slide along, so its arc is refused and the greedy fit keeps
  the longest open run.
- A sweep past half a turn posts as pieces that end on interior
  vertices. In a sided fit each such split vertex must lie within the
  slack of the circle, or the run is refused.
- A sided fit looks past up to 4 refused run ends before a run ends
  (`ARC_SIDE_LOOKAHEAD`), because a run that ends on a facet's flat is
  refused while the run to the next corner fits.

The two-sided fit (`fit_arcs`) is unchanged for every other caller.

**Why.** The two-sided fit let a vertex stand half the operation
tolerance (0.05 at 0.1) off the circle in either direction. Toward the
part that is a cut past a verified point, and the verifier forgives only
`GOUGE_TOL` (0.01), whatever the operation tolerance. So a coarse fit on
a tessellated round wall posted arcs that verify then held as gouges,
and the result depended on the walk direction.

**The slack value (1e-3 mm).** The maintainer's session decided that
golden output does not change. The first held fix used a slack of 1e-9:
exact holes, bores and the M8 boss then lost their closed-ring arcs to
rounding noise, and seven golden tests failed. 1e-3 is far above that
noise and ten times below `GOUGE_TOL`, so no fitted arc passes a vertex
near what the verify holds as a gouge. With it every golden test passes
unchanged. Values up to 5e-3 were possible; the smaller value keeps the
larger margin, and the tests did not need more.

**Split vertices.** Without the split check, the Ø30 cylinder fixture
walked conventional (ccw) posted a sweep over half a turn split at a
vertex 0.05 mm off its circle. The verify then held that piece as a
gouge, although every vertex was on the air side of the circle.

**The lookahead.** Without it the same fixture fit 22 (climb) and 32
(conventional) short arcs, and the UI draw law `arc_moves_draw_curved`
(at least 8 chords for each arc move) failed. With it each level fits 3
arcs in both directions, and the draw law holds on the climb walk, as
the issue asks.

**Considered and dropped.**
- Regenerate the goldens with the 1e-9 slack: refused by the
  maintainer's session; a true circle must post as before.
- Cap the vertex share at `ARC_FIT_TOL` (0.005): the Ø30 cylinder wall
  then fit no arcs at all at tolerance 0.1 - the offset wobble of the
  facets is wider than that.
- Prove each arc against the legal region after the fit (as the adaptive
  plan's `arc_proof` does) and drop the refused ones: it keeps chords
  where a slightly larger circle is safe, and costs a region test per arc.
- Raise the verifier's allowance to the operation tolerance: that hides
  a real cut past the wall, against the Prime Directive.
- Leave the compensation part contour (`contour_comp`) two-sided: the
  control offsets that contour by the register, so a part-side sag of it
  cuts past the wall the same way, and the verifier reads only the
  centre moves, so it would not see it. It walks the centre ring's way
  round, so its air side is also `climb`.

**Not changed.** The pocket, ring, waterline and plasma fits stay
two-sided; their sag laws are their own.
