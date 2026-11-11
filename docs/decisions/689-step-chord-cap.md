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

**An over-budget model, both directions (review round 2).** A model that
hits the budget stops short of the 0.005 mm target. It keeps a mesh
coarser than the cap and only logs "reduced fidelity" to stderr. The
facets are chords inscribed in the true surface, so the error cuts both
ways:
- **Concave wall:** the facets stand toward the tool, and a nominal
  finish can read as a false gouge. This is the safe side.
- **Convex wall:** the facets sit inside the true surface, so the mesh
  wall looks farther from the tool than the real wall is. A real gouge,
  up to the model's actual chord error, can pass. This is the unsafe side.

This PR does not introduce that risk. I verified this in `step.rs`:
- Before #689 the target was `extent/1000`, and the refinement passes
  (start `extent/4`, halve, clamp at the target) were the same.
- The new target is never larger than the old one. Up to the old target
  the pass sequence is identical, and the budget stop is the same check
  on the same meshes.
- So every STEP mesh is now at least as fine as it was before, for every
  model.

This PR removes the risk for every model that fits the budget. For an
over-budget model it remains: tracked in #864 (record the achieved chord
error and tighten the floor by the excess; flag the model visibly and
gate the post).

**`MAX_ARC_CHORDS` and the quarter reading.** `cl::arc_chords` clamps an
arc at 1024 chords.
- At the quarter reading (0.0025 mm), a full circle hits that clamp above
  r ~ 531 mm. For a sweep θ the radius is about `531 · (2π/θ)²` mm. The
  old 0.01 mm reading hit it above r ~ 2125 mm.
- Below the clamp the reading stands within 2.5 µm of the arc.
- Above it the reading degrades toward the old one, never past it: it
  takes at least as many chords as the old reading, and its worst
  sagitta is never larger.
- Pinned by
  `verify_arc_reading::the_quarter_reading_is_never_coarser_than_the_old_one_at_any_radius`,
  over radii from 5 to 3000 mm.


**The chord cap is in millimetres; the mesh is made in the file's unit
(review round 3).** `step.rs` tessellates each shell in the STEP file's
own length unit and scales the mesh to mm only afterwards (#306). The
cap was compared in file units, so a bare 0.005 meant different things
per unit:
- **Inch file:** capped at 0.005 in = 0.127 mm, so the `extent/1000`
  target ruled. The inch copy of Milling-test.step kept 0.022 mm chords
  on the Ø25 hole, and the #689 bore was still refused with the 20 mm
  gouge.
- **Metre file:** capped at 5 m, which never bound. sample_ap214.step
  stayed at about 0.03 mm chords.

Now the cap is `MAX_CHORD / mm_per_unit` in file units. Every other
constant in the function was checked:
- `extent/1000` is relative and needs no conversion.
- The coarsen bound `extent * 16` is relative.
- `MIN_TOL` is in file units, because it guards truck.
- The reduced-fidelity log now prints mm. It fires when the achieved
  tolerance exceeds the unfloored target.

`MIN_TOL` rose from 1e-6 to 1e-5 file units. At 5e-6 m truck split the
metre sample into a non-manifold body: its vertex weld tolerance is
1e-6 model units. As a result:
- A metre file reaches 0.01 mm, not 0.005 mm. That is better than the
  ~0.03 mm before, and it is logged as reduced fidelity.
- Millimetre and inch files are unaffected.

The full fix for large units is to scale the B-rep to mm before
tessellating. It belongs with #864, where an achieved chord error above
the cap tightens the verdict.

## Review round 4: law (b) reads the union of both joint sets outside Area mode

Chosen: outside `Mode::Area`, law (b) reads an arc at the union of its
chord joints at the whole tolerance and at the quarter
(`sim::verify::section::segments`). The high-tier review (B1) showed the
two joint sets are not nested: 3 chords at `tol` became 5 at `tol / 4`,
and a sharp island vertex on an old joint fell between two new ones, so
a gouge 50 um past the floor that the old reading convicted passed.
Joints lie on the arc and never read closer than the truth, so the union
adds no false gouge and keeps every old conviction. Area mode is
unchanged.

Dropped: splitting each old chord in two (nested, but the 1024-chord
clamp breaks it on large arcs). A whole-arc reading against wall edges
would close the remaining point-sampling gap; that is #905, not this PR.
