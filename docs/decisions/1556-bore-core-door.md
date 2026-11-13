## #1556 - A bore too wide for the Bore ride is refused by name, and Detect holes leaves it out of a Bore op

**Decided.**

- The Bore kernel (`machining2d::bore`) refuses at generate any hole whose
  ring would leave a core: its ride is a centre dive (clears radius `r`),
  one radial sweep and one ring on the wall (clears the band
  `[rc - r, rc + r]`, `rc = R - r - leave`). The two meet only while
  `rc <= 2r`, i.e. for holes up to `3 x tool diameter + 2 x stock to
  leave` (`machining2d::max_bore_diameter`). Past that the refusal names
  the hole (diameter, position), the cutter, the ring radius, the core
  left and the limit, and points at a pocket/adaptive op or a larger
  tool. Never generated as ok.
- Detect holes on a Bore op keeps only the holes within that limit for
  the op's own tool and names each one left out ("the 74.300 mm hole at
  (...) was skipped: wider than the 12.000 mm a 4.000 mm tool bores
  without leaving a core - ..."). If every hole is too wide, the detection
  is refused (nothing stored). Drill and Thread ops are not filtered.
  The note rides the MCP reply (`skipped`, an array of sentences) and the
  GUI status line (a `DetectedHoles` queue arm that stores the same
  `EditOperationParams`, so undo is unchanged).
- Priority raised P3 -> P1 on the issue: a freed core after a clean
  verify is unsafe output that passes.

**Why.** The Prime Directive asks for correct output or a visible
refusal. The refusal is one closed-form inequality at the kernel's door,
shared by detection through one public function, so the two can never
disagree, and it changes no path the ride already clears (every
accepted hole's floor is proven covered by the new law test). It is the
smallest safe change.

**Considered and dropped.**

- *Clear the floor fully* (rings or a spiral out to the wall). Makes the
  op useful for wide holes, but it is new toolpath behaviour (stepover,
  full-width engagement per ring, entry choice) beyond a refusal law, and
  for a Ø74 bore with a 4 mm tool a pocket/adaptive op is the better
  program anyway. Filed as #1566.
- *The issue's "about two tool diameters" threshold.* The geometry says
  three: the dive's disc and the ring's band meet at `D = 3d + 2 leave`.
  A 2d limit would refuse holes the ride clears today.
- *Detect holes keeping the wide hole with a warning.* A stored hole the
  generator then refuses is a trap; leaving it out with the reason in
  the reply is what the issue asks for.
- *A general "freed piece" detector in verify / the stock sim.* Neither
  catches a disconnected piece today (with the door disabled, the wide
  bore generates and verifies ok). That needs design - freeing a slug is
  sometimes intended - so it is filed as #1565, not bolted on here.

**Side effect.** Three motion fixtures (`bore_rides_the_wall_circle`,
`center_cut_law`'s bore, `level_links`' `bore_params`) bored a Ø20 hole
with a 6 mm tool (3.33 d): that ride left a ring of floor from Ø6 to Ø8
uncut, so it is refused now and the fixtures moved to a Ø16 hole. No
golden and no app fixture held a hole past the limit.

**Review fix (PR #1575, pullrequestreview-5899268871): the corner radius.**

- *Decided.* The core door reads the cutter's corner radius
  (`BoreParams::corner_radius`, the op tool's `corner_r`) and judges a
  through hole at its floor plane. The limit is
  `D = d + 4 reach + 2 leave` (`machining2d::max_bore_diameter`), with
  `reach` how far the cutter cuts at the plane the core would stand on
  (`machining2d::bore_floor_reach`). Detect holes asks the kernel's own
  verdict (`machining2d::bore_leaves_core`) with the same corner and the
  same through allowance (`drilling::THROUGH_CLEAR`) generate hands it,
  so the two cannot disagree at the limit or over the leave.
- *Derivation.* A corner-radiused end mill of radius `r` and corner `cr`
  (`0 <= cr <= r`) is a flat disc of radius `r - cr` with a quarter torus
  around it: the corner's centre circle sits at radius `r - cr`, height
  `cr` above the tip. At height `h` above the tip (`0 <= h <= cr`) the
  torus' outer radius is `(r - cr) + sqrt(cr^2 - (cr - h)^2) =
  (r - cr) + sqrt(h (2 cr - h))`; from `h = cr` up it is `r`
  (`machining2d::cutting_radius_at`). Every level of the ride sweeps the
  same dive disc and ring band, and the deepest level's cutter passes
  every plane above its tip, at a height that only grows going up, so its
  radius only grows: the floor plane is the worst plane. On a through
  hole the deepest tip runs `a = THROUGH_CLEAR` (1 mm) under the floor,
  so the floor sees `reach = (r - cr) + sqrt(a (2 cr - a))` when
  `a < cr`, and `r` otherwise. The dive (radius `reach`) and the ring's
  inner edge (`rc - reach`) meet while `rc <= 2 reach`; with
  `rc = R - r - leave` that is `D <= d + 4 reach + 2 leave`. A square end
  mill (`cr = 0`) gives `reach = r`, the old `3d + 2 leave`. A ball
  (`cr = r`, no flat) is the same formula with `r - cr = 0`; a ball
  smaller than the drop (`r <= a`) reaches its full radius.
- *Worked example (the review's).* Ø12, `cr = 2`, through: `reach = 4 +
  sqrt(3) = 5.732`, so the limit is `12 + 4 x 5.732 = 34.928 mm`, not 36.
  Between those, the ride left a C-shaped ridge at R6 about 0.5 mm wide
  and up to 1 mm tall, cut once by the radial sweep - free. Ø6 ball,
  through: `reach = sqrt(5) = 2.236`, limit 14.944 mm.
- *Blind holes keep the full radius.* The floor stays under a blind hole:
  what the corner leaves between the dive and the ring is a ridge
  standing on that floor, up to `cr` tall - the cusp every bull-nose
  floor pass leaves - and never comes loose. The door's job is a free
  piece, so a blind hole is judged at the height where the cutter is
  full width (`reach = r`). A flat floor under a bull nose is a stepover
  question for #1566 (clearing wide holes), not this refusal.
- *A corner that does not fit the cutter* (negative, or over half the
  diameter) is refused at the kernel's door by name; detect filters
  nothing for such a tool and lets generate name it.
- *Considered and dropped: an overlap margin at the limit.* At the exact
  limit the dive's disc and the ring's band touch edge to edge (the
  coverage law proves no floor point is left). A margin of a micrometre
  guards nothing a spindle can see, and a meaningful one (runout, a few
  hundredths) would move every printed limit off the round
  `3d + 2 leave` the user can check by hand, and treat this one op
  stricter than every pocket stepover in the code base, which also takes
  nominal geometry. The ring is posted as arcs within `tolerance` of the
  polygon, whose chords stand inside the circle, which only widens the
  overlap. Kept touching-is-covered; revisit with #1566 if a cleanup
  ring is added.
- *Old projects.* A project saved before this fix keeps its cached path
  and stamp (no generator version in the stamp), so it can still post a
  cored path without regenerating. Noted on #1565 (a verify-time check
  of each Bore op's holes against this limit closes it).

**Cost to change.** Lifting the limit (#1566) means changing
`machining2d::bore_core` (`bore_leaves_core`, `max_bore_diameter`) and
the ride together; detection follows the verdict automatically.
