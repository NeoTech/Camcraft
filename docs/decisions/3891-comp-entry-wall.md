## #3891 - Control and wear compensation refuse a Linear lead, and pin the Perpendicular entry clearance

**Decided.** `contour_comp::door` refuses a 2D Contour under control or
wear compensation when its lead-in or its lead-out is Linear. The refusal
names the mode, the end and the reason, for example: "control
compensation does not ride a Linear lead-in - the entry's descent at its
start runs on the ring's own offset before G41/G42 engages, so a
register over the nominal radius, or a positive wear value, cuts the
finished wall there; pick the Perpendicular lead-in, or Computer
compensation". The Perpendicular lead stays legal, and its length law
(already on the parameter, re-checked on the actual moves by
`push_span`) holds the pre-engage moves one full radius beyond the
ring's line.

**Why.** The pre-engage moves of the Plunge and the Arc entry post under
G40 at the nominal centre:

- The Plunge dives at the lead-in's start, and the Arc entry's chord
  runs from further out onto that same point.
- A Linear lead's start sits ON the ring's offset line, one nominal
  radius off the finished wall - the same line the Ramp and Helix
  descents were refused for (#3888). A register over the nominal
  radius, or a positive wear value over the stock, cuts the wall at the
  dive point before G41/G42 engages. The mirrored lead-out has the same
  defect at its end: the cancel withdrawal lands on the ring line, past
  G40.
- A Perpendicular lead's start stands its length further into the air.
  The door's length law (and `push_span`'s re-check of the actual move)
  holds that at one tool radius, so every pre-engage position keeps at
  least (radius + lead length) off the finished wall. That is the
  maximum register difference the door vouches for: the register must
  sit one full radius over the nominal before the wall is at risk, and
  such a register is a different tool - the room law (corners, necks,
  arcs), judged at the nominal radius, has already failed it.

The #3888 decision's claim that "the door's lead-length law keeps [the
dive] at least one radius off the ring" was true only for the
Perpendicular shape; this closes that gap.

**Pins.**

- `crates/motion/tests/machining2d.rs`
  `a_linear_lead_under_control_or_wear_is_refused_by_name_on_both_sides`:
  both ends, both modes, wall side and bore side; computer unchanged.
- `crates/motion/tests/machining2d.rs`
  `the_perpendicular_comp_entry_keeps_the_lead_length_beyond_the_ring`:
  Control and Wear, both sides, Plunge and Arc - every cut-adjacent
  move before each span's engage stands at least the radius (5) plus
  the lead (6) off the part, and the closest position is the dive point
  itself. The bore side is pinned here (a whole-model source posts only
  the wall side).
- `crates/app/tests/control_comp.rs`
  `a_linear_lead_in_or_out_under_control_or_wear_is_refused_and_posts_nothing`:
  through `GenerateOperation`, nothing generated, nothing posted.
- `crates/app/tests/control_comp.rs`
  `the_entry_descents_before_g41_hold_the_wall_clear_under_an_oversized_register_and_positive_wear`:
  from the posted Fanuc program - an oversized register (5.0 mm on the
  Ø6 tool, 2 mm over its nominal 3) and a positive wear value (1.0 mm,
  the effective radius 4.0) do not touch the finished wall at any stand
  under G40; the closest stand is the dive point at 9 mm, the radius
  plus the lead.

**Dropped.**

- Engage G41/G42 on the descent itself: RS-274 / ISO 6983 builds the
  offset over a straight XY block; an axial plunge has no direction to
  offset against and a vertical-plane arc is not the straight block the
  startup law needs. Not legal to post.
- Move the Linear dive point out along the air normal under
  compensation: a new lead shape with a new room-law proof, the same
  class of change #3888 dropped ("not the smaller change").
- Keep the Linear lead and bound the register: the register is the
  operator's machine-side value; the generator cannot bound it, so the
  Perpendicular length law is the only clearance the door can vouch
  for.
