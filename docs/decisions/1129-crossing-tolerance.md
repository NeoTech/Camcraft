## #1129 - The crossing law (a) reads a shared vertex a rounding apart as a touch

**What was decided**

1. **A named tolerance, a distance.** `camcraft_sim::verify::CROSSING_TOUCH`
   = 1e-9 mm (1 pm), documented in `crates/sim/src/verify/laws.rs`. f64
   rounding of a 1e4 mm coordinate is ~2e-12 mm (a hundred-odd times below
   it); cutting tolerances are microns (a million times above). It is
   compared as a point-to-point distance (`hypot`), never as a raw
   cross-product magnitude, so a 1 um chord and a 1 m chord are judged the
   same.
2. **Only END points are excused, not the whole line.** The new
   `proper_cross_touch` (used by law (a), `verify/crossing.rs`) reads a
   chord end as on the other chord's line (orientation 0, a touch) when it
   stands within `CROSSING_TOUCH` of one of the other chord's END points.
   Away from those, the exact sign stays the law. Every case the issue
   names (an orbit closing a ulp off its start, a microlift return landing
   on a chord end, a ring starting on the last ring's vertex) is a shared
   vertex, so this is enough.
3. **`proper_cross` itself is unchanged.** `dist2_segs` and the arc-wall
   law (`arc_wall.rs`) keep the exact predicate; their verdicts are out of
   this issue's scope.
4. **The verifier is the reference.** The doc comment says the adaptive
   planner's `guard::crosses` (#1048, PR #1134's `TOUCH`) should use or
   match this constant and this end-point reading, since the plan check is
   meant to be this law asked first.

**Why**

- The #1134 review (NeoTech, head d1dc959b) showed that reading "within
  `TOUCH` of the other chord's LINE" hides a real crossing: a walk
  passing through a chord at a vertex 0.5 pm off that chord's interior
  makes both of its chords there read as touches, so the pass-through,
  of any depth, is not reported. It said #1129 must not port that test to
  the verifier, where it would remove the backstop. Excusing only end
  points keeps that probe reported (pinned below), at 0 and 1 m.
- Two chords that meet at (nearly) one vertex cannot properly cross each
  other elsewhere unless they are near-collinear, where the exact test
  was already rounding noise.

**Considered and dropped**

- *The line band of #1134 (`|orient| <= TOUCH * |ab|`).* Matches the
  planner as it stands, but opens the pass-through hole above in the
  post's last check. Dropped under the Prime Directive.
- *Pass-through detection at a near-line vertex (look at the walk's
  previous and next points).* Closes the same hole while keeping the line
  band, but needs the move-to-move topology the per-pair law does not
  carry. Not needed for the issue's cases.
- *Stacking on #1134.* It carries `needs-rework`; this branch is cut from
  main and does not depend on it.

**Residual, unchanged in kind.** Two walks crossing in an X exactly
through a shared vertex were never reported by the exact-sign law (every
orientation there is 0); this change extends that same reading to a
vertex a picometre apart. No new class of miss is added.

**Pinned by** `crates/sim/tests/crossing_touch.rs`:
`a_shared_vertex_a_rounding_apart_does_not_warn` (1e-12 mm, 16
directions, origins 0 / 37.25 / 1000 mm, chords 1 um to 1 m, and each
scale asserts the exact test tripped),
`a_crossing_a_micron_through_still_warns`,
`a_walk_through_a_chord_at_a_vertex_a_picometre_off_its_line_still_warns`,
`collinear_overlaps_keep_their_reading`,
`a_joint_just_past_the_touch_keeps_the_exact_reading`.
