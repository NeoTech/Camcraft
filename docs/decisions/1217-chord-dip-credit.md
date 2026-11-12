## #1217 - The hole-wall excuse credits a declared rim's chords, not a polygon's flats

**Decided.** The containment kinds' hole-wall excuse (`swept::chord_dip`,
read by `solid_index` under `SolidLaw::hole_walls`) now judges each facet
edge on its own:
- An edge whose two ends both lie on one **declared rim** circle (within
  `RIM_TOL` = `DEV_TOL`, 0.01 mm, in XY, on an upright move) is a chord
  of the round wall that the region judges exactly. It is still excused
  its whole dip, `rho - sqrt(rho^2 - l^2/4)`.
- Any other edge is credited at most `HOLE_CHORD_CREDIT` = `DEV_TOL`,
  0.01 mm. That is the mesh tolerance a round wall's tessellation stands
  within, and the same value #1099 chose for its thread grant.

The rims come from the operation (`generate::contain_into`, through the
new `verify::solid_hits_rims`): each declared hole's circle
(`Hole::xy`, `Hole::radius`) for Drill, Bore and Thread. A drill cycle
also gets its own circle at the hole centre (tool diameter / 2), because
a drill or tap cuts its own diameter there. That is how the
`tap_plate` golden reads it: the hole record is the tap drill's 2.5 mm,
and the modelled thread wall is at the tap's own size. `solid_hits`
keeps its signature and passes no rims. The cross-drill law and the
probe law therefore get only the 0.01 mm credit.

**Why.** From the mesh alone, an octagon with its corners on r 5 is
exactly what an 8-segment tessellation of an r 5 circle looks like.
What tells them apart is what the operation declared:
- In the issue's case the thread was declared at r 4.4, so the corners
  at r 5 lie on no declared rim, the flats get no excuse, and the cut
  convicts.
- A capped credit alone (the first attempt on this branch) broke pinned
  behaviour: `drill_op_recognizes_from_the_model_then_machines_clean`
  (a 16-gon r 2 hole that dips 0.038 mm) and the `drill_plate` and
  `tap_plate` goldens (32-gons that dip 0.014 and 0.024 mm) all
  convicted as false gouges. Every coarse STL drilled hole would have
  done the same.

**Considered and dropped.**
- **A bare cap at 0.01 mm:** false gouges on every coarse round hole
  (above).
- **A fixed coarser cap (about 0.05 mm):** it would pass a real flat by
  that much, and the number is arbitrary.
- **Reading the model's recorded chord (#864's `ReducedFidelity`):**
  only STEP imports record one, so STL and mesh models would be no
  better off.

**Residual, stated.** A hole declared (or recognized) at a coarse
polygon's own corner circle is still excused in full on its flats,
because the declaration says the hole is round. The same holds for a
drill whose diameter matches the corner circle. The region law judges
tool against declared rim; only the recognizer could tell a polygon
from a circle, and that is follow-up work.

**Cost to change.** `HoleRim`, `solid_hits_rims` and two constants; the
rim list is built in one place (`contain_into`).
