## #2511 - Uncut-corner check: a note when the check did not run

**What was decided.** When `unreached_corner_note` returns an error,
`keep_law::narrow_noted` now adds a note to the toolpath:
`<kind>: the check for corners of the stock tighter than the tool did not
run (<error>) - look at the stock corners around the part`. The log line
stays. The path is kept and the cut is not blocked, as for the band check
directly above it.

**Why.** The band check already says in a note that it did not run. The
corner check wrote only a log line. Material could stay in a corner and
the user saw nothing (review of PR #2503).

**The pin.** `operation::keep_law::tests::a_corner_check_that_cannot_run_says_so_in_a_note`
is a unit test beside `narrow_noted`, because that function is
`pub(super)`. It uses a tool diameter of 4e12 mm. The erosion then
returns `GeoError::OutOfRange`, so the two checks fail and each must give
its note. No input through the commands gets to this error today; the
test pins the guard for the day one does.

**Considered and dropped.**
- One shared note for the two checks: dropped. Each check names different
  material (bands, corners), and one can fail without the other
  (the corner check has more steps: the dilation and the stock clip).
- A blocking warning in place of a note: dropped. The check is advisory
  and the band check does not block either.
- An end-to-end test in `crates/app/tests/narrow_band.rs`: dropped. No
  legal document makes the corner check fail.

**Recorded, not changed: the 0.03 mm noise threshold.**
`CORNER_HALF_WIDTH` (0.03 mm, `crates/motion/src/machining2d/unreached.rs`)
can hide a real cusp at an obtuse stock corner (review of PR #2503). At a
150 degree corner with a 10 mm tool the cusp is about 0.18 mm deep
(`r * (1 / sin(75 deg) - 1)` = 0.176 mm). By calculation its largest
inscribed circle has a radius of about 0.087 mm, which is above the
threshold; a corner more obtuse than about 162 degrees has a cusp below
it and gets no note. This calculation is not a measurement. The decision
of #2503 accepted this threshold. This PR does not change it. The number
must be reviewed against a measured case before it changes; that review
is open work and needs its own issue.

**Cost to change later.** One format string and one test assertion.
