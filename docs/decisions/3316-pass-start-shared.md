## #3316 - "Pass starts at" is one shared parameter (2D Offset first)

**What was decided.**

- One domain type, `machining2d::PassStart { Auto, Inside, Outside }`,
  in `crates/motion/src/machining2d/pass_start.rs`. `app` re-exports it.
  Every kind with a pass order uses this type. No kind gets its own enum.
- One project key, `pass_start` (`project::PassStartDto`, snake case).
  `auto` is the default and is never written. Files without the key read
  as Auto. There is no version step, and every file without a choice
  stays byte-identical.
- One command: the field is a plain `OpParams` field, so
  `EditOperationParams` carries it (undo and redo included). One MCP
  tool: the generic `edit_operation` merge patch. The patch guard keeps an
  explicit `"auto"` (`pass_start_kept` in `mcp/edit.rs`), because a
  skipped default never comes back from the round-trip.
- One Passes-tab row, `Pass starts at`
  (`crates/ui/src/egui_app/pass_start_row.rs`). A kind shows the row when
  its pane passes `Some(&mut pass_start)`.
- The digest writes the value only off Auto, so every earlier operation
  keeps its cache stamp.
- The ring law (review fix after the second read, `level_rings.rs` and
  `level_order.rs` in `machining2d`):
  - Auto: the planner's old core-first order (`waterline_passes`),
    byte-identical. Inside on an area with no hole is Auto too.
  - Outside: rings are level curves of the distance from the outer edge
    only. Ring 0 is the stretch of the area's boundary on its outer edge.
    Ring j is the outer edge (holes filled) eroded by j stepovers and
    clipped to the cutter-centre area, as OPEN pieces
    (`Region::clip_ring`, new in `geo2d/clip_line.rs`). The walls (the
    holes and the "Do not touch" keep-outs) are cut last, with the stock
    to leave. Inside is the mirror: ring j is the holes grown by j
    stepovers, the outer wall last.
  - Each part of the material is planned from its own edges: an outer
    loop with the holes directly inside it. Material inside a hole (the
    bore of a hollow boss cleared to the pocket floor, the stock inside a
    ring part) is a part of its own, at any depth of nesting. Review fix 2
    (third read of 9939a6238): the first fill read the outer loops with
    `Region::new`, which turns a nested outer loop into a hole, so such
    material got one ring along its edge and its middle stayed standing
    (153.7 mm2 on the ring-part fixture, 26.4 mm2 on the hollow boss).
    Dropped: `Region::union_all` of the outer loops (the reader's
    proposal). It cuts the nested material with rings of the FAR outer
    edge, so its first cut lands in its middle, at the full width; a part
    of its own starts on its own edge and is named when that edge is a
    wall.
  - Inside on a part with no hole (material nested in a hole): that part
    is cut from the middle outward, as Auto cuts it, with the note
    "Pass start Inside: N part(s) of this area have no hole to start
    from, so they are cut from the middle outward, as Auto cuts them".
  - Each connected piece of the area is cut on its own, ring by ring, the
    nearest piece next. Each piece is walked with the uncut side on the
    right for climb.
  - The completeness guard (Prime Directive: no silently short path).
    After the plan of a level, the passes are swept by the tool disc and
    compared with the cutter-centre area. If more than 0.1 mm2 of it
    stays unswept (read after a 50 um erosion, so arc flats and ring-end
    slivers do not count), the plan is not posted: the level takes the
    Auto plan, and the toolpath says "Pass start <side>: the rings from
    the start side would leave A mm2 of this area uncut - Offset rings
    cut this area in the Auto order instead". Chosen over marking the
    operation incomplete: Auto is a complete, proved plan, and the
    warning names why the order changed. Dropped: a refusal, which would
    leave the user with no path for a case the planner can still cut.
    The note carries `machining2d::RINGS_FALLBACK`, so `rings_fallbacks`
    counts it and the op posts the advisory `Warning::StrategyFallback`
    (#3035) for the area. The maintainer decided this WARN behaviour
    for the whole pass-start family: every fallback from the requested
    ring order raises that warning, not only a note.
  - A piece joins the pass before it only by a chord that stays in the
    area, crosses no planned cut and runs through cut material all along
    (each sample inside the swath of a walk already cut, 0.9 tool radii
    from it). Else the pass ends, and the next pass starts on a vertex of
    a walk already cut, with a lead-in chord of the same law to the
    piece: its re-entry plunges into a cleared column, not into solid.
    Where no lead-in is proved (a stepover of 0.9 radii or more), the
    piece re-enters by the user's entry (`entry_restart`, #2265).
  - Between passes, a keep-down pass link must also run through cut
    material (`through_cut`, columns cleared by
    `plunge_law::column_cleared`), or it lifts to the retract or
    clearance plane by the existing rules.
  - The end of a ring piece on the area's boundary is inserted as a
    vertex of the boundary walk (ring 0 or a wall), and the walls are
    clipped from the same walk as ring 0: the walks meet at shared
    points, so the verifier's crossing law reads touches, not crossings.
  - Cost on fig8 (debug, motion only): Auto 9.5 s, 24768 moves, 18
    lateral rapids; Outside 26.6 s, 25986 moves, 93 lateral rapids, 15 %
    more feed length (the lead-ins and the lifts between pieces). The
    first version without lead-ins took 290 s: each piece re-entered by
    a ramp along the pass.
  - What the tests prove: on the second read's band (stock 84 x 24,
    reach 100 x 40, a round part of radius 15, tool radius 4, step 4) the
    widest cut along every level is 4.87 mm (the old order: 7.97 mm, a
    full-width cut along a part-side arc). With a "Do not touch" pin:
    4.87 mm. Inside from an open bore: 4.52 mm. No two planar feeds
    cross (Outside and Inside, steps 4 and 3). The bound pinned is one
    step plus a quarter radius (5 mm), no overcut, no interior leftover.
    On fig8 with EM-0800-4FL no Outside cut has uncut stock on both
    flanks (the old order: 8697 such points). The two fig8 Outside
    generation tests run in the app heavy lane (about 170 s each in
    debug with the verifier).
  - What they do not prove: no claim that a ring is never wider than one
    stepover in general. A cut takes the full tool width where the first
    ring runs along a wall (Outside on a closed pocket, Inside on an
    Outside band, Inside round an island), or where the area is narrower
    than the tool plus one stepover, or where a wall hides the start
    side. Each case writes a "Pass start" note on the toolpath, and the
    op page shows it as a warning line. The stock footprint (in the
    area's frame) tells a start ring in open air from one on a wall.
- Only 2D Offset reads it in this PR. The motion entry points are
  `pocket_started_on` and `pocket_avoid_started_on` (with the stock);
  `pocket_started` / `pocket_avoid_started` take a bare `PassStart`, and
  the old `pocket_entered_with_relief` and `pocket_avoid_with_relief`
  call them with Auto. `plan_passes` is unchanged, so the 3-D pocket, the
  HSM fallback and rest machining are not changed.

**Room for the #1448 start point.** The start point (where on its ring a
pass starts) and the exit point go beside `pass_start` as their own
fields with their own key. The planner input is the pair: the level
curves choose the order, the start point picks where a whole ring starts. The
same entry functions take the new input when #1448 lands.

**Why.** The maintainer asked for one parameter with one shape for every
kind (#3316 comments, 2026-11-22). The field rides the existing command
and MCP door, so the GUI and agents use the same undoable path.

**Considered and dropped.**

- A `pass_start` field on `PocketParams`. About 57 struct literals
  without `..` would change for one value that only one kind reads now.
  An argument on the new entry functions keeps every other caller
  byte-identical. When Pocket and Adaptive take the parameter (their
  sub-issues), the field can move into `PocketParams` in one step.
- A change of Auto for an Outside band. The maintainer said Auto keeps
  the current behaviour.
- A pure distance-from-boundary rank. It interleaves the two families
  where the band width changes, and then a ring can have uncut stock on
  both sides.
- The first fix of this PR, a rank of whole loops (`PassStart::rank`:
  counter-clockwise loops before clockwise loops). The second read showed
  that a ring that splits at a narrow place gives closed loops with
  stock-side and part-side arcs; the part-side arcs then cut the full
  tool width (7.97 mm on the meter). No order of whole loops can cut one
  loop's two sides at different times.
- Clipper2's open-subject clip (`add_open_subject`) for the level curves.
  No code in the repository used it, its output direction and the join
  of a ring's wrap need repair afterwards, and an open-path defect in the
  port would be a gouge risk. The own clip splits each segment at every
  boundary crossing and keeps a piece by the parity of its midpoint: the
  direction is kept by construction and every point lies on the input.
- A warning from the containment alone. An enclosed pocket under an
  Outside containment is closed by walls, so the stock footprint decides.

**Cost to change later.** Low. The type and the key are shared already.
A new kind adds its field, its row call and its planner read.
