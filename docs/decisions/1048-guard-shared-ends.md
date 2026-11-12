## #1048 - the plan's crossing check reads "on the line" to within 1 pm, so a shared end point a rounding apart is no crossing

**What was decided.** The adaptive planner's own crossing check
(`crates/motion/src/machining2d/adaptive/guard.rs`, the `scan` behind
`first_crossing` and `crossing_pairs`) asks a new pair predicate,
`crosses`, instead of the exact-sign `proper`. It is the same
proper-crossing test, except that a chord end within `TOUCH` = 1e-9 mm
(1 pm) of the other chord's line counts as on it (sign 0), where only an
exact zero did before. Two chords cross only when the ends of each stand
strictly on opposite sides of the other's line.

- A shared end point a rounding apart (an orbit closing on its start, a
  microlift return landing on a chord end, a ring starting on the last
  ring's vertex) now reads exactly as an exact shared vertex always did:
  a contact, no crossing.
- A real crossing still crosses: the figure-eight, a chord crossing the
  lap 0.15 mm past a shared vertex, a vertex one micron through a chord.
- A non-finite point (NaN, +-inf) in any walk is reported as a crossing
  at that point (its walk paired with itself). Before, the length filter
  silently dropped NaN chords, and an infinite y slipped past the bounds
  check (it only looked at x); both now read as crossings.
- `guard::proper` itself is unchanged: `lift_link` uses it for its own
  link law and is outside this issue (and touched by open PR #1080).

**Why 1 pm.** The tolerance is there to read one vertex reached twice as
one vertex. f64 rounding on a 10 m workspace is ~2e-12 mm per coordinate
and a few ulps per computed point, so 1e-9 mm sits about a thousand times
above it. Cutting tolerances are microns (the verifier's is 0.005 mm), a
million times above it. A chord end within 1 pm of a line can only meet
the other chord at that end, so the most a "hidden" crossing could be is
a 1 pm overlap at a touch point, which no tool can see. The test compares
`|orient| <= TOUCH * |ab|`, with no division; the chord-length filter
(1e-12 mm) is unchanged.

**Why contacts are not crossings (changed from the brief).** The brief
asked that a vertex on another chord's interior and a collinear overlap
stay crossings, and the first version of this PR did that. It failed a
pinned plan: in `adaptive::tests::a_channel_is_grown_as_one_spiral_that_never_crosses`
the lifted plan's pass 197 retraces pass 196's last chords exactly (32
chord pairs of the same line, same direction: a lifted piece starting
over the stretch just cut). The exact-sign law always read that as a
contact (orient 0), and the verifier's law (a) does the same, so a
stricter planner check would send a good adaptive plan to Offset rings,
the very loss #1048 is about. The faithful fix is the old law with its
exact zero widened to `TOUCH`: every contact the planner used to accept
exactly it now also accepts a rounding off, and nothing else changes.
Pinned by `a_contact_is_no_crossing_and_a_micron_past_is`.

**Considered and dropped.**
- *Shared ends excused, all other contacts crossings* (the brief's
  shape). Breaks the retraced lifted pieces above. Dropped.
- *A relative tolerance (scaled to the coordinates).* The workspace is in
  mm and bounded; an absolute 1 pm is simpler to state and holds below
  ~1e5 mm. Dropped.
- *Snapping walk vertices together before the check.* It would change the
  posted path, not only the check. Dropped.

**Left open (follow-up issue).** The verifier's law (a)
(`crates/sim/src/verify/laws.rs`, `proper_cross`) is still exact-sign. A
plan with an end shared a rounding apart, which the planner now accepts,
could still read as crossing there, and that holds the post instead of
falling back to Offset rings. Still safe (refused by name), but the two
laws should agree. That is a verifier-law change in `camcraft-sim`, so it
is filed separately.

**Cost to change later.** One constant and one predicate in `guard.rs`;
the pins in `guard_tests.rs` name each case.
