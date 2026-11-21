## #3146 - Rest pass keeps corner reliefs whose start is on the rest center boundary

**What was decided.** `rest_pocket_with_relief` keeps a planned corner relief
when its start point is inside the rest center area grown by
`RELIEF_START_TOL` (0.01 mm). Before, it used a bare `Region::contains` on
the rest center area. Each relief that the rest pass does not keep now gives
a toolpath note: `corner relief at (x, y) is not cut: the rest area does not
reach that corner`.

**Why.** The relief start is a vertex of the pocket's own center area. Where
the rest area touches a corner, that vertex is on the boundary of the rest
center area. A point-in-region test on a boundary point gives `true` or
`false` by rounding. On a sharp 30 x 30 mm pocket, it kept 1 of 4 dogbones.
The tolerance is the same two arc tolerances as `REST_MARGIN`. The start
stays a legal tool position: it is always on the plain pocket's own center
area, so the tolerance cannot move the tool past a wall.

**Considered and dropped.**
- Test the distance from the relief corner to the rest area. This asks the
  right question in other words, but needs a new distance helper on
  `Region`. The grown-region test gives the same answer here with the
  existing API.
- Keep every relief of the full plan in the rest pass. A sharp inside corner
  always has rest material after a larger round tool, so this is nearly
  always right. But a relief forced by an override on a corner with no rest
  material would then add a spur to a pass that does not go near it. The
  note is the visible answer for that case.
- A refusal for a dropped relief. The rest pass is still correct without
  the relief; the plain pocket is the workaround. A note keeps the program.

**Cost to change later.** Low. The filter and the note are in one block of
`rest2d.rs`, pinned by `crates/motion/tests/rest_corner_relief.rs`.
