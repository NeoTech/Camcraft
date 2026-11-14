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

**`uncovered()` (posting_sim.rs): left as it is.** The issue asked
whether it should refuse an op at `None` on a model source when the sim
refuses to judge, perhaps scoped to these two kinds. With `Mark`, no
Engrave or Trace on a model source is at `None`, so a scoped refusal
would have nothing to refuse. It would also be option (b), which keeps
the verdict tied to whether the sim can model the stock - the thing the
issue asks to remove - and would refuse every legal engraving on a
turned stock, a 3+2 frame or a rotary. The kinds still at `None` on a
model source (whole-model Pocket / Adaptive2D / Offset2D, 2D Chamfer)
are #1593's audit, unchanged.

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
