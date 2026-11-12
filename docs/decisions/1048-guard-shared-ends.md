## #1048 - the plan's crossing check reads a chord end within 1 pm of the other chord's END point as a touch, so a shared vertex a rounding apart is no crossing

**What was decided.** The adaptive planner's own crossing check
(`crates/motion/src/machining2d/adaptive/guard.rs`, the `scan` behind
`first_crossing` and `crossing_pairs`) asks a new pair predicate,
`crosses`, instead of the exact-sign `proper`. It is the same
proper-crossing test, except that a chord end within `TOUCH` = 1e-9 mm
(1 pm, a point-to-point distance) of one of the other chord's END points
counts as on that chord's line (sign 0). Away from end points the exact
sign stays the law. This is the same constant and the same rule as the
verifier's law (a) (`camcraft_sim::verify::CROSSING_TOUCH` and
`proper_cross_touch`, #1129 / PR #1308), so a plan the planner accepts is
not held by the verifier for a rounding. `TOUCH` is a local constant with
a comment pointing at the sim one: motion takes no dependency on sim.

- A shared end point a rounding apart (an orbit closing on its start, a
  microlift return landing on a chord end, a ring starting on the last
  ring's vertex) now reads exactly as an exact shared vertex always did:
  a contact, no crossing.
- A real crossing still crosses: the figure-eight, a chord crossing the
  lap 0.15 mm past a shared vertex, a vertex one micron through a chord,
  and a walk passing through a chord at a vertex a picometre off its
  interior (the #1134 review's probe).
- A non-finite point (NaN, +-inf) in any walk is reported as a crossing
  at that point (its walk paired with itself). Before, the length filter
  silently dropped NaN chords, and an infinite y slipped past the bounds
  check (it only looked at x); both now read as crossings.
- `guard::proper` itself is unchanged: `lift_link` uses it for its own
  link law and is outside this issue (and touched by open PR #1080).

**Changed after review (NeoTech, head d1dc959b).** The first version
excused a chord end within `TOUCH` of the other chord's LINE
(`|orient| <= TOUCH * |ab|`). That holds per chord pair (a missed pair
penetrates at most 1 pm), but not per walk: when the vertex shared by a
walk's chords `c-d` and `d-e` lands within 1 pm of chord `ab`'s
interior, `c-d` reads as ending on `ab` and `d-e` as starting on it, so
neither pair is reported though the walk passes through `ab` with any
depth. The earlier claim here ("a chord end within 1 pm of a line can
only meet the other chord at that end, so the most a hidden crossing
could be is a 1 pm overlap") was wrong for that reason, and is
withdrawn. Excusing only end points closes the hole: an interior
T-contact keeps the exact sign, so the pass-through reports at +-0.5 pm,
at small and metre coordinates. Pinned by
`a_walk_through_a_chord_at_a_vertex_a_picometre_off_its_line_still_crosses`.

**Why 1 pm.** The tolerance is there to read one vertex reached twice as
one vertex. f64 rounding on a 10 m workspace is ~2e-12 mm per coordinate
and a few ulps per computed point, so 1e-9 mm sits about a thousand times
above it. Cutting tolerances are microns (the verifier's is 0.005 mm), a
million times above it. Being a point-to-point distance, it judges a
1 um chord and a 1 m chord the same.

**Why exact contacts stay no crossing.** The brief asked that a vertex on
another chord's interior and a collinear overlap be crossings, and the
first version of this PR did that. It failed a pinned plan: in
`adaptive::tests::a_channel_is_grown_as_one_spiral_that_never_crosses`
the lifted plan's pass 197 retraces pass 196's last chords exactly (32
chord pairs of the same line, same direction). The exact-sign law always
read that as a contact (orient exactly 0), and the verifier's law (a)
does the same; the end-point rule keeps that reading unchanged. Pinned by
`a_contact_is_no_crossing_and_a_micron_past_is`, which now also pins
that a vertex a rounding PAST another chord's interior crosses.

**Considered and dropped.**
- *Within `TOUCH` of the other chord's line* (this PR's first head).
  Hides a pass-through, above. Dropped.
- *Pass-through detection at a near-line vertex* (look at the walk's
  previous and next points). Also closes the hole, but needs walk
  topology in a per-pair predicate and would differ from the verifier's
  rule. Dropped.
- *Shared ends excused, all other contacts crossings* (the brief's
  shape). Breaks the retraced lifted pieces above. Dropped.
- *Snapping walk vertices together before the check.* It would change the
  posted path, not only the check. Dropped.

**Residual, unchanged in kind.** A walk crossing a chord at a vertex
EXACTLY on its interior, or two walks crossing in an X exactly through a
shared vertex, were never reported by the exact-sign law (every
orientation there is 0); the verifier reads them the same way. The
end-point rule extends that only to a vertex a picometre from another
chord's END point.

**Cost to change later.** One constant and one predicate in `guard.rs`,
kept equal to the verifier's; the pins in `guard_tests.rs` name each case.
