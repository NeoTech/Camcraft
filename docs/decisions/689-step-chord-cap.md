## #689 - STEP meshes no coarser than half the gouge verdict

**Chosen:** the STEP importer's chord target is
`min(extent * 1e-3, MAX_CHORD)`, where `MAX_CHORD = DEV_TOL / 2` = 0.005 mm.
Before, it was `extent * 1e-3` alone. The program sim judges the program
against the imported facets. On a concave wall a chord stands inside the
true surface by its sagitta, so a bore that cut the true Ø25 hole cut
about 0.022 mm of mesh "material" on Milling-test.step. That is past the
verdict's 0.01 mm, and the column was charged the wall's full 20 mm
height. With half the verdict spent on the mesh, the other half is left
for the cut: a nominal bore passes and a bore 0.05 mm oversize is still a
gouge.

**Why this and not a looser verdict:** granting the gouge check the
model's chord tolerance as extra slack would forgive real gouges up to
`extent / 1000`. That is 0.06 mm on this 60 mm part and 0.3 mm on a 300 mm
one. The issue's own "Done means" (0.05 mm oversize still refused) rules
it out, and so does the Prime Directive. Making the mesh as fine as the
verdict claims keeps the verdict honest.

**Cost:** curved faces get more facets. Milling-test.step went from 448 to
952 facets (9 ms to 16 ms import); sample_ap214.step stayed at 1550. On a
large curved part the chord edge at 0.005 mm is still about 2 mm on a
100 mm radius. The existing facet budget (refinement stops at budget / 8,
with the 30 M cap) still bounds any model, and the importer logs when it
stops short of the target.

**Not covered:** STL and OBJ imports come pre-tessellated, so their
fidelity is whatever the file has. A coarse STL can still read a nominal
finish on a concave wall as a gouge. That would need the verdict to know
the file's chord error, which isn't recorded anywhere.

## #689 - the arc fit and the arc reading leave the mesh its share

The chord cap alone moved the error elsewhere: five suites running
contours on Milling-test.step reported 0.010-0.012 mm gouges. Three
budgets were each allowed the whole gouge verdict (0.01 mm) and summed:
the mesh's chords, the arc fit's error, and the verifier's reading of an
arc by its inscribed chords. The coarse mesh had hidden the other two.

**Chosen (orchestrator's option 1, taken unattended):**

1. **Arc fit, vertex share** (`motion::machining2d::arcs`,
   `ARC_FIT_VERTEX_SHARE = 0.5`): every vertex an arc replaces stands within
   half of `tol` of the fitted circle. A vertex off the circle is a point the
   plan verified that the arc passes beside, toward the part as often as
   away. The chord-hug test (the arc's bow off its own chords) keeps the full
   `tol`, because it measures the tessellation's own sagitta. On the
   solid_law contour the R35 arc's centre was 0.027/0.040 mm off the origin
   at the full budget; at half it is 0.004/0.006, and all ten gouges are gone.
2. **Closing rings, turning direction from signed area** (same file): a run
   whose ends meet within `tol` takes the anchor branch, and its direction
   comes from its shoelace sign, not from the anchor triple. On a regular
   rim the two quarter-turn anchors tie, the first one found lies before the
   pivot in travel, and the triple read the ring backwards. Every closed rim
   therefore ended on a chord. On the Ø25 hole of Milling-test.step that
   kinked the seam tangent, and the lead-out crossed the ring (nine
   `Crossing` findings on solid_law once the gouges were gone). A closed
   rim now fits whole.
3. **Tangency law reads arcs at a quarter of the verdict**
   (`sim::verify::section`, `ARC_READ_SHARE = 0.25`): law (b) read an arc
   move by its inscribed chords at the full gouge tolerance. Those chords
   stand up to `tol` inside the arc, which pushed a nominal arc round a
   convex wall over the line (shoulder_adaptive's Offset-ring fallback, four
   gouges) and hid up to `tol` of real gouge where an arc bows into a
   concave wall. The verdict, the floor `r - leave - tol` and every other
   law are unchanged. Only the reading moved closer to the cut arc, and it
   is stricter on the concave side than before.

**Dropped:**
- Option 2, giving the program gate the model's chord error as slack. It
  forgives real gouges up to `extent / 1000` and fails "0.05 mm oversize
  still refused" on larger parts.
- Option 3, mesh-only tuning. It is a knife edge both ways.
- A side-aware fit (deviation only toward the part kept small). It needs
  every `fit_arcs` caller to know its material side, and pockets and plasma
  have walls on both sides. The symmetric half share is the bounded form
  of the same rule.
- Loosening the solid law's allowance or `GOUGE_TOL`. Never.

**Cost:** more short arcs become lines. This is always safe, because lines
are the verified polygon. Nine `golden_cl` goldens moved and were
regenerated in their own commit. The PR lists which goldens moved and why.
Law (b) reads arcs with about twice as many chords.

**Verdict flip, named (review of PR #839).** The quarter reading turns
some former gouge verdicts into passes, and some former passes into
gouges.
- **Former gouge, now a pass:** a nominal arc round a convex wall. With
  the r = 5 cutter on r = 34.998 round an r = 30 island, the true
  clearance is 4.998 against a 4.99 floor. The old chords read 4.988 and
  convicted it.
- **Former pass, now a gouge:** an r = 20 arc bowing 5 um past the floor
  into a straight wall. The old 5-chord reading put the apex mid-chord,
  0.0096 mm low, and passed it.

Both flips are pinned in `crates/app/tests/verify_arc_reading.rs`. With
the share set back to 1.0, exactly those two tests fail. Arcs cut past
the floor round a convex wall and inside a concave one are convicted
under both readings.

**STEP timing (debug build).**

| Model | Before (no cap) | After (0.005 mm cap) |
|---|---|---|
| sample_ap214.step | 1550 facets, ~81 ms | 1550 facets, ~64 ms |
| Milling-test.step | 448 facets, ~14 ms | 952 facets, ~27 ms |
| tool_csys_box_iso13399.step | 12 facets | 12 facets |

The facet budget bounds a huge model:
- Refinement passes run only while the mesh is at most `budget / 8`
  (`MAX_FACETS` = 30 M, less what earlier shells used).
- A pass at most quadruples the count, so a kept mesh stays under about
  `budget / 2` (15 M).
- Any triangulation past the budget is dropped and the last mesh is kept.
- There are at most 24 passes.

Such a model stops short of the 0.005 mm target and logs "reduced
fidelity". Its concave walls can then still read a nominal finish as a
gouge, as STL/OBJ can.
