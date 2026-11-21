## #3132 - A None lead-in stands the wall arc entry down

**What was decided.** On an outside contour, the Arc entry does not run its
vertical wall arc when the lead-in is None. The level descends by the
grazing dive at the ring's start, the same dive that a Plunge entry, a
tabbed level or a blocked arc uses. A note says so once:
"entry: the arc stands down with no lead-in - ...".

**Why.** With no lead, the wall arc lands on the seam itself. Its start is
about one arc length back along the seam's tangent (8.7 mm on the user's
part). From above, in the viewport and in the posted chords, that is a
straight move into the seam: the same shape as the seeded 8 mm linear lead.
This is the only setting on `Milling-test.step` that reproduced the report:
Plunge, Ramp and Helix entries with both leads None already start the cut
on the ring. The maintainer's law on the issue is that a None lead adds no
XY lead move at all.

**Considered and dropped.**
- Keep the arc and only add a note. The user would still see the lead
  that they refused.
- Turn the arc into the plane normal to the wall. It still adds XY travel
  at the seam.
- Stand the bore-side (Side::Inside) Arc entry down too. In a bore, the
  ramp, helix and arc are certified descents in bounded air. They are
  entries, not leads, and the issue keeps entry moves. Not changed.
- Stand the arc down when a requested lead was refused for room. That is
  not the user's None, and it is the pre-existing shape. Not changed.

**Cost to change later.** One condition in `contour.rs` (`no_lead_in`).
A non-centre-cutting tool with Arc entry and no lead now meets the
graze's centre-cutting refusal. That refusal's hint ("pick the Arc
entry") does not help when the lead-in is None; a wording follow-up can
name the lead.
