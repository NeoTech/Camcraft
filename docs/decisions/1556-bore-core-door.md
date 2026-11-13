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

**Side effect.** The motion unit fixture `bore_rides_the_wall_circle`
bored a Ø20 hole with a 6 mm tool (3.33 d): that ride left a 1 mm-wide
ring uncut on the floor, so it is refused now and the fixture moved to a
Ø16 hole. No golden and no app fixture held a hole past the limit.

**Cost to change.** Lifting the limit (#1566) means changing
`max_bore_diameter` and the ride together; detection follows the
function automatically.
