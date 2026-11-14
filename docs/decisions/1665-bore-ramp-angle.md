## #1665 - Bore ramp entry descends at the ramp angle

**What was decided.** A Bore with `entry: Ramp` and no entry ramp angle
no longer takes the single chord from the last ring start to the centre.
The bore now opts into the ring ladder's ramp (`Motion::with_ring_zig`,
#1609). The ramp zig-zags along the level's first cut, the radial sweep
from the centre to the ring, at the transition's ramp angle
(`Links::ramp_angle`, 3 deg by default). If the sweep is too short for
the ramp in at most 64 legs, the ladder dives at the plunge feed and an
`entry:` note names it. The plunge that starts each ring again after the
sweep (the pass link back down the column just cut) also gets an
`entry:` note when the entry is not Plunge.

**Why.** The chord is only one ring radius long, so it descends at up to
84 deg at the cutting feed on a small hole. The ramp angle is already a
user parameter (the level transition's ramp angle), so no new parameter
and no hard-wired value are added. This is the same law that #1609 and
PR #1663 gave to the Circular.

**Considered and dropped.**
- A new bore-only ramp angle parameter: the transition's angle already
  exists and the Circular reads it for the same case.
- Keep the tool down between the sweep and the ring (no lift and no
  plunge, because the ring starts where the sweep ends): this is better
  motion, but it changes every bore path and the tests and goldens that
  read them. The issue asks for a note. A follow-up can remove the hop.
- A helix about the centre in place of the ramp: the user selected Ramp.

**Cost to change later.** Small. The flag is one call in `bore.rs`, and
the note is one branch in `ladder_links::pass_link`.
