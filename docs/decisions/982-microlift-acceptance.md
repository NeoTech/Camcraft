## #982 - Microlift acceptance: remove the sliver, keep the strict law

**What was decided.** The acceptance in `adaptive/lift.rs` keeps its law
with no tolerance (`lift::keeps`: the lifted plan's worst reading may not
exceed the spiral's own, or the bound where the spiral read under it).
What changed is what the two readings are taken on. A lifted piece is
posted on longer chords (`thin_turn`, 0.01 mm sag, 4 degree joints) only
where the planner's stock shows they change nothing: they read within the
bound **and** leave standing no cell the piece's own chords clear
(`lift_coarse::cut`, `Raster::stands_over`). Otherwise the piece is cut
on its own chords, the spiral's. The lifted plan's stock is then never
more than its pieces' own chords leave, so its reading and the spiral's
differ by stock (a different order, a dropped graze), never by the
chords.

**Why not a tolerance, which the issue asked for.** A reading is a count
of rim samples (360 over 180 degrees), so every reading is a whole half
degree and there is no finer reading to compare. The 48.0 against 47.5
was not a tie broken by rounding: it was one real sample of engagement.
A chord cuts inside the arc it replaces, by up to its sagitta, and leaves
that sliver on the stock side; the next piece meets it. At the slot's
bound 0.01 mm of extra radial width is about 0.25 degrees, half a sample,
and a planner cell (D/120) is about two samples wide at the rim - so whether a
sliver shows as zero, one or two samples depends on which cell centres it
covers, and that depends on nanometres of ring geometry. A tolerance
wide enough to absorb it (one sample was tried by the first worker, two
are needed: 48.5 was read here) accepts a plan that overloads past the
stated bound and has to name it; the slot's own pin forbids that note.
Removing the sliver keeps the bound exact: a microlifted plan is never
kept if it reads over the spiral at all.

**Measured (release, Linux x86-64, upstream/main 3e9b8ae1).** "Opening"
is `rings::open` eroding that much deeper than it dilates back; "width"
is the slot itself that much wider. `L/S` is lifted against spiral
reading in degrees; the bound is 47.5014.

| 60 x 14 slot, D10 | before | after |
|---|---|---|
| unperturbed | 47.5 / 47.5 kept | 47.5 / 47.5 kept |
| opening 8, 32, 48 nm | 48.0 / 47.5 dropped | 47.5 / 47.5 kept |
| opening 180 nm | 48.5 / 47.5 dropped | 47.5 / 47.5 kept |
| opening 16, 24, 62, 90, 125, 250, 350, 510, 700, 1000 nm | kept | kept |
| width -125, -62, -8, +32, +510 nm | 48.0 / 47.5 dropped | 47.5 / 47.5 kept |
| width +8 nm | 48.5 / 47.5 dropped | 47.5 / 47.5 kept |
| width -250, -32, -16, +16, +62, +125, +250, +1000, +5000 nm | kept | kept |

Before: 10 of 30 twins dropped the lifted plan (the bare spiral then
fails the travel law and the slot falls to Offset rings). After: 0 of
30, and every one reads its worst at the same place the spiral does
(the first turn out of the entry bore), on the same stock - the tie is
the same reading, not two readings that happen to agree.

Cost: on the slot a third of the pieces keep longer chords instead of
two thirds; the `adaptive_hsm` narrow slot posts 14 534 moves instead of
13 378 (+8.6 %), cut and air lengths within 3 mm. Planning time is
unchanged (the piece's own chords were walked before only where the
longer ones failed; now always, in place of the second walk of the
longer ones).

**The one fixture verdict that changes, and why it must.** The issue's
Done-means asks that no adaptive fixture's verdict change on main. One
does: `rotary_adaptive`'s r6.2 neck levels (radius 5.7 and 5.2), pinned
"refused for travel by name" since #55. That verdict was the same knife
edge, not a decision. On unchanged main, with only the openings
perturbed:

| opening | radius 5.7 | radius 5.2 | the old pin |
|---|---|---|---|
| 0, 8, 125 nm | rings (48.0 / 47.5) | rings (48.0 / 47.5) | green |
| 16, 62 nm | **HSM-planned** | rings | red |
| 250, 510 nm | rings | **HSM-planned** | red |

No acceptance rule can keep the slot at 48.0 against 47.5 and refuse
these bands at 48.0 against 47.5; they are the same case (a channel a
few tool diameters wide, kept in cut only by its microlift returns).
After the fix both levels read 47.5 against 47.5 and are HSM-planned at
every perturbation tried (0, 16, 32, 250 nm). The pin is rewritten as
`the_r62_neck_levels_are_hsm_planned_on_microlift_returns`. What they
were cut by before is Offset rings at a full-width load (100 % D, 15-21
breaks of 45-135 degrees); what they are cut by now reads 20.3 % D on
the suite's 0.1 mm dexel and no planner overload.

Two readings of those plans are pinned at what they are rather than at
the whole bar's laws, and are **not** this change's doing (the same
readings appear on main whenever the coin lands the other way):
- **20.3 % D** on both levels, against the whole bar's 19.8 % (optimal
  + 35 %). It is one move, the first turn out of the entry's bore
  (55.5 degrees on the 0.1 mm dexel; the r7 bar's lifted level reads
  53.0 at the same place; the planner's own 0.05 mm stock reads 47.5).
  The pin holds it to optimal x 1.40.
- **3 in-cut breaks** past 5 degrees on radius 5.2 (5.1, 5.5, 5.5
  degrees at the band's west end; the spiral's own chords); radius 5.7
  has none. The pin holds 5.2 to at most 3.
Both are filed as #1050 (the developed-band bore exit and end kinks);
the pin's two constants (`R62_LOAD_FACTOR`, `R62_BREAKS`) go when it
is fixed.

**Every other verdict is unchanged** (release, `--include-ignored`):
motion lib, `adaptive_hsm` 6/6 (bore, rectangle, L-pocket, island,
narrow slot, channel: same strategy, same load, same breaks),
`adaptive_band` 2/2, `pocket3d_hsm`, `level_links`, and the other four
`rotary_adaptive` tests. The L-pocket and the rotary skin levels still
drop their lifted plans, now on what is left once the chords are out of
it: those plans leave more stock than the spiral (19 to 513 cells: air
stretches that grazed), read 48.0 to 49.5 at every perturbation, and
are real overloads the strict law is there to refuse.

**Goldens.** No golden file moves (`tests/golden/` holds no HSM plan).
Posted programs of lifted plans change (more pieces on their own
chords); the suites that pin them pin laws, not digests.

**Considered and dropped.**
- *One- or two-sample tolerance* (the issue's proposal; the first
  worker's first try). Keeps the slot, but posts plans reading over the
  stated bound and must name them; where the edge goes next (two samples,
  three) is again a matter of nanometres.
- *Fine-chord retry* (the first worker's held commit 39a06802): re-plan
  the whole lifted plan on fine chords when the coarse one fails. Same
  verdicts as this change, but it plans twice, gives up every longer
  chord in the plans it rescues (the slot keeps a third of them here),
  and leaves the sliver in every plan that happened to pass.
- *Sub-sample readings* (bisecting the arc ends): still quantized by the
  cell, and costs every walk.
- *Headroom in the spiral* (grow to the bound less a sample, accept up
  to the bound): moves every grown plan in the product by ~4 % more
  revolutions to protect the lifted ones.
- *Keeping the r6.2 levels refused by a rule of their own*: nothing
  honest tells them from the slot at plan time, and the refusal sends
  them to a full-width ring cut.

**What it would cost to change later.** `lift_coarse::cut` is the one
place the chords are chosen; loosening it (say, slivers up to n cells)
is one comparison, and the slot's sweep
(`the_slot_keeps_its_verdict_under_sub_micron_perturbations`) says at
once whether the knife edge is back.

**Not settled here.** #882's other two knife edges stand: the #850
planning budget (the rotary window's and the r7 bar's deepest levels
flip between "refused past the budget" and HSM-planned when the
openings are 250 nm deeper, on main and after this change alike; the r7
pin is red there on both - filed as #1051) and #845's island (PR #953).
`grow.rs` and `budget.rs` are not touched.
