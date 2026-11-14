## #1595 - An Engrave or Trace on the finished part answers to a mark law of its own

**Decided:** `depth_law_of` gives `OpParams::Engrave` and
`OpParams::Trace` a new `DepthLaw::Mark` whenever the source is the
finished part, whole model or pick (`crates/app/src/operation/mark_law.rs`).
The law: under the cutter, the part's own surface may give way by the
**mark budget** and no more,

    budget = min(Top - Bottom, cutter radius)

It is tested with the existing solid law (`verify::solid_hits`, the
tool's own tip and stack swept against the mesh) run against the part
LOWERED by the budget. A cutting move whose edge still enters the
lowered part is a blocking `Warning::Gouge` on that move; the path also
carries a note that names the budget and what to do. A source that is
not the part (the Stock, or a pick off the setup's stock) is still
`None`, as for every kind.

The program sim credits the same thing (`stock_sim::credit_marks`,
called beside #303's `credit_breaks`): a column the mark's own cutter
swept is judged against `max(what that cutter swept, model top -
budget)` instead of the model's top. So the two judges agree where both
speak, and the operation's verdict stands alone where the sim refuses
(a turned stock, posed moves, rotary and lathe frames).

**Why a new law:** both kinds ride the cutter's centre ON a line of the
part, so `Section` would convict every legal engraving, and `None` left
them with no judge. Measured on the #1585 fixture (40 x 40 x 8 block,
R15 pocket to Z4, 6 mm end mill), whole-model source, before this
change:

| cut (Top 8) | stock aligned | stock turned |
|---|---|---|
| 0.02 mm deep | refused by the sim ("worst 0.02 mm") | posts |
| 0.3 mm deep | refused by the sim | posts |
| 7 mm deep (the issue's repro) | refused by the sim | **posts** |

Both columns were wrong: no engraving on a model could post on an
aligned stock, and any depth posted on a turned one. After:

| cut (Top 8 unless said) | op gouges | aligned | turned |
|---|---|---|---|
| 0.02, 0.3, 1, 3 mm deep | 0 | posts | posts |
| 3.1 mm deep | yes | refused | refused, by the op's moves |
| 4 mm, 4.1 mm, 7 mm deep | 258-516 | refused | refused, by the op's moves |
| Top typed 7 (surface at 8), 0.3 stated | yes | refused (charged 1.0) | refused |
| pocket floor picked, 0.3 deep along the wall's foot | yes | refused | refused |

**What the budget is, and why two terms:**
- `Top - Bottom` is the depth the operation states. Lowering the part by
  it asks one question: would the cutter standing at Top touch the part?
  That convicts what a typed depth can hide - a Top typed below the
  surface the line lies on, and a wall standing beside the line (a
  picked pocket floor is traced along the wall's foot, and the flank
  takes the cutter radius off that wall for its whole height).
- The cutter radius caps it. The stated depth cannot be the whole
  budget: the seeded Engrave and Trace carry the job's generic Top and
  Bottom, so an unedited one runs the full height of the part (the
  issue's repro), and nothing typed says that is wanted. A kerf deeper
  than half its width is a channel, not a mark, and a channel the model
  does not show is a gouge by the same reading #1593 gave the typed
  slot. The cap is also where a ball's groove stops being its tip form
  and gains straight walls.
- The way out for a deliberate deep groove is to model it: a modelled
  groove is the part's own surface, the budget is measured from its
  floor, and the cut passes.

**`uncovered()` (posting_sim.rs): left as it is.** Both issues asked
whether it should stop counting an op at `None` on a model source as
judged when the sim refuses to judge. Measured against the rule's own
test - a legal program must still post on a turned stock:
- Before this PR a legal whole-model Pocket, Adaptive2D and Offset2D
  were at `None`. The rule would refuse every one of them on a turned
  stock, a 3+2 frame or a rotary: a false refusal, so those ops needed
  a law of their own instead (#1603, below).
- It keeps the verdict tied to whether the sim can model the stock,
  which is what #1595 asks to remove.
- It cannot name the move. The op-level laws do.
- With `Mark` and `Keep`, the only 2D kind left at `None` on a model
  source is the 2D Chamfer, whose depth is derived and whose break the
  sim credits (#303). A rule for one kind that is legal by construction
  would only add refusals.

**Considered and dropped:**
- (b) refuse on a model source when the sim cannot judge, and (c) both:
  above. (b) alone also leaves the aligned false refusal of every
  engraving standing.
- `budget = Top - Bottom` alone (the stated depth, anchored on the
  model). It acquits the issue's repro: Top 8, Bottom 1 states 7 mm and
  cuts 7 mm. "Done means" requires that cut refused, and the seed makes
  it the default program.
- A cap of the cutter diameter. Looser for no stated reason; 5.9 mm
  down the outside wall of this fixture would post. The radius is the
  tighter of the two defensible numbers.
- A fixed cap in millimetres. It does not scale: 1 mm is deep for a
  0.5 mm cutter and shallow for a 12 mm one.
- A new "mark depth" parameter. It would repeat Top - Bottom and change
  the project format for nothing the heights do not already say.
- A column lattice for the operation's law. The standing rule is that a
  lattice verifies and never judges an operation; the lowered-part
  solid test is the same question asked of the mesh. On a part with no
  overhang the two are the same law; under an overhang the solid law is
  the exact one.
- A new `Warning` variant for "deeper than a mark". The variant list is
  cached in project files and read by the post, the MCP findings and
  the UI; `Gouge` is the true verdict (the cutter is past the finish
  surface the op was granted), and the path note carries the reason.
- Crediting the mark in the sim for every Engrave and Trace, whatever
  the source. A stock-source cut carries no law on the operation, so
  the sim keeps judging it against the sharp model (pinned).
- Storing the budget in the cached path, as #303 stores the edge
  breaks. A cached record is file data that #529 then had to bound; the
  budget is computed from the operation's own parameters and tool each
  time the sim runs.

**Known cost (conservative side):**
- A deliberate, unmodelled groove deeper than the cutter radius is
  refused. Model the groove, or use a wider cutter.
- A Trace along the foot of a wall is refused whatever its depth: with
  a flat cutter centred on the wall line it cuts the wall. A Contour is
  the kind for that line.
- On an aligned stock the post's refusal still leads with the program
  sim's clause when both judges convict (the gate's existing order,
  as in #1593); the operation's own findings are on the op, in `verify`
  and over MCP.

**Goldens:** none change.

**Cost to change later:** the budget is one function
(`mark_law::budget`), read by the operation's law and the sim's credit
alike; the law's kinds are one match arm.

## #1603 - The pocket family keeps the part: a whole model, or the parts under a stock source

**Decided:** 2D Pocket, 2D Adaptive and 2D Offset answer for the part
on the operation itself (`crates/app/src/operation/keep_law.rs`):
- **Whole-model source:** a new `DepthLaw::Keep` - the solid law alone
  (`contain_into` with nothing excused), against the source's own mesh.
- **Stock source, or a pick off the setup's stock:** the same solid law
  against the setup's parts (`setup_align::parts_facets_in`, the mesh
  the program sim judges against; fixtures and a stock-only model left
  out), collected into the job as `keep_parts`. A setup with no part
  judges nothing.
A cutting move that enters the part is a blocking `Warning::Gouge` on
that move. Picks on the part stay `Section`, as before.

**Why:** these were `DepthLaw::None` ("the source is the volume to
remove"). That contract is true only of the stock. A whole model is
cleared AROUND (#12: its sections are islands), and with the default
area controls every ring stands outside it - but an area offset or an
extension moves the boundary into the part, and a stock source covers
the part outright (#1554). Nothing on the operation judged either, so
the post rode on the program sim, which refuses a turned stock.
Measured on the #12 fixture (30 x 30 x 10 block, 12 x 12 boss to Z20,
10 mm end mill, 50 x 50 x 20 stock), cut to Z5:

| case | op gouges before | turned stock before | op gouges now | aligned / turned now |
|---|---|---|---|---|
| whole model, Area offset +3 (Pocket, Adaptive) | 0 | posts | yes | refused / refused by the op's moves |
| whole model, extension +5 (Pocket, Adaptive) | 0 | posts | yes | refused / refused by the op's moves |
| stock source (Pocket, Adaptive) | 0 | posts | yes | refused / refused by the op's moves |
| whole model, default controls (Pocket, Adaptive, Offset) | 0 | posts | 0 | posts / posts |
| stock source stopping on the part's top | 0 | posts | 0 | posts / posts |

**Only where the part is kept.** With no stock set up, a whole-model
pocket source is still the volume to remove (#12's reading needs a
stock to clear around), so `Keep` stands down there
(`GenJob::around_part`); the tilt suite's parked pocket is that case.

**Tests that changed, and why:** three existing fixtures ran a
stock-source pocket straight through a part and asserted a silent
verify. Each was a real cut into the part:
- `post_not_judged.rs` and `machine_switch.rs` (the tilted pocket): the
  part filled the whole stock. The part is now the 10 mm the pocket
  leaves, so the cut stops on it.
- `lead_approach.rs` (the user's reported stock-source pocket on
  `Milling-test.step`): it clears the part's own section, which is
  #1554. The two tests now assert that gouge and read the entry laws on
  the remaining warnings.

**Why the solid law and not `Section`:** `Section` adds the plane law,
which reads tangency against the operation's region. For a whole model
the region is the stock around the part, not the part, so that reading
has nothing true to add; the solid law sweeps the real cutter against
the real mesh and is the same test `Section` ends in (b2).

**Correction to #1593's audit** (`docs/decisions/1593-trochoid-law.md`,
merged with #1597): its bullet says a whole-model Pocket / Adaptive2D /
Offset2D "stays outside the part". That holds only with the default
area controls (M5's review of #1597, pullrequestreview-5899991928). With
an offset or extension it cuts the part; this entry is the fix. The
pinning unit test moved with it: the table is now
`the_pocket_family_keeps_a_whole_model_and_sections_a_pick` and
`the_2d_chamfer_alone_stays_unjudged_on_the_part`.

**Considered and dropped:**
- The `uncovered()` rule: above. It refuses the legal rows of the table
  on a turned stock.
- Refusing a stock-source pocket at generate whenever a part stands in
  the stock (#1554's first option). A stock-source clearing that stops
  on the part's top is legal and common; the law convicts only the
  moves that enter the part. #1554's generate-side behaviour (rough the
  stock minus the part) stays its own issue.
- Judging every stock-source 2D kind against the setup's parts here.
  #1593 decided and pinned the opposite for the typed kinds
  (`a_trochoid_on_the_stock_is_never_judged`), and Engrave / Trace
  would need the mark credited against a part they do not name. Probed:
  a stock-source Engrave or Trace 7 mm into the part has 0 op findings
  and posts on a turned stock. Filed as #1607 rather than
  widened here.

**Known cost (conservative side):** the same as #1593's: a ring finished
to size against a coarsely tessellated wall is convicted once the chord
sag passes the gouge tolerance (0.01 mm). The default clearing on the
#12 fixture reads 0.005 mm and is clean; leaving stock clears it.

**Cost to change later:** one match arm (`depth_law_of`) and one
function (`keep_law::parts_in_stock`).
