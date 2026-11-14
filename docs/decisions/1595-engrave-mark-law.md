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
`None`, as for every kind. (Amended by the review of PR #1615, below:
the mark is also bounded by the part under the cutter, and the shank,
holder and rapids are judged against the part as it stands.)

The program sim credits the same thing (`stock_sim::credit_marks`,
called beside #303's `credit_breaks`): a column the mark's own cutter
swept is judged against `max(what that cutter swept, model top -
budget, underside + 0.2 mm)` instead of the model's top. So the two judges agree where both
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

## #1595 (review of PR #1615) - A mark is bounded by the part under the cutter, and only the edge gets the budget

Two findings of the review (NeoTech's runner review, items 1 and 2;
M5's correction pullrequestreview-5900983707). Both amend the mark law
above; the rest of it stands.

### 1. The floor a mark must leave

**The defect:** on a 40 x 40 x 2 plate with a 6 mm end mill, Top 2 and
Bottom -0.9 state a 2.9 mm "mark". Lowered by 2.9 the plate lies at
Z -2.9..-0.9, clear of a cutter at Z -0.9, so the operation's law found
nothing; the sim's credit floor `max(sweep, top - budget)` was -0.9,
under the plate, so the program sim forgave the same columns. Engrave
and Trace cut THROUGH the finished plate and posted on both stocks.
The same held for any web shorter than the budget (a deck over a
void).

**Decided:** a mark is bounded by the material actually under the
cutter, in both judges:

    depth of a mark <= min(budget, thickness under the cutter - MARK_FLOOR)

`MARK_FLOOR = 0.2 mm` (`camcraft_sim::verify::MARK_FLOOR`, one constant
read by both).
- **The operation's law** (`mark_law::mark_into`): beside the
  lowered-part reading, the cutting edge is swept with the same solid
  law against the part's **undersides raised into a solid 0.2 mm high**
  (`verify::seat_skin`). An edge that enters it stands less than the
  floor above the part's bottom, or above the roof of a void in the
  part. That is a blocking `Warning::Gouge` on the move, and the path
  carries a second note that says so by name: "<kind> cuts through the
  part: under the cutter less than 0.200 mm of the part is left on its
  underside ...".
- **The sim's credit** (`stock_sim::credit_marks`): the floor a swept
  column is judged against is now `max(sweep, top - budget, underside
  + MARK_FLOOR)`, the underside being the highest one in the column.
  A column with a top and no underside under it (not a closed part
  there) is credited nothing.
- **What an underside is** (`verify::undersides`): a facet with the
  solid just above it and air just below, by the vertical-ray parity
  the solid law already uses. The winding is not trusted (an STL may
  be wound either way), and a face between two bodies of one part
  (material on both sides) is not an underside. A facet whose column
  cannot be read is kept as one: a floor too many, never one too few.

Readings on the thin plate (Engrave and Trace alike, whole model):

| cut on the 2 mm plate | op | aligned | turned |
|---|---|---|---|
| Top 2, Bottom -0.9 (the review's repro) | Gouge, "cuts through the part" | refused | refused, by the op's moves |
| 1.9 mm deep (0.1 mm left) | Gouge, "cuts through the part" | refused (sim charges 0.10) | refused |
| 1.8 mm deep (exactly the floor left) | clean | posts | posts |
| 0.3 mm deep | clean | posts | posts |

**Why 0.2 mm, and why a fixed number:** the floor answers one
question - is the part still whole under the mark? - and that is a
property of the machine and the sheet, not of the cutter or the depth.
Z setting, spindle runout and the flatness of a clamped sheet each run
to several hundredths of a millimetre, so a web under about 0.2 mm is
torn or cut through on a real machine even when the numbers say it
stands. It also keeps the floor well above the tolerances the two
judges read with (0.01 mm gouge tolerance on the operation, the sim's
lattice tolerance), so neither can round a through-cut into a pass.
It is deliberately small: thin tags are engraved (a 0.5 mm tag still
takes a 0.3 mm mark), and the floor exists to refuse breakthroughs,
not to judge how stiff the remaining web is.

**Considered and dropped:**
- Capping the budget by the thickness with no floor (the review's
  first suggestion). A mark that leaves 0.01 mm is a through-cut on
  any machine; the two judges would also be reading a zero-thickness
  web at their own tolerances.
- A proportional floor ("a mark takes at most half the part under
  it"). Scale-free, but it refuses deliberate deep scoring of sheet
  (fold grooves leave a thin web by design) and it would have changed
  what M5 verified on the user's part. The maintainer may prefer it;
  it is one constant and one line in each judge.
- "Refuse when the lowered part leaves nothing under the cutter" (the
  review's second suggestion). It convicts the 2.9 mm case but not a
  1.95 mm cut on the 2 mm plate, which is the same breakthrough.
- One global cap from the part's least thickness. Any chamfered or
  wedge-shaped edge has zero thickness somewhere, so every mark on
  such a part would be refused.
- A column lattice on the operation's side. The standing rule: a
  lattice verifies, it never judges an operation. The raised
  undersides are a mesh, read by the same exact sweep.
- A new `Warning` variant for a breakthrough: as for "deeper than a
  mark" above, `Gouge` is the true verdict and the note names it.

**Known cost (conservative side):**
- Where the part is thinner than 0.2 mm (a knife edge where a sloped
  top meets the underside) no mark can be made at all, and a cutting
  edge passing within 0.2 mm above such an edge is convicted even if
  it clears the part there.
- A mesh made of several closed shells that overlap (not a modelled
  solid) is read facet by facet: a facet that is an underside
  anywhere under its centre or half way to its corners is treated as
  one over its whole area.
- A part with an open mesh (no underside under a top) gets no mark
  credit from the sim in those columns.

### 2. Shank, holder and rapids: the part as it stands

**The defect:** `mark_into` lowered every facet and then asked the
whole solid law, so the shank and holder checks (and the rapids) ran
against the lowered part too. A shank standing up to the budget
(as much as the cutter radius) inside the part was excused.

**Decided:** only the cutting edge (the fluted length) gets the mark
budget. On a cutting move the shank and holder bands, and on a rapid
the whole assembly, are swept against the part as it stands
(`verify::stack_hits`, a second reading of the same solid law that
leaves a cutting move's edge to the mark law): `ShankHit` on the part,
`RapidInStock`. From the lowered-part reading only the edge's `Gouge`
is kept.

**One excuse, stated and pinned:** on a straight rapid UP along the
tool axis that starts exactly where the cutting move before it ended,
the cutting edge is excused in the pose it already stands in - and
nothing else is. That pose is the kerf the cutting move just made (the
mesh does not show the kerf), judged on that move by the mark law, and
every Engrave and Trace ends each chain that way. A rapid sideways out
of the kerf, a rapid down into the part, and a lift from anywhere no
cut left the tool are all convicted (`only_the_retract_up_the_marks_
own_kerf_is_a_rapid_under_the_surface`).

**The lift itself is judged** (second review of PR #1615; the first
form skipped the whole move, on the reading "the stack was judged
there on the cutting move" - true of the start pose only, and a lift
carries the assembly up through space no cutting move swept). On the
excused lift two things are swept against the part as it stands:

- the shank and the holder, over the whole lift;
- the column the edge rises through above its own top: a cylinder of
  the edge's largest radius from the edge's top at the start to its
  top at the end. The edge's profile never narrows upward, so that
  column is exactly what the edge enters beyond its start pose. It
  matters where no band speaks for it: a cutter with no shank or
  holder entered, or a shank thinner than the cutter.

Either one in the part is a `RapidInStock` on the lift, and the post
refuses: a part overhang above the end of a line, which the holder
reaches only on the way up, is no longer silent. Considered and
dropped: judging the body only (the review's minimum - leaves the edge
free to rise into a lip beside a thinner shank); dropping the excuse
and judging the whole assembly (convicts every legal mark, whose kerf
the mesh does not show). Pinned by `crates/sim/tests/mark_retract.rs`,
`the_retract_out_of_the_kerf_is_judged_past_where_the_cut_left_the_
tool` (mark law) and `a_retract_into_an_overhang_of_the_part_is_
convicted_on_the_lift_and_never_posts` (generate and post, both
stocks). Not reachable today through the generator as "posted clean":
Engrave and Trace close every traversal, so the lift stands over the
plunge and the approach rapid down the same column already convicted
such an overhang; the law does not lean on that.

**Stricter than before, on purpose:** a rapid that dips into the part
within the budget (a re-entry rapid into an earlier pass's kerf) was
excused by the lowered part and is now `RapidInStock`. The generators
do not emit one for these kinds (each level re-plunges at feed from
above Top; the suites and the user's `Milling-test.step` are
unchanged), and a rapid has no mark to make.

### 3. Notes for the maintainer (no code change)

- **The budget grows with the cutter.** `budget = min(Top - Bottom,
  radius)`: a 12.7 mm end mill may take a 6.35 mm deep, 12.7 mm wide
  rabbet along every model edge and call it a mark. The stated depth
  already caps it (a 0.3 mm engraving has a 0.3 mm budget whatever the
  cutter); what is open is the unedited seed, whose Top and Bottom
  span the part. Options, each one line in `mark_law::budget`:
  (a) a fixed engraving maximum, e.g. `min(.., 1.0 mm)`. It makes
  "mark" mean one thing for every tool, at the cost that today's
  legal 3.0 mm cut with the 6 mm cutter (M5's reading on the user's
  part) becomes a refusal;
  (b) a cap by tool type (the full radius only for a ball or a
  V-point, whose groove is its tip form; a small fixed depth for a
  flat end mill, which cuts a rabbet);
  (c) leave it (today). Not changed here: (a) and (b) both turn
  programs that post today into refusals, and that is the
  maintainer's call. Recommended: (b), in its own issue.
- **No stock, no keep law.** With no stock set up, `DepthLaw::Keep`
  stands down (`GenJob::around_part`): a whole-model Pocket with an
  area offset of +3 has no finding and posts. That is #12's documented
  contract (with no stock the model itself is the volume to remove),
  not a regression of this PR, but it means a project whose stock is
  simply not set yet is unjudged. Recommended: keep the contract, and
  consider a path note "no stock is set up: the model is cleared as
  the volume to remove" so the reading is visible.
- **A Trace on part B that enters a neighbouring part A** has no
  finding on the operation: the mark and keep laws read the source
  entity's mesh only, so on a turned stock nothing judges A (the
  aligned sim does). It predates this PR and holds for `Section` kinds
  on B as well. Not fixed here: the fix is to judge the edge against
  every part of the setup (`setup_align::parts_facets_in`, the mesh
  #1607 already carries into the job as `keep_parts` for stock
  sources), and #1607 is rewriting exactly those lines on top of this
  branch. Filed as #1621 (P1: a gouge that posts) to land with or
  after #1607.

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
