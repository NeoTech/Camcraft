## #3888 - Control and wear compensation refuse a Ramp or Helix entry by name

**Decided.** `contour_comp::door` refuses a 2D Contour under control or
wear compensation when its entry, or its level transition, is Ramp or
Helix. The refusal names the mode, the arm and the reason:
"control compensation does not ride a Ramp entry - its descent runs on the
lead-in and the ring before G41/G42 engages, so the register does not
apply to it; pick the Plunge or Arc entry, or Computer compensation". The
rule is the same on the wall side and on the bore side. Computer
compensation is unchanged.

**Why.** G41/G42 engages on the lead-in move that lands on the ring
(`contour.rs`, `contour_comp::push_span`). Every move before it is posted
under G40, at the cutter centre that the generator computed from the
nominal tool radius.

- The wall side (PR #3833) zig-zags along the lead-in and the ring.
- The bore side zig-zags along the same path, and its helix and its dive
  links can land on the ring too.

A register larger than the nominal radius, or a positive wear value, then
cuts the finished wall, and the verifier reads only the nominal centre
moves. The refusal closes the whole class (both sides, both arms, entry
and transition) in one place, before any geometry is computed. The dive at
the lead-in's start, which the door's lead-length law keeps at least one
radius off the ring, stays legal.

**Dropped.**

- Start the ramp after the G41/G42 lead-in: the span law needs one planar
  feed walk between the engage and the cancel moves, so a ramp inside the
  span is a new span shape and a new proof. That is not the smaller change.
- Ramp on the lead-in only, else graze: the lead-in under G40 still ends on
  the ring, so the last leg touches the wall at the nominal radius. It also
  replaces a Ramp the user asked for by a plunge without a refusal.
