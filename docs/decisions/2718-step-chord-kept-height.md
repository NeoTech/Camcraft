## #2718 - STEP import: report the height of the kept curved facets, and split to the tolerance

**What was decided.** The curved-face fit (`step_fit.rs`, #2600) measures
every edge of the facets that it keeps, inside and boundary. It reports the
highest edge height times a bound for the inside of a facet:

- cylinder: 1.0 (the facet is inscribed in the circle; its edges bound it);
- cone, sphere, torus: 1.5 (the bound for a surface curved two ways is
  4/3, from the linear-interpolation error identity of Waldron, SIAM J.
  Numer. Anal. 35, 1998; 1.5 is a margin over it).

The import takes the larger of this value and the pass tolerance as the
model's chord. An inside edge is now split until its height is at most
`tol / inside`, not `2 x tol`.

**Why.**

- Before, the fit kept inside edges up to `2 x tol` (0.01 mm) and the
  import reported `tol` (0.005 mm). The model said full fidelity while its
  facets stood up to twice the cap off the surface.
- With only the honest report, the D50 cross-hole bar fixture
  (`bar_cross_holes.step`) reads 0.00999 mm. That is over `CHORD_CAP`
  (0.005 mm), so the bar becomes reduced fidelity and verify refuses ops on
  it. To keep the bar at full fidelity with a raised cap is a loosening of a
  threshold. To split to the tolerance keeps the report honest and under the
  cap: the bar reads 0.005 mm after the change.

**What was considered and dropped.**

- Report only, keep `2 x tol`: honest, but it moves the bar fixture and
  each similar model to reduced fidelity. Dropped.
- Raise `CHORD_CAP` to 0.01 mm: a loosening of the gouge verdict's margin
  (#689: half the verdict for the mesh, half for the cut). Dropped.
- An exact per-facet nearest-point measure: more cost for each facet, and
  the edge identity already gives a sound bound. Dropped.

**Cost to change later.** The split limit is one line in `fit()`. A
tighter split makes more facets: a face that runs out of facet budget keeps
its larger edges, and the import then reports them (reduced fidelity, a
refusal, not a silent pass). Issue #2713 may change the split rule in the
same function; a merge must keep the limit `tol / kind.inside()` and the
report of all kept edges.
