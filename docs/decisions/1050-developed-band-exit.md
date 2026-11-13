## #1050 - The bore exit is held on a coarse rim too; the closing laps' kinks are spread

**What was decided.** Two changes in the growth planner, both in the new
`adaptive::conform` (two call lines in `grow.rs`):

1. The disc phase - the revolutions out of the entry bore, circles about
   the seed - is read on the planner's own rim and on the rim of a
   yardstick twice as coarse (`COARSE` = 2 planner cells), the second
   reading converted back by the straight-wall law (`load::deg_at_rim`),
   and the worse of the two is what the revolution is held to.
2. The closing laps are spread by `tidy::smooth_walk` at 4 degrees (the
   star spiral's law, `STAR_SMOOTH_DEG`) before the walk is lifted and
   read again on the stock.

With them the measured pins `R62_LOAD_FACTOR` (1.40), `R62_BREAKS` (3)
and `BAR_50_BREAKS` are removed from `rotary_adaptive.rs` and the budget
twins: the r6.2 neck levels and the r7 bar's level 5.0 are held to the
whole bar's laws (load within optimal + 35 %, no in-cut break past
5 degrees). No law and no pin's bound was loosened.

**Cause, measured (release, Linux x86-64).**

- *The bore exit's 20.3 % D* was not a planner overload but a
  conforming-contact reading. Out of the bore the tool (r 3) stands
  inside a cleared disc of radius about 4, a wall curving the same way
  as its rim. There the contact arc grows with the rim's radius faster
  than the straight-wall law converts it. The same r6.2 level 5.7
  program read on a dexel of D/120 (the planner's cell): 47.5 deg (the
  planner's bound); D/80: 51.5; D/60 (the suite's): 55.5 = 20.3 % D.
  Worst move: the second revolution out of the orbit, arc
  (21.685, 7.722) -> (20.753, 6.837).
- *The 5-6 degree breaks* were the closing laps' own chords: where the
  corner fillet (R1.25) hands over to a straight wall run, one longer
  junction chord (0.19 mm against 0.054) turns 5.1-5.5 deg (headings
  99.9 -> 94.4 -> 90.0). The arc fitter leaves them chords.

**Readings.** r6.2 level 5.7: 20.3 % D / 0 breaks -> 17.3 % / 0. Level
5.2: 20.3 % / 3 -> 17.7 % / 0. The r7 bar: 6.5 and 6.0 read 19.5 %, 5.5
17.3 %, 5.0 18.4 %, 0 breaks on every planned level (law 19.8 %). The
window's 6.0 and 5.0: 18.4 %, 0 breaks.

**What it moved elsewhere.** The changed disc phase moves every grown
plan's first revolutions. On `adaptive_hsm`'s fixtures the readings stay
inside their laws (bore 15.4 % D, rectangle 15.4, L-pocket 15.7, narrow
slot 15.4; 0 breaks). On the narrow slot the new plan exposed two latent
faults of the microlift plan - a rapid landing on a wall strip thinner
than a planner cell, and a retract-and-plunge on one spot between two
abutting pieces - which are #1169's, fixed in the same PR
(`docs/decisions/1169-landing-on-swept-stock.md`).

**Considered and dropped.**
- *Smoothing the whole grown walk*: 24 umbrella passes shrink every ring
  by its sagitta and the neck levels lose their microlift plan.
- *Measuring on a finer dexel*: that loosens the yardstick.
- *A tolerance on the bore exit*: a pin by another name.
- *Refusing the neck levels to Offset rings*: the planner holds the law
  there once the contact is read on both rims; the rings read 100 % D.
- *Scoping the coarse-rim hold so it leaves the slot's plan alone* (the
  way out named when the work was escalated): it would have hidden
  #1169's landing fault behind an edge drawn for that purpose; the fault
  is fixed instead.

**What it would cost to change.** `COARSE` names the coarsest yardstick
the disc phase is promised on (D/60 for a D/120 plan). A suite measuring
coarser than that can read the bore exit past the law again; the remedy
is the constant, at the price of smaller first steps out of the bore
(more revolutions in the disc phase, a few percent of a plan's length).
