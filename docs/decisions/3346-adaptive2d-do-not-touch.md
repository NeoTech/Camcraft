## #3346 - "Do not touch" on 2D Adaptive operations

**Decided:** `do_not_touch_applies` accepts `OpType::Adaptive2D`, for both
methods. The new module `machining2d::adaptive_avoid` gives one law to the
two methods:

- The keep-out is one region. It covers each tip height from the floor up
  to the clearance plane, so it holds every cut, every entry and every
  descent of the path.
- The planner cuts its cutter-centre area back by that region, then plans
  as it does without protection:
  - **Spiral** (`adaptive_spiral_avoid`) calls `pocket_over_with_relief`,
    the body of the plain pocket over a given area (rest machining uses it
    too). The crossing law, the entry and link certificates and the
    Offset-rings fallback of the plain planner all read the reduced area.
  - **Trochoidal** (`adaptive_trochoidal_avoid`) calls the plain planner
    with the reduced area (`trochoidal_inside`). Its move check
    (`check_path`) reads the reduced area.
- **Pass starts at** (#3355) is read on the protected Spiral path too
  (`adaptive_spiral_avoid_on`): the plan takes the `StartSide` over the
  area that the keep-out leaves, as the plain planner takes it over the
  full area. The Trochoidal method still refuses Inside and Outside by
  name before the keep-out is built.
- Rapid links are lifted over the items with `elevate_rapid_links`. A link
  that cannot clear them refuses the operation.
- A corner relief that enters the keep-out refuses the operation, as on
  2D Offset.
- Without protected items, both methods call the same planner as before.
  A keep-out that does not reach the region also runs the plain planner,
  so the path stays the one it always was.
- The generic checks in `generate.rs` (`route_links`, `verify_avoid`) and
  the post gate (`live_avoid_problems`) already read every kind that
  `do_not_touch_applies` accepts. They needed no change.

**Why:** the plain planners already hold the laws that make a 2D Adaptive
path legal. With the reduced area they hold them for the protected case
too, and no second copy of the pass logic is needed.

**Dropped:**

- `pocket_avoid_with_relief` for Spiral (the per-level planner of
  2D Offset). Its Adaptive branch was tried first. On a plate with a
  protected tower in the middle it took 95 s in debug, and its helix
  entries and its fitted Offset-rings arcs crossed (the verifier held the
  post). It does not have the crossing law of #2240 that the plain pocket
  has (`post_crossing_free`). That gap is in 2D Offset's planner too; it
  is not changed here.
- A keep-out for each level. The plain planners plan one area for all
  levels. A plan for each level is a new planner, not a fix.
- A refusal of "Do not touch" for the Trochoidal method. The full-height
  keep-out gives a safe path in the common case (an item at the side of a
  channel), so a refusal would refuse legal work.

**Cost:** beside a protected item that is lower than the top of the cut,
2D Adaptive leaves more stock than 2D Offset: the upper levels lose the
same area as the floor. The guide says so. When the keep-out leaves a ring
round an item, the adaptive plan falls back to Offset rings, and that
fallback is slow in debug (a heavy-lane test pins it). A keep-out for each
level can come later without a file change.

**Cost to change later:** low. The change is one module in `motion`, one
`match` in `generate.rs` and one line in `do_not_touch_applies`.
