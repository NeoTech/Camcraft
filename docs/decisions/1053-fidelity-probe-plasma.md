## #1053 - reduced-fidelity models: probe and plasma are judged; the cone and engrave rides stay refused for a stated reason; the cross drill is left open

**What was decided.** #904 left five kinds refused outright on any
reduced-fidelity model. This slice takes two of them off the refused list
by the #864 rule, unchanged: a tolerance gives the model's excess back
while a positive tolerance is left (`verify::excess_floor`); past that the
law reads untightened and the program gate refuses the operation by name.

1. *Probe.* Its only laws against the model are the protected-move law's
   two halves: the ball's distance law (`probe_op::stylus_hits`) and the
   stem and body bands' solid law (`stylus_reach_hits`). Both are granted
   `probe_op::stylus_slack(excess)`: `STYLUS_SLACK` (0.01 mm) less the
   excess. The `_excess` twins carry the number; the old functions forward
   with 0, so their callers are untouched (#904's twin pattern). A probe
   cuts nothing, so no `verify` law runs on it (it returns before them).
2. *Plasma.* Its only laws against the part are the kerf law (no lit move's
   kerf inside the part grown by `kerf / 2 - KERF_TOL`) and the cover law
   (every edge within `kerf + KERF_TOL` of its cut). Both now read
   `PlasmaParams::kerf_tol()`: `KERF_TOL` (0.025 mm) less
   `PlasmaParams::model_excess`. The cut-order, kerf-crossing and tip-up
   laws read the path only. The lead planner reads the same grown part,
   so a lead with no room is refused by the same tightened reading.
3. *The floor* (`fidelity_law::floor_of`) lists both slacks
   (`own_law_tol_of`). `GOUGE_TOL` stays the base for every kind, so
   plasma, whose own law holds 0.025 mm, is refused at the same 0.01 mm of
   excess as everything else.

**Why.** Each of these laws is a comparison of a measured distance with a
slack, exactly the shape #864 tightened; nothing else judges either kind
against the model. A metre-unit STEP file is always reduced fidelity, so
before this no probe or plasma program could post from one.

**Decisions taken unattended.**

- *The cone rides (2D chamfer, deburr) and the engrave rides (trace,
  engrave) stay refused, permanently under this rule.* No law holds them
  to a tolerance against the model: law (b) stands down for both modes
  because the ride IS the mesh's edge by construction, so the cut stands
  off the true edge by the mesh's own chord error, and no reading of that
  mesh can see it - there is no tolerance to give the excess back from.
  Deburr is worse: it finds edges by the bend between facets, and a
  coarse mesh of a smooth face bends more between neighbours, so it can
  be read as a crease the ride then breaks, a cut no law speaks to.
  Judging them needs a stated ride accuracy (a new promise and its own
  law), which is open design, not a tightening. The reason is in
  `fidelity_law`'s header.
- *The cross drill stays refused, and #1053 stays open for it.* Its part
  law (`cross_part`, `GOUGE_TOL`) and its rapid law (`rapids_radial`'s
  `r - gouge_tol` disc) could give the excess back in one line each, but
  the chain is also proved by construction laws that measure the mesh
  with no slack at all (1e-6): the standoff radius and Z top the kernel
  stations about (`cross_stock::standoff`, `rapids_clear`), the entry
  along each hole's line (`cross_entry`), the flute reach and the holder
  faces (`drill_reach`). Each reads a coarse mesh's chord error straight
  through. A sound tightening exists - grow every measured standoff by
  the excess - but it moves the kernel's stations and touches four
  modules; it is the remainder of #1053, so this PR says `Refs`.
  Tightening only the two easy laws now was dropped: #904 set that a law
  is not tightened for a kind that stays refused, since it never judges.
- *The probe's tightening can refuse a bore touch on a coarse model.* A
  bore's facets are chords standing inside the bore, so a ball tangent to
  the true wall stands nearer the facets by their sagitta. Past about
  0.0025 mm of excess the tightened slack convicts that as a gouge; the
  untightened slack already did past 0.01 mm of sag. It is a false gouge
  on the safe side (the #1057 shape), the probe can still be posted from a
  finer import, and loosening it would need the hole-wall excuse the
  containment kinds carry; not done here.
- *The cover law is tightened too*, though an uncovered edge leaves
  material rather than cutting the part: it reads the model-derived
  contour against the same tolerance, and one number for both plasma
  laws can only be stricter.
- *`model_excess` is a field of `PlasmaParams`*, not a new argument of
  `plasma_profile`: the params struct is what the generator reads, and
  the four struct literals (one app, three tests) state 0.

**Considered and dropped.**
- A per-kind floor of 0.025 mm for plasma: honest to its own law, but the
  only gain is posting plasma from imports between 0.015 and 0.030 mm;
  the common floor is stricter and simpler.
- A document-level plasma fixture: the plate's part region comes from the
  model's silhouette slice, which cannot be grown by a chosen 0.02 mm; the
  law is pinned through `plasma_profile` with the grown part handed in,
  and the probe is pinned through the document as well.

**Cost to change later.** Low for the slices taken: one `excess_floor`
call per law. The cross drill's remainder is the four standoff readings
named above, then deleting its clause in `fidelity_law::excess_aware`.
