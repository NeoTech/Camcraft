# #2929 - A drafted side is not a modelled chamfer

## Chosen

A no-pick 2D chamfer reads a side of an edge loop as **drafted**, and not as
a modelled chamfer, when two conditions are true:

- Its plan offset from the wall at the corner depth is at most
  `depth * tan(15 deg)`. `depth` is the distance from the top plane to the
  corner plane.
- The section at half depth stands half way between the top edge and the
  wall within `LEAN_SLACK` (0.0001 mm). This check limits the departure
  from a straight wall to the section grid slack.

A loop with drafted sides and no modelled chamfer is ridden along its wall at
the corner depth. It counts as a sharp loop for the #2596 law: when the model
has no modelled chamfer, every edge is broken, the drafted loop included.
The partly bevelled plate of #2887 is still refused.

## Why

- The issue gives two options: cut the drafted plate as before, or refuse it
  with a correct message. A drafted side is a normal part feature (cast and
  moulded parts). A refusal makes the user pick each edge. Cutting it is the
  better result, and the tip still touches the wall at the corner depth.
- Two sections cannot tell a draft from a small modelled chamfer: both are a
  plan offset. A third section at half depth checks the straight slope.
  A draft is one straight face, so the half-depth section is at half the
  offset. A modelled chamfer that ends before the corner plane departs
  from this slope. The check permits only the section grid slack.
- The slope limit keeps a 45 degree chamfer that is deeper than the ride
  (it is straight too) out of the drafted reading. 15 degrees is above the
  usual draft of cast and moulded parts and well under a chamfer angle.

## Dropped

- **A slope limit only, with no third section.** A 0.2 mm modelled chamfer
  under a 1 mm ride reads as a 11 degree slope. It would pass as a draft,
  and the sharp sides beside it would be cut.
- **Keep the refusal and change the message.** Correct, but the user must
  pick the edges of each drafted part.
- **Ride the drafted loop in the bevelled group (the exact behaviour before
  #2894).** Then a sharp pocket in a plate with a drafted side is left
  unbroken with no message. A draft is not a modelled chamfer, so the loop
  goes with the sharp loops.

## Review correction

Use the existing `LEAN_SLACK` for the half-depth residual. A 0.25 mm bevel
that ends at 0.7 mm depth passed the former 25% allowance for a 1 mm ride.
The tighter check refuses this partly bevelled loop. The relative allowance
was dropped because it can classify a bevel as a straight drafted wall.

## Second review correction

Check the half-depth section against a point, not a distance. The point is
half way between the top edge and its foot on the wall. A side at X0.2,
X0.3 and X0 (top, half depth, corner) passed the distance check: its slope
turns back, but the unsigned distances agree. The point check refuses this
side. The unsigned distance check was dropped because it ignores the
direction of the slope.

## Third review correction

Match one half-depth loop to the corner wall before reading its edge pieces.
The loop must have the same winding. Choose the closest loop by the maximum
of the vertex gaps in both directions. This comparison uses the whole wall,
so an unrelated hole cannot supply a matching point.

For each piece, require a parallel corner-wall segment with the same direction
that covers the piece's full projection. Derive the expected half-depth
segment from that wall segment and the top piece. Require continuous coverage
of this segment by parallel segments of the matched half-depth loop, with the
same direction, within `LEAN_SLACK`. Join their projected intervals so extra
collinear vertices from tessellation do not change the reading.

Drop the midpoint-only check and the minimum distance over all half-depth
loops. One matching point does not establish agreement along the wall. A
slanted half-depth edge can cross that point, and an unrelated hole can pass
through it. Full coverage on the matched wall keeps those partial bevels out
of the drafted reading.
