## #1231 - A polygon declared at its corner circle keeps no full rim credit

**Decided.** A facet edge earns #1217's full chord-dip excuse only if it
is a chord of a declared rim (both ends within `RIM_TOL`) **and** it dips
at most `RIM_SAG_MAX` = 0.05 mm inside that circle
(`r - sqrt(r^2 - l^2/4)`, in `solid_index::on_rim`). A deeper chord is
a polygon's flat. Like every off-rim edge, it gets at most the 0.01 mm
mesh-tolerance credit. So an octagon (flats 0.38 mm deep) or a 16-gon
(0.096 mm) on r 5 convicts against a drill, bore or thread of the
corner-circle size, whether the hole was declared, recognized or met by
a drill cycle's own rim.

**Why here, not in the recognizer.**
- The hole list is editable data, so a user can type the corner
  diameter by hand.
- A drill cycle adds its own rim at tool diameter / 2 (#1228, for the
  `tap_plate` golden). That rim lets a drill of the corner diameter
  claim the corners no matter what the recognizer declared.
- Declaring recognized polygons at their inscribed radius would close
  neither path.
- The verifier sees every path through one gate, which is the sound
  place for the check.

**Why 0.05 mm.**
- It sits above every round hole the tests pin: the operations
  fixture's 16-gon r 2 dips 0.038 mm, and the `drill_plate` and
  `tap_plate` 32-gons dip less.
- It sits well below any real polygon's flats.
- A coarser round hole (an STL exported coarser than this) is judged as
  the polygon it is. That gives a refusal, never a pass.

**Considered and dropped.**
- **Declaring at the inscribed radius in the recognizer:** the drill's
  own rim and hand-entered holes bypass it (see above).
- **The source's chord tolerance (#864's `ReducedFidelity`):** only
  STEP imports record one. A model with no record is "known exact", and
  that would convict the pinned 16-gon r 2 (0.038 mm) as a false gouge.
- **The STEP surface type:** the verifier gets triangles, not faces.
  Plumbing it through would be format work, and STL models would gain
  nothing from it.
- **A minimum segment count (angle spacing):** it scales with radius,
  so a large 16-gon's millimetre-deep flats would still pass.

**Residual, stated.** A polygon whose flats dip at most 0.05 mm, declared
at its corner circle, can still be cut that much into its flats. That
is bounded, where #1228 left it unbounded.

**Cost to change.** One constant and one predicate in
`crates/sim/src/verify/solid_index.rs`.
