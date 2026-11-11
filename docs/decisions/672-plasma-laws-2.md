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
  no lead-out ends on its ring) is not a tip-up; only an endpoint strictly
  inside, or a proper crossing, is.

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

**5. The machine check's note** (#674) now names where the through-cut IS
judged: the plasma laws at the post gate, the kerf and cover laws at
generation, and that the 3D stock sim does not run for a torch.

**Cost to change later:** the laws read a neutral `PlasmaProgram`; a
post-side reader (the `.cnc` stream, #679) could build the same program
from its torch records and judge the posted text too, without touching
the laws.
