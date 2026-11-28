## #3252 - The STEP vertex gap limit follows the tolerance the file states

**Decided (unattended; the maintainer decides at review).** A built STEP
solid may keep a vertex off the ends of its edges by up to twice the distance
uncertainty that the file states, from `HEAL_GAP_LIMIT` (0.001 mm) to
`EDGE_GAP_LIMIT_MM` (0.02 mm): `EdgeGapCap::vertex_limit_mm`. A file that
states no uncertainty keeps 0.001 mm. SolidCraft records keep 0.001 mm.

- A kept gap over 0.001 mm gets a note on the solid with the limit that
  passed it.
- The gap counts in `EdgeGap::widest_mm`. A gap over the chord makes
  `camcraft_geometry::step` mesh at the file's looser tolerance from the
  start and hand the model out as reduced fidelity (#864). Without that, the
  mesher passes a 5.7e-3 mm corner gap at a 0.005 mm chord
  (`a_vertex_gap_over_the_chord_meshes_coarser` fails with the loosen removed).
- The short-edge recovery of #3259 still starts at 0.001 mm: when healing
  removed edges and left a gap over 0.001 mm, the file's own edges are tried
  first and must pass the full validity check. The file's limit then bounds
  what is kept.

**Why.** Two real Fusion exports (the heater block of this issue, and the
part of #4542) state 0.010 mm and keep source vertex gaps of 0.0077 mm and
0.0036 mm. FreeCAD (Open CASCADE) reads the first as one valid solid. The
edge-gap cap already follows the stated tolerance with the same rule
(decision 2976). The fixed 0.001 mm limit was a chosen bench value, to be
revisited against real models (docs/decisions/27-solidcore-healing.md). With
this rule and the fix of #4542, the #4542 part imports closed, at the
correct inch scale, at reduced fidelity (0.020 mm).

**Dropped.**
- Raise the fixed limit for every file: a file that states a tight tolerance
  would lose its stricter check.
- The stated tolerance itself (factor 1) as the limit: the edge cap uses 2,
  because the writer rounds its numbers. One rule for edges and vertices is
  easier to read.
- Accept the healed body whenever its gap meets the file's limit, before the
  #3259 recovery: that skips the full validity check on the file's own edges
  and passes `refuse_an_unhealed_source_with_an_open_gap_between_curves`.

**Test changes.** `refuse_short_edge_model_when_its_source_gap_exceeds_the_limit`
now uses a file that states 0.0005 mm (limit 0.001 mm), so it still pins the
limit refusal and its location text. The same geometry in a file that states
0.01 mm is pinned by
`a_source_gap_inside_the_stated_tolerance_still_meets_the_validity_check`: it
stays refused, by the validity check.

**Cost to change.** `vertex_limit_mm` is one function. The factor and the
bounds are the edge cap's constants.
