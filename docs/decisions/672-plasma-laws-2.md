## #672 - plasma laws, step 2: the CL reader, the post gate, the cover law

Sources (clean room, behaviour only): Tormach's *1300PL Operator's Manual*
UM10720 v1223A (§1.4.2, §4.12.8, §4.13 - tip-up, the torch over an open
kerf, kerf ≈ 1.5x the orifice) and the LinuxCNC Plasma Primer §12 (torch
height control diving into a cut kerf), as cited on the issue. No vendor
CAM output or implementation was read.

**1. The program is read from the records, never from text.**
`PlasmaProgram::from_toolpath(&Toolpath)` (`sim::verify::plasma::program`)
builds the laws' input from `Toolpath::plasma` - the kerf and each
`TorchSpan`'s lit moves, chorded at 0.01 mm sagitta, with the torch-off
travel between spans as the rapid into each cut. A path with no record, or
a record `PlasmaCut::is_sane` rejects, is refused by name: nothing is
guessed from bare moves. *Dropped:* re-parsing the `.cl` text (the records
round-trip the cache already, #676 decision 11).

**2. A refusal blocks the post the way a gouge does.** The gate's new
plasma clause (`posting::plasma_clause`, `posting_plasma.rs`) runs inside
`verification_report`, so every surface that reads the gate - the post,
the per-op and per-mount posts, the script API, the status line - blocks
on a plasma refusal ("post blocked - Plasma1: plasma cut order: ..."). The
3D stock sim carves nothing for a torch (it skips the op); that skip is now
SAID: the report's not-judged line reads "stock sim did not judge this
program: the 3D stock sim does not run for plasma - Plasma1 judged by the
2D plasma laws (cut order, cut crossing, kerf crossing, tip-up): clean"
(or the warnings, named). A plasma op with no path, or a path with no
record, is a blocking problem of its own - two missing judges never add
up to a pass (the `posting_sim` law). *Dropped:* a new `GateReport` field
for the plasma line - the existing `not_judged` line already rides to
every consumer, and a plasma program IS a program the stock sim did not
judge; when both lines exist they are joined with `; `.

**3. The laws, step 2.** Beside step 1's cut order (refusal), kerf
crossing (warning) and tip-up (warning):
- **Cut crossing (refusal).** A later contour's RING properly crossing an
  earlier contour's ring: the kerf runs through a part already cut - the
  through-cut's 2D gouge. The ring is the stretch of a cut between the
  first point it returns to and that return (leads and a small hole's
  overburn fall outside it), so a lead running over an earlier kerf stays
  the torch-height warning, not a gouge.
- **Enclosure is recomputed from the rings.** Each cut's parent is the
  smallest earlier-or-later ring enclosing every point of its ring (its
  path, for an open run or a one-point small hole). A `PARENT` the
  geometry disagrees with - names a parent but lies in none / in another,
  names none but lies in one - is refused by name. The cut-order law runs
  on the verified parent.
- The review's notes (#675): `small` only on an `INNER` contour; a
  one-point cut without `small` is "no cut path"; a parent that is an
  open run is malformed; a closed-role cut whose path never returns to
  its start is malformed.
- A rapid that *leaves from* the edge of the part just freed (a cut with
  no lead-out ends on its ring) outward is not a tip-up. **Review fix
  (#791):** "an endpoint strictly inside or a proper crossing" let a
  rapid in or out of a part *through a corner* go unjudged (a corner hit
  is never a proper edge crossing, and parts nested on a grid make exact
  corner hits common). Now the rapid is split at every contact with the
  ring - a proper crossing, a vertex it passes through, the ends of a
  stretch along an edge - and judged piece by piece: a piece whose
  midpoint lies strictly inside is a tip-up, reported at that midpoint.
  The departure exemption falls out of it (every piece is outside); a
  rapid that leaves the edge and then runs over the part does not. Pinned
  by the review's three fixtures.
- **Cut crossing, review fix (#791):** proper crossings alone missed two
  overlapping rings that meet only at vertices or along collinear edges
  (`square(0,0,50)` then `square(25,0,50)`): the later ring ran through
  the earlier part and only the torch-height *warning* fired. Now a later
  ring with a piece strictly inside the earlier ring AND a piece strictly
  outside it (the same split, on each of its edges) is a crossing too,
  reported at the inside piece. A ring wholly inside is enclosure (the
  cut-order law's); two parts sharing an edge enter nothing and stay
  legal common-line cutting (pinned). *Dropped:* sampling only vertices
  and edge midpoints, which misses an edge entering at one vertex and
  leaving at another with its midpoint on the ring.

**4. The cover law (#676 decision 12), at generation.** An outward offset
bridges a slot or notch narrower than the kerf (and an inward offset drops
a keyway narrower than the kerf from a hole) as ONE loop, so the ring law
accepted it and the feature was silently left in the material. Now
`motion::plasma::laws::cover_law` samples every edge of every closed
contour at 0.25 mm and refuses by name any point farther than one kerf
width (2r + `KERF_TOL`) from its own cut's lit chords: "a slot, notch or
inside corner narrower than the kerf is bridged by the offset, not cut".
*Why one kerf width:* an inside corner the torch rounds off leaves
r(1/sin(φ/2) − 1) at its vertex - under r for any opening of 60° or more,
the fillet every round torch leaves and every CAM accepts - while a
bridged slot's floor stands its whole depth away. *Dropped:* the exact
right-angle fillet allowance r(√2 − 1), which would refuse every acute
inside corner the torch can only round; a closing-area measure
((contour ⊕ r) ⊖ r minus the contour), whose area does not separate a
shallow notch from a sharp corner's sliver. The law lives beside the kerf
law in the generator (the nominal contours are there; the CL has no part),
in `laws.rs`, with `kerf_law` moved out of `emit.rs`.

**4a. Sharp inside corners are not refused (review of #791; decided by
the main session).** The 2 r allowance alone refused every inside corner
under 60°, whatever the width of its mouth - a 20 mm wide 30°, 45° or 55°
V-notch, a chevron, the crotch of sign lettering - with a message that
blamed a narrow slot and told the user to widen it, which does not help
(only opening the angle would). That is ordinary plasma geometry: every
inside corner leaves a kerf-radius tip uncut, and every CAM accepts it.
*Decided:* the cover law exempts each inside corner's **tip zone** and
judges only the material beyond it. The torch, radius r, can get no
nearer an inside corner of opening θ than r / sin(θ/2) along the bisector
(where the corner's opening equals the kerf width, 2 r), and its circle
there touches each wall r / tan(θ/2) = r · tan(δ/2) from the vertex
(δ = π − θ, the turn at the vertex); beyond that point the wall is
reached at exactly r. The stretch of wall that even this best possible
path leaves farther than 2 r away is r (tan(δ/2) − √3) from the vertex -
the exempt zone. It is empty for openings of 60° and more (so the
right-angle notch and keyway corners get nothing), 0.14 mm at 55°,
0.51 mm at 45°, 1.5 mm at 30°. An inside corner is a right turn on an
outer ring or a left turn on a hole (both rings run counter-clockwise);
convex corners and straight runs have no zone. *Why the tighter measure
and not r / sin(θ/2) as a radius:* it is derived from the law's own
threshold - what the ideal miter path cannot cover, nothing more - so a
real path worse than the ideal at a corner is still refused, and the 1 mm
notch and the 1 mm keyway (90° corners) are refused exactly as before.
*Consequence accepted:* a V-notch whose mouth is under the kerf is wholly
a corner tip and generates with the tip left, like any sharp corner; its
walls are shorter than the zone, so no path could reach them. The
refusal now names the gap it measured against the kerf ("a slot, notch
or keyway about 1.000 mm across, narrower than the kerf (1.500 mm), is
bridged by the offset"): the nearest edge of the same contour that is
not the point's own edge or its neighbours, when that is under the kerf.
Pinned: 30°, 45° and 55° V-notches 20 mm wide generate; the 1 mm notch
and keyway are still refused, the 3 mm notch still cuts.

**5. The machine check's note** (#674) now names where the through-cut IS
judged: the plasma laws at the post gate, the kerf and cover laws at
generation, and that the 3D stock sim does not run for a torch.

**Cost to change later:** the laws read a neutral `PlasmaProgram`; a
post-side reader (the `.cnc` stream, #679) could build the same program
from its torch records and judge the posted text too, without touching
the laws.
