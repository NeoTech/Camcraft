## #1164 - Wire EDM: taper by angle word (control-computed lean)

#536 posted a constant taper as U/V moves and refused angle-word controls by not offering them: with an angle word the control decides the lean at corners and lead-ins, which the wire laws could not see. This adds the dialect with that decision stated as machine data.

**Decided:**

- **The dialect is machine data: `edm.taper_angle`, an object of two enumerated words with no default.** `positive` (`top_left` / `top_right`): the side of the direction of travel a positive angle word leans the wire's top toward. `corner` (`mitre` / `cone`): what the control cuts at a corner. Stating the object is stating the dialect: the post writes the angle word and no U/V. A garbled word refuses the file; an unstated one is refused by name at generation ("states no edm.taper_angle.corner (mitre or cone)"), never guessed. The angle code is `edm.codes.taper_angle`, a template with `{angle}`, beside the existing `taper_on` / `taper_off`; each missing code refuses by name. A machine stating neither `uv` nor `edm.taper_angle` is refused by name, and the refusal names both.
- **The generator and the verify model the stated rule; the post writes only what the control reads.** `mitre`: the sides keep their planes and meet along the mitre - #536's path exactly (its CL golden byte for byte, pinned). `cone`: the wire pivots about each top-plane corner with its lean held at the angle, the lower end on an arc of radius `H tan(angle)`; the CL lays it as posed turning moves at most 2 degrees apart (each chord within `|d| (1 - cos 1 deg)`, 1.5e-4 of the lean offset - 0.1 um on the fixture) so the #534/#536 wire laws read it at every height like any lean. The post drops the turning moves (the control makes them) and writes each move's X Y on the programmed plane.
- **The programmed plane is the work's top**, where the profile holds (#536) and the control pivots; the section's note says its Z (the control is set to it). The guides' U/V stroke check still runs on every pose, as the control still drives U/V.
- **Lead-in and lead-out:** the post writes `taper_on` and the angle word before the lead-in and `taper_off` before the lead-out, and the path models the control ramping the lean linearly over each of those moves - #536's lead-in twist and upright lead-out. Under `cone`, a lead-in that meets a corner is refused by name (the corner's turn would be cut by no move); move the start hole.
- **The word's sign is computed from the path:** the side the top leans relative to the travel on every side move, the same everywhere (else refused), signed by `positive`.
- **4-axis (#537) on an angle-word machine is refused by name** at generation and at the post: one angle cannot state different upper and lower profiles.
- **U/V machines are unchanged:** the stamp, CL and NC of every U/V cut are byte-identical (the #536 and #537 goldens pass unedited).

**Considered and dropped:**

- **An `arc` corner rule with a stated radius** (the control rounds the lean change over a radius). A third value is cheap to add later; two rules cover the pinned cases and each value added needs its own model and pin.
- **A default corner rule.** Guessing what the control cuts is exactly the silent motion #536 refused.
- **Refusing sharp corners under any angle-word taper.** Safe but needless once the rule is stated and modelled.
- **XY on the lower guide plane, as the U/V post.** The control's corner rule acts on its programmed contour; the profile and the pivot are on the top plane.

**Not exercised on a real EDM control (D10).** The fixture's codes and words are invented; a user fills `edm.taper_angle` and `edm.codes.taper_angle` from their manual and dry-runs before any cut.

**Sources:** plane trigonometry (a cone's section; the textbook mitre), #290 D2/D5/D7/D10, #536's decision record. No vendor post, control manual or CAM product behaviour was consulted.

**Cost to change later:** low - another corner value is one arm in `motion::wire_edm::taper` and one word in `post::machine::edm_taper`.
