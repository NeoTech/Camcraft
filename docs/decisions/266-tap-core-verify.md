## #266 - The generator's verify judges a tap's cutting moves at its thread's core

**Chosen:** after the sim fix (PR #713, austin62jake: the program sim sweeps
a Tap-kind tool at `tool_law::cutting_radius`, ISO 724's basic minor
diameter), the op's own verify still convicted a tap on a tap-drill hole:
its tangency and solid laws read the full Ø6 against the Ø5 wall
("move 1/2 cuts past the finish surface"). Generate now re-runs the same
cutting laws (`verify_bodies` + the containment solid law) with the tap cut
to its core and drops each `Gouge` the core run does not also raise
(`operation::tap_core::acquit`). Only `Gouge` is re-read; rapids, shank,
holder and fixture verdicts keep the full diameter, because a tap must never
rapid into its thread band.

**Why:** the threshold is the basic minor diameter D1 = d - 1.0825 p, which
sits at or under every tap-drill size, so a hole drilled for the thread
passes and a hole under D1 (by more than the gouge tolerance), a tap over
solid metal, or a tap deeper than its hole stays convicted. An acquittal
pass (remove convictions only) cannot make the verdict less safe than the
core reading itself, and needs no new field in `VerifyContext` (built in
46 files).

**Scope:** Z drill cycles with a Tap-kind tool, upright frame only. The
helical thread mill (`Thread` op), cross holes and tilted (3+2) taps keep
the full-diameter verdict - unexercised against modelled holes, and
austin62jake's note shows the thread mill has other findings there; they
over-report, never under-report.

**Dropped:** a second radius in `VerifyContext` (wide churn for one tool
kind); shrinking only the containment law's cutter (the tangency law still
convicts - tried on #271); changing the `solid_law_goldens` convention that
models a tapped hole at the major diameter (still passes: a re-read only
removes warnings).

### Review rework (PR #1289, 2026-11-12): the acquittal covers the thread band only

- **Chosen:** for every move whose full-diameter `Gouge` the core run drops, carry the core run's body verdicts on that move (`ShankHit`, `SweepHit`, `HolderHit`, `FixtureHit`), one per kind and move. The core re-read is also limited to the tapping cycles (`Tap`, `FloatingTap`, `Auto`); a Tap-kind tool in a Drill/Peck/Bore cycle keeps its full verdict.
- **Why:** the solid law reports a move once (`Gouge`, else the body), so a full-diameter `Gouge` hid a Ø6 shank fed past a 4 mm thread length into a Ø5 hole; dropping the `Gouge` posted a real shank collision. Pinned by `a_tap_whose_shank_rides_into_the_hole_is_still_convicted`.
- **Dropped:** carrying every core-run warning anywhere (a narrower cutter can raise band-reach shank flags on moves that never gouged, false convictions on correct taps); keeping the full run's `Gouge` whenever the tool's shank is at least the core (blocks valid DIN 376 taps with reduced shanks too coarsely).
