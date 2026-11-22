## #3452 - Part-off: the whole kerf stands on the chuck side of the part's end face

**Decided.** The Part (part-off) operation's "Bottom Z" (the parting plane)
is the finished part's END FACE. The whole kerf of the blade stands on the
chuck side of that plane (the bar that stays held), one flank on the plane.
The blade's CL (the middle of its tip edge) is half a blade width from the
plane, on the chuck side. The posted corner (#641) is unchanged:
orientation 2 posts the -Z corner, orientation 1 the +Z corner,
orientation 6/9 the centre.

The issue's case (end face Z-70, 3 mm blade, orientation 2, bar to Z-84):

| | CL | kerf | posted Z (orientation 2) | part length |
|---|---|---|---|---|
| before | Z-70 | Z-71.5..-68.5 | Z-71.5 | 68.5 mm, chamfer lost |
| after | Z-71.5 | Z-73..-70 | Z-73 | 70 mm, chamfer kept |

With orientation 1 the same kerf posts Z-70 (its +Z flank, on the face).

**The scrap side is the chuck side** (`motion::turning::parting_law::chuck_side`).
It is the side where the bar is held. In a normal part-off the piece that
falls off (the free end, away from the chuck) is the part, and the bar stays
in the chuck. No setup or machine field states which spindle holds the bar
(a search for a sub-spindle, a counter-spindle or a chuck side in the crates
found none). So the side is ISO 841's lathe Z: +Z points away from the
workholding, and the chuck is at -Z (lower Z) of the setup frame. A plane
with the whole model on the chuck side (the part would be the empty side) is
refused by name. The kerf position parameter of #3469 is where another side
will come from.

**Where the side matters, a setup that cannot say which end is free is
refused** (second and third re-reads of PR #3472). The setup's
`part_alignment` is baked into the facets that the kernel reads, so the
kernel works in the setup frame. In an OP20 flip setup the chuck at -Z holds
the FINISHED part and the free end is the scrap. The kept side matters only
when the model goes on past the plane on both sides
(`parting_law::plane_inside_model`): at an end of the model the model itself
is the part, and no setup can turn that wrong. So the Part arm refuses only
there, with `setup_align::part_off_problem`:

- What fixes the sign: the part AS MODELLED is the as-modelled setup (no
  alignment), which holds the document +Z up the spindle. The part's own
  turning axis is fitted on the setup-frame facets (`multiaxis::axis::fit`,
  the fit the turning frame uses) and read back into the document frame with
  the inverse of the alignment rotation.
- A part modelled standing (its axis at least 0.5 along the document Z): the
  document-up end of the axis must point up the spindle by 0.5 or more.
  -0.5 or less is "end for end", anything between is "no clear hold".
  The limit is never 0, so float noise at a horizontal axis decides
  nothing.
- A part modelled lying down (along X or Y): no setup is "as modelled", so
  nothing tells which end the user means as free. A sign rule (for example
  "the largest component is positive") is a guess: it fails for a part drawn
  with its stub at +X. So a lying model with the plane inside it is refused.
  At an end of the model it generates, as under TURN today.
- A turn about the spindle axis only keeps the hold as modelled.

The first form of this door (`reverses_axis`, document +Z below 0) was
dropped: it missed the flip of a model drawn lying down, where document Z
maps to horizontal in OP10 and in OP20 alike.

**Saved projects.** The loader restores the stored stamp, and an op whose
stamp equals the live one reads Ok. A part-off cached before this change
holds the old centred kerf. The Part stamp therefore carries the salt
`part-off-kerf-side-v1` (the `rotary-picked-faces-v1` precedent of #2848):
every part-off saved before #3452 loads Stale, and the post refuses it
until it is generated again. The other blade kinds keep their stamps (their
paths did not change). Pinned by `part_off_saved_before_3452.rs` with a
fixture recorded on main.

**Behaviour that worked before and is refused now** (#3469 is the way back):
a part-off with the plane inside the model in an aligned setup that does not
hold the part as modelled, and in a setup with a part modelled lying down.

**Where the rule decides, the path says so.** A plane at an end of the model
needs no rule: the model is the part. A plane with model on both sides (a
model that carries its grip stub) relies on the chuck-side rule, and only
then the path carries `CHUCK_NOTE` in plain words: "the bar is taken as held
on the chuck side (lower Z), so the piece above the plane is cut off as the
part". The generation verdict cannot prove the user's intent in that case;
it proves only that the kerf takes nothing above the plane.

**The refusal "nothing to part"** tests the kerf against the BAR (the
model's span widened by the stock's two ends; `StockEnv` gets `s_bot` for
the near end), not against the model only. The correct kerf beside the
part's end is accepted when the stock runs on. A kerf that meets no bar is
refused with its numbers, for example "nothing to part - the 3.000 mm kerf
beside the parting plane (Z-73.000 to Z-70.000) does not meet the bar
(Z-70.000 to Z0.000); set the setup's stock if the bar runs on past the
part". With no stock set and the plane at the model's end, there is no known
material to part, so this refuses (before, the centred kerf cut 1.5 mm of
the part and passed).

**The generation verdict** (`part_kerf_refusal`, called in the Part arm of
the generator) refuses a part-off path whose cutting move puts the blade's
band (its own width, never wider) into model material on the part side of
the plane (the side away from the chuck) by more than 1 µm. It reads the
side from the chuck alone, not from the kernel's kerf, so a kerf put on the
wrong side by any cause is caught. It also refuses a plane with no model on
the part side. It is a refusal (`GenError`), not a warning: the post never
sees the path. The meridian scan (`blade_warnings_excess`) still reads a
through-axis kerf as scrap; that is correct on the chuck side, and the new
law covers the part side that the scan cannot see. Groove Rough and Groove
Finish do not run the new law: their kerf is inside the groove window by
construction (`blade.rs:170-171`), so no legal groove can be refused by it.

**Second read of PR #3472.** The first head took the side of a plane inside
the model from the stock overhang (the side where the stock ran further past
the model). A model with its grip stub (Z-84..0, plane Z-70) under a stock
with a 0.1 mm front facing allowance then parted Z-70..-67, 3 mm into the
part, and the verdict used the same rule and stayed silent. Fixed: the side
is the chuck's, never inferred; the verdict reads it on its own.

**Dropped.**
- Keep the blade centred on the plane and move the plane by half a blade in
  the generator only: the refusal and the verdict would still judge the
  wrong band.
- Infer the scrap side from the model ends or the stock overhang: a facing
  allowance or a grip stub in the model turns such a guess (any threshold
  fails at some size).
- In a flipped setup, part toward +Z (keep the held piece): the setup does
  not say whether the held piece is the part or a bar that runs on, so the
  side would again be a guess. Refused until #3469 states it.
- Refuse every part-off in an aligned setup that is not a turn about the
  spindle: safe, but it also refuses a plane at an end of the model, where
  the side cannot be wrong.
- A warning in place of a refusal: a gouge of the finished part must block
  the post (Prime Directive).
- A user parameter for the kerf position or the chuck side (a facing
  allowance on the back, the blade centred for a later second-side facing, a
  part held from +Z): a strategy choice is a parameter with a default, but
  it is new behaviour, not this P1 fix. Filed as #3469.

**Cost to change later.** Posted output of every existing part-off moves by
half a blade width toward the chuck (breaking post-output change). #3469's
parameter defaults to this behaviour, so it changes no program.
