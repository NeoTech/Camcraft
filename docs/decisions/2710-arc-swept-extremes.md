# docs/decisions/2710-arc-swept-extremes.md

## #2710 - The travel check judges the quarter points that an arc sweeps

Date: 2026-11-19. Taken unattended by the issue worker.

**What was decided**

`machine::judge::move_extremes` returns the two endpoints of an arc, plus a
quarter point of its circle only when the sweep passes that point. The sweep
runs from `from` to `to` in the direction of `arc.ccw`, in the arc's own
plane. This is the same reading that `cl::plane::sweep` gives to the chords
and that the post writes as G2/G3.

The check stays strict. It keeps all four quarter points in these cases:

- The in-plane endpoints are 0.01 mm apart or less (`SAME_WORD`). Their NC
  words can be equal, and a control turns equal endpoints a full circle.
- The sweep reads zero or is not finite (endpoints on one ray, a bad center).

The radius is the larger of the two endpoint radii. The old code used the
radius at `from`. A ramping arc (a thread ramp) can be larger at `to`, and a
selective check must not read it smaller than it is.

A quarter point within 1e-9 rad of an endpoint counts as swept
(`ANGLE_SLACK`). No test can show this guard alone, because the endpoint
itself is always judged. It costs nothing and it cannot hide a point.

One function now serves XY, XZ and YZ arcs. `ArcPlane::uvw` is the identity
for XY, so the XY values do not change.

**Why**

A four-decimal inch post has a 0.00254 mm step on each axis. Endpoints
less than 0.0036 mm apart can round to the same point on a diagonal.
The 0.01 mm bound includes margin and agrees with `ring_zig::MIN_ARC_CHORD`.

A shallow arc of a 22 m radius is a legal move. The old check judged points
22 m away from the tool and refused the program on each machine size. The
issue's reporter and the triage comment asked for the swept reading.

**Considered and dropped**

- A 0.001 mm endpoint bound. It misses endpoints that round to one point
  with four-decimal inch words or three-decimal mm words.

- A cap on the arc radius, or a switch to chords for large radii. Two
  readings of one arc can disagree; the swept quarter points are exact.
- A call to `cl::plane::sweep`. It is `pub(crate)` in `camcraft-cl`, and the
  judge needs the start angle, which `ArcMove::radius_sweep` does not give.
  A wider `camcraft-cl` API is not needed for six lines of angle math.
- A change to `#1882` (program bounds) and to `#2585` (the move of
  `judge.rs` to `camcraft-post`). They are separate issues.
- A check for a non-finite quarter point. The comparison with the stroke
  already passes NaN silently in the callers; that is older behaviour and a
  separate defect, if it is one.
