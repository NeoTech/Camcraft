## #2176 - The simulation draws a drawn holder as drawn

**Decided.** The viewport simulation draws a drawn (spun) holder body
with its own outline (`SpunProfile::outline`): its radii, lengths, tapers
and flattened arcs. The collision checks, the verify step and the post
gate do not change. They still read the bands of the same landed body
(`tool_body::judged` + `tool_body::stack`), and those bands are never
smaller than the outline. Before the simulation draws the outline,
`JudgedOutline::with_drawn` checks that each drawn point is inside a band
that the laws read at its height. If a point is outside, the body is
refused by name (`OutlineError::PastJudged`) and not drawn. With a
drawn holder, the simulation also draws the shank as stored. Since #2207,
`HolderProfile::enclose` uses strict interior bounds. A holder band that
only touches the shank top does not widen the judged shank. The stored
and judged cutter outlines are equal in the cone and picture fixtures.
The stored-cutter substitution stays as a defensive guard: `with_drawn`
refuses a stored cutter wider or longer than the judged one. A
machine-default holder (#828) and an imported (STL/STEP) body keep the
band drawing, because the laws judge them as floored or enveloped bands.

**Why.** The maintainer saw the simulation draw stacked cylinders for a
holder that he had drawn with a taper (#2176). The old rule of #743 was
"what is drawn is what is checked". For a drawn holder that rule showed a
shape that the user did not draw. The trace (issue comment) found that
all consumers already read one landed body, so no check reads a wrong
shape. Only the picture was wrong.

**Dropped.**
- Remove the stored-cutter substitution after #2207. It has no effect in
  the current cone and picture fixtures, but its size check remains a
  useful guard for the display outline.
- Keep the stepped bands in the simulation (the #743 rule). This shows a
  shape that the user did not draw, which is the reported bug.
- Make the collision laws read the exact cone and arc segments. That is
  a new geometry law under the safety floor (`effort:max`). The bands are
  conservative, so the checks stay safe without it.

**Cost to change.** Small. `JudgedOutline::drawn` is display data only.
To go back, `for_operation` stops the call to `with_drawn`. The picture
can now be slightly smaller than the check: a check can refuse a tool
position where the drawn taper does not visibly touch the part. That is
the safe direction.
