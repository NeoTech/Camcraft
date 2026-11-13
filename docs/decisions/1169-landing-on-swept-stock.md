## #1169 - A rapid lands only on stock the cut pieces swept, proved exactly

**What was decided.** The microlift plan (`adaptive::lift`) keeps a
second, exact record of the stock beside its cell raster: the segments
its pieces have cut so far (`adaptive::lift_swept::Swept`). The one
return that comes down in rapid - the return over the retract plane,
`LinkLaw::high`, which rapids to its lift 0.3 mm over the floor and
ramps down the piece's lead - now lands only where the whole cutter
footprint (radius r less 10 um, `EDGE`) lies within r less 12.5 um
(`FIT`) of a segment already cut. The proof is a quadtree over the disc: a
square is covered when its center is within the bound less its
half-diagonal of a cut segment (the distance to a segment set is
1-Lipschitz), divided to 1 um, and the landing is refused where the
proof does not close. Nothing is sampled, so a strip of any thickness
is seen. The cell test stays as the cheap first reading.

Two smaller changes ride with it:

- The hop's landing (`lift_link::landing`) takes a lead point the
  pieces swept before one only the cells read clear. With none proved
  the cells' choice stands: the hop comes down at plunge feed from over
  the stock (`ladder_links::pass_link`), never in rapid, so that case is
  a descent the reader counts, not a rapid into stock.
- A piece that starts exactly where the tool stands is ridden on to at
  the floor (`lift::ride`), no longer reached by a retract and a plunge
  on one spot.

**Why.** The planner's stock is a bitmap of cell centers (D/120 =
0.083 mm on the suite's D10). The easing laps are split at their air
stretches as that bitmap reads them, so a wall strip thinner than a cell
that falls between two rows of centers is "air", is dropped from the
plan, and stands until the last lap takes it. On the narrow slot 14x60
with #1050's change the last lap's lead ran along such a strip (0.075 mm
at y = 13.881 to 13.956; the cell row at 13.958 is just past the
footprint). `high` read the landing (17.407, 8.956) clear on the cells
and rapided to 0.3 mm over the floor with the strip under the rim: the
verifier's `RapidInStock` at move 14089 (footprint point 5.064 mm from
every earlier cut on the level). The verifier was right; the posted
program and the stock the planner read differed by less than a cell.

The second reading on that slot, "4 descents into stock for 2 levels",
was not the strip. The growth's last piece and the first easing lap
both stood in cut at the point they share, (5.135, 7.312); no lifted
return lands in cut, so the plan fell to the ladder's hop: a retract
and a plunge on the same spot, at the cut front, which the reader
counts (39 deg on the plunge). The walk runs on there - riding on is
the spiral's own path, and it removes the hop.

**Readings (release, narrow slot 14x60, with #1050's change).**

| | verifier | descents (2 levels) | returns | over the retract plane | rides | hops | out of cut / cut |
|---|---|---|---|---|---|---|---|
| before | 1 `RapidInStock` | 4 | 200 | 3 | 0 | 1 | - |
| after | silent | 2 | 199 (1365 mm) | 0 | 2 | 0 | 0.88 (law 1.5) |

Load 15.4 % D and 0 breaks, as before. The slot's three returns over
the retract plane all landed on leads along the walls; none is proved
at `FIT`, and each is now a lifted return routed through the cleared
area (a feed, 0.3 mm over the floor). A proof costs 250-750 squares,
5-25 ms, and is asked a few tens of times a plan.

**Considered and dropped.**
- *A wall-clearance test on the landing* (footprint off the wall cells
  too; tried by the previous worker): changes nothing, the strip is not
  on the grid.
- *A margin on the cell test* (land only a cell or more off any stock or
  wall cell): refuses every landing beside a wall, where the spiral's
  air leads mostly run, and still proves nothing about a strip between
  two rows.
- *A finer planner grid*: moves the unseen thickness, never removes it,
  and costs every plan.
- *A sampled footprint like the verifier's* (73 points): the planner
  would then pass exactly what the verifier's samples miss; the proof
  must be stronger than the check, not equal to it.
- *Proving the ramp down the lead on the swept stock too*: the ramp is a
  feed above the floor - a strip under it is cut, not collided with -
  and the spiral's air leads ride the cleared outline tangentially, so
  an exact law there would refuse most returns. Left on the cells.
- *Loosening the verifier or the descents law*: never.

- *`FIT` at 5 um* (the arc fit and a like chord sag; the first cut of
  this fix): it proves a landing on an earlier cut's own path and kept
  the slot's three returns over the retract plane, verifier silent. But
  the verifier flattens a posted arc at its gouge tolerance, 10 um
  inside the arc, so a landing proved with under 12.5 um to spare could
  still be read as stock by it. The law must hold on the verifier's
  reading, not beside it.

**What it would cost to change.** `EDGE` (10 um) is the gouge tolerance
the mill laws' footprint is shrunk by; `FIT` (12.5 um) is the adaptive
arc fit (2.5 um) plus the chords the verifier reads an arc on (10 um).
`FIT` is past `EDGE`, so a landing on an earlier cut's own path is
proved only where neighbouring cuts swept the stock about it - anywhere
but along a wall. Along a wall `high` therefore gives way to the routed
lifted return or the hop: more lifted travel in a slot (1349 -> 1365 mm
here), never an unsafe landing. A verifier that read arcs exactly would
let `FIT` drop to the arc fit and return those landings. The swept
record holds one entry per posted chord of the level (bucketed by r);
it is dropped with the plan.
