## #2317 - Waterline and Offset rings hold the crossing law on the plain fit when arc turns are on

**Decided.** When arc turns (#1733) are on, Waterline and Offset rings
hold the crossing law (#2240, `arc_cross`) on the plain arc fit, before
the corners are rounded. The rounded walks are then posted without a
second refusal round. Adaptive and rest machining (`hold_crossings`)
keep the law on the posted rounded walks, as before #2273. Without arc
turns, nothing changes from #2273.

**Why.** PR #2273 (`d8c710f5f`) sent the rounded walks of Waterline and
Offset rings through `post_crossing_free`. Its `refuse` step maps a walk
back onto its pass by the pass's own vertices (`arc_proof::judge`). A
rounded walk has new vertices, so the map fails and `plain()` gives up
every arc of the pass, the rounded corners included. Two triggers:

- a fitted arc that truly crosses another feed (the Waterline island
  pocket has one);
- a blend arc that lands tangent on a leg of another pass that is on the
  same line: the 1 um centre lattice puts its bracket 7e-8 mm past that
  line, which the strict predicate reads as a crossing.

On the `corner_arc_radius` island pocket, Waterline kept 6 arcs of 38 and
Offset rings 0 of 102. With the law on the plain fit, the fit's real
crossing is still refused, all blends stand, and
`verify::crossed_pairs` finds no crossing pair at 0.01 mm or 0.0005 mm.

**Dropped.**

- Exempt the blended walks from the law (the state before #2273): this
  loses the #2240 guard on the fit's arcs.
- Teach `refuse` to undo one blend in a rounded walk: new geometry in the
  refusal path, larger than this fix.
- Add a touch tolerance to the `arc_cross` predicate: it would change the
  law for the HSM plan and the 3D ladder too.

**Cost to change.** Local to `pocket_emit::emit_pocket_passes` and
`arc_cross::crossing_free_runs`. If a blend arc must be held to the
law itself later, `refuse` needs a map for rounded walks first.
