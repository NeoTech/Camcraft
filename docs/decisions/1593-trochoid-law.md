## #1593 - A Trochoidal slot is judged against the finished part, like the Slot

**Decided:** `depth_law_of` routes `OpParams::TrochoidalSlot` through
`authored_law(src)`, the law the Slot and the Circular share (#1537,
#1543, #1550): an `Entity` or `Selection` source naming the finished part
is `DepthLaw::Section`, a Stock source is `None`. The arm sits after the
`part_solid` early return, so a Stock source, or a pick off the setup's
stock, still cuts unjudged. #1513's `slot_fit::refuse_unfit_slot` is kept
as it was: a picked slot that does not fit its selection in XY is still
refused at generate; the new law adds the depth (and, on a whole model,
the walls) the fit check never read.

**Why:** the Trochoidal slot's endpoints and width are typed by hand,
exactly like the Slot's. Falling to `_ => DepthLaw::None`, nothing on the
op judged them, and `uncovered()` (posting_sim.rs) counts any fresh,
stamped op as judged. Measured on the #1585 review fixture (40 x 40 x 8
block, R15 pocket to Z4, 6 mm end mill), before this change:
- a whole-model slot 3 mm under the floor generated clean and, with the
  stock turned (the program sim refuses to judge), posted;
- a whole-model slot through the pocket wall (x 4..20, 10 wide, at the
  floor) generated clean and posted with the stock aligned as well as
  turned - the program sim missed it too.
Both now carry blocking `Warning::Gouge`s on the op, and the post is
refused by name on the op's moves ("T1: move N cuts past the finish
surface"). A picked slot that fits its floor in XY but is typed 3 mm
under it is refused the same way.

**Audit - the 2D kinds still at `None` on a part-solid source:**
- Pocket / Adaptive2D / Offset2D with a whole-model source: by contract
  the source is the volume to remove. Probed at bottom Z1 on the same
  fixture: every move below the floor stays outside the part (radius
  23+ against the 20 mm half-block), no gouge, the program sim agrees.
  Left as is; picks were already `Section`.
- Chamfer2D: enters the part by design (the broken edge); its depth is
  derived from width and angle, not typed. Left as is.
- Engrave and Trace: enter the part by design, and stay `None`. But the
  probe shows the same hole this issue closed: on a whole-model source
  cut to Z1 they follow every edge 3-7 mm under the model, the aligned
  program sim refuses it ("worst 7.00 mm"), and with the stock turned it
  POSTS. Their legal depth is a design question (an engraving is meant to
  cut the part), not the typed-geometry law, so it is filed as its own
  P1 (#1595) rather than folded in here.
- Drill with cross holes: the lathe turret's cross drill, judged by its
  own frame laws; not a 2D mill kind.
The unit test `the_kinds_that_enter_or_remove_the_part_by_contract_stay_unjudged`
pins that table, so moving one of those kinds is a visible decision.

**Considered and dropped:**
- Making `uncovered()` refuse every op whose law is `None` on a model
  source (the issue's "also consider"). It would refuse every whole-model
  pocket, engraving and chamfer outright, including the legal ones above;
  the Engrave/Trace follow-up (#1595) is where a refusal or a law for those kinds
  belongs.
- `DepthLaw::Contain`: same reasoning as #1543 - no rim is declared for
  a slot, and `Section` is what the Slot takes.

**Goldens:** none change - no golden runs a Trochoidal slot on a
part-solid source (golden_cl and solid_law_goldens pass unchanged).

**Cost to change later:** one match arm.
