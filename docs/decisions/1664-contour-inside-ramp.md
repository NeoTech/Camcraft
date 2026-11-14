## #1664 - Bore-side Contour ramp entry holds the ramp angle

**Decided.** In `contour_bore.rs`, the `Entry3D::Ramp` arm with no entry
ramp angle set (`approach.ramp_angle == 0`) keeps the single chord only
when that chord is no steeper than the links' ramp angle (3 deg by
default; a value that is not a real angle reads as the default). A chord
with no run (a closed ring: the leg is the ring start) or a steeper chord
zig-zags along the lead-in and the ring at that angle, with the same
`plan_zig_capped` / `push_zig` the entry-angle arm uses. When the path is
too short for the angle, the descent substitutes by name
("the lead-in and ring are too short to ramp at the angle"): an axial
dive at the plunge feed with an `entry:` note.

**Why.** On a closed ring the chord has zero length, so the old arm fed
straight down at the cutting feed at each level with no note (the
center-cut guard was the only check). That is a plunge called a ramp.

**Considered and dropped.**
- Route the arm through `descent::plan_arm` / `ramp_fallback` (#1663's
  `ring_zig` arm). The contour does not use the `Descent` planner; its
  arms push moves directly, so the same law is applied here with the
  same zig helpers. No dependency on #1663.
- Zig-zag also when the chord is blocked. It would move the pinned
  `a_blocked_bore_entry_substitutes_and_says_it` behaviour; a blocked
  chord still substitutes by name, which the issue accepts.
- Change the wall side (`Side::Outside`). It already makes named axial
  plunges at the plunge feed; unchanged.

**Cost to change later.** One match arm and three small helpers in
`contour_bore.rs`.
