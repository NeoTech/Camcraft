## #3824 - 2D Offset "Pass starts at": the part wall reads as one wall, and the pass link tries the edge route

**Decided.** Two changes, both in the motion layer:

1. **The side tolerance of the ring set** (`level_rings::SIDE_EPS`) goes
   from 1 um to 10 um. `sides` splits the boundary of the cutter-centre
   area into ring 0 (the start side) and the walls. It reads a stretch as
   the start side when the stretch stands within `SIDE_EPS` of it. The
   area and the start side are two offsets that flatten the same round
   joins apart, each within Clipper's 5 um (`ARC_TOL_UNITS`). At 1 um,
   two stretches of the fig8 part wall (0.117 mm each, at X about +-46.5,
   Y about 123.2, where the band is 5 mm wide) read as the stock side:
   - Inside: the wall ring was cut in two pieces with a gap at each
     stretch, and each gap gave a lift (3 per level, part of 6);
   - Outside: the two stretches were ring 0, cut first, each reached by
     a lift across the part and entered by a plunge into material (4 per
     level, the four links of #3825).
   10 um is twice the flattening error, and still far below any length
   that a cut can see. The tolerance is a distance from the start side,
   not a length: a real wall stretch of any length that stands more than
   10 um off the start side stays a wall. The unit pin
   `ring_side::tests::the_side_tolerance_keeps_a_real_short_wall_and_drops_a_flat_sliver`
   holds this: a 0.1 mm stretch 50 um off the start side stays a wall
   piece, and a stretch 5 um off reads as the start side.
2. **The pass link tries the edge route** (`ring_links::link`, called by
   `ladder_links::pass_link` round a part). When the straight chord to the
   next pass start fails, the link tries `ring_links::edge_route`, the walk
   that the planner's join of `level_order` already uses. Each chord of the
   route gets the proof of the straight chord (`ring_links::chord_proof`:
   inside the certified area, no crossing of a planned pass or of a link
   cut on that plane, each sample in air or over a cut column). The whole
   route stays within the stay-down limit. The route is recorded as a cut
   of the level. A lift stays only when the chord and the route both fail,
   and it keeps the reason of the straight chord, so the `link:` notes are
   the same texts.

**Readings** (fig8, EM-0800-4FL, release):
- `lifts::fig8_start_sides_lift_only_with_a_named_reason`: with the edge
  route alone, Inside 6 and Outside 12 lifts (the 18 of #3824, the same
  as before). With the side tolerance: Inside 0, Outside 3. The 3 that
  stay are one short hop each level, from (-8.887, -36.945) to
  (-7.926, -33.062), named "would cut uncut stock with the full tool
  width".
- `lifts::fig8_outside_pass_links_keep_off_the_part_and_enter_on_cut_floor`:
  0 lifts across the part (4 each level before) and no "plunged into
  material" note (6 before).
- `flute_reach::a_ten_mm_flute_names_its_cause_before_the_rapid_findings`:
  3 rapid findings (9 before).
- `pass_start_levels::ring_links` (debug): disc and dog bone 0 lifts on
  each side, as before. The clamp case (`lift_reason_is_named`): Outside
  26 lifts, as before; Inside 4 lifts (3 before) and one named "second
  dive" note. The pass start there (-3.29, 15.66) is where the clamp's
  rounded keep-out meets the part's ring almost at a tangent, so the
  meeting point moves by micrometres with the side tolerance: 2 lifts at
  2 um, 4 lifts from 3 um to 10 um. Each lift is named; the test holds.
- The edge route of the pass link changed no reading above: on these
  cases no failed straight chord had a clear edge route. On fig8 the two ends of each failed link stand on different
  loops of the area, or the pass start stands in uncut stock, so no edge
  route passes the proof.

**Why.** The maintainer's words on #3741: the tool "should have just
followed the contour keeping its plan not lifting the tool". The second
review of #3818 read the remaining lifts as a gap of the pass link (no
edge route). The readings show that the edge route alone changes nothing
on fig8: the passes that the tool lifted to were slivers of the part wall
that the side split read as the other side. The route stays, because the
pass link must try every in-plane link that the planner's join tries
before it lifts (#3824 "Expected").

**Dropped.**
- A side tolerance of 2 um (the clamp case lifts 2 times on Inside): it
  is below the 5 um flattening error, so fig8's kind of sliver can come
  back on another part.
- A filter that moves short stretches to the class of their neighbours:
  more code, and it treats the symptom; the cause is a tolerance below
  the flattening error.
- A new lift reason for a failed edge route: the straight chord's reason
  is the useful one, and the note texts stay the same.
- Changes to the entry code (#3746, PR #3823) and to the ring-law
  predicate of the pocket and Adaptive (#3822): out of scope.

**Cost to change later.** Small: one constant in `level_rings.rs`, one
function in `ring_links.rs`, and the walk that `pass_link` records.
