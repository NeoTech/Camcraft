## #3811 - The 10 mm flute case pins the cause of its rapid findings, not a count

**What was decided.** `a_ten_mm_flute_names_its_cause_before_the_rapid_findings`
no longer asserts three rapid findings. It asserts that there is at least one
`RapidInPart` finding, that each one is a straight lift or drop along the tool
axis, and that its low end puts the shank under the part top. It also asserts
that there is no `Gouge` and no `RapidInStock` finding. The debug test
`a_twenty_five_mm_flute_gets_no_flute_finding` now also asserts that the same
job with a 25 mm flute has no shank, rapid or gouge finding.

**Why.** A release bisect over `v0.26.0..main` found the first bad merge:
`6c164521f` (#3790, issue #3744, one ring set for Pass starts). Its parent
`df899f029` gives 3 findings. `6c164521f` gives 12. `1a4c69d06` (before
#3777) gives the same 12 moves, so #3777 only renamed the finding. #3818
(`7f4915a6d`) changed the ring links again, and the count became 9. The 9
findings on that head are all vertical rapids at two plan points near
X +-46.5, Y 123.1..123.5: retracts up from Z 1.5 or Z 2.0, and drops down to
Z 4.0 before a plunge. With the tip at Z 4.0 or lower, the 10 mm flute ends
at Z 14.0 or lower, and the shank (5 mm radius over a 4 mm cutter) stands in
the part wall under the part top at Z 20. These are real contacts of the
shank, and they block the post. The same path with a 25 mm flute has no
finding of any contact kind, and `a_same_diameter_shank_past_the_flute_is_an_advisory_note`
shows that a shank of the cutter diameter meets nothing. So the toolpath is
clean for a tool that reaches, and the count only shows how many rapids the
ring plan puts at those two points.

**Considered and dropped.**
- Set the count to 9: the next ring plan change breaks it again, and a count
  does not say why a finding is correct.
- Change the ring plan to get back to 3: the plan is not wrong, and the
  findings are not silent. The post stays held.
- Raise #3811 to P1: no collision passes. Every contact is reported and holds
  the post.

**Cost to change later.** Test-only. A later ring plan change that adds a
lateral rapid through the part, or a gouge, makes the test red with the move.
