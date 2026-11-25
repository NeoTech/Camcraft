## #3679 - A feed plunge deeper than one stepdown below the uncut floor is named

Taken unattended, as the recommended option under the owner's standing
word (Autonomous Work Law). The issue's Expected offers two forms: "The
finding blocks the post, or it is a named warning that the user must
see." Taken: **the named advisory form**.

**What was decided.**

- A new verifier law, `camcraft_sim::verify::plunge_depth` (law (d3)),
  judges every non-rapid axial descent (feed or plunge, XY within
  `z_tol`, descending) against the rest model's floor: for each disc
  sample of [`rapids::footprint`] at radius `r - tightened(gouge_tol)`,
  the bite is `from.z.min(rest.surface_before(q, i)) - (to.z +
  cutter.lift_at(rho))` where stock stands (`rest.in_stock`) - the
  move's own take, never the column above its start (the
  reconciliation below). The move is named once, at its
  worst sample, when `!(bite <= limit + z_tol)` with `limit` the op's
  stated stepdown. The `!(.. <= ..)` form fails closed: a non-finite
  stepdown or a NaN bite is a conviction, never an
  acquittal. `Mode::Revolve`, `Mode::Radial` and `Mode::Rotary` stand
  the law down (law (d)'s own words: the XY column is not those paths'
  geometry). The program ahead's surface acquits a conviction
  (`acquit_plunge_by_prior`, wired beside `acquit_by_prior`).
- The gate is the new `VerifyContext::plunge_stepdown: Option<f64>`,
  the same `None` discipline as `finish_tol`, `model`, `spindle` and
  `lathe_axis`: `None` keeps the law silent and every stored verdict
  byte-identical. The operation layer
  (`crate::cutting_rows::plunge_stepdown`) sets it for the kinds that
  cut in vertical levels and state a stepdown: 2D Contour, Pocket,
  Offset2D, Slot, TrochoidalSlot, Adaptive2D, Pocket3D, MultiAxisRough
  and the staged Face. Drills, bores, threads, the turning, wire and
  plasma families keep `None`. A tilted (3+2) frame keeps `None`: the
  law reads world-Z descents, and a tilted plane's descents answer to
  its own entry method's law.
- The finding is `Warning::PlungeTooDeep { at, mm, limit_mm }`, MCP
  kind `plunge_too_deep`, UI line 227 - **advisory**, in the
  `blocks_post` exemption list beside `FluteShort`.

**Why advisory, not blocking.**

- The maintainer's P3 re-triage: "Nothing gouges or collides, and the
  posted part geometry is correct. The risk is the tool load." This is
  the #3525 pattern exactly - there the owner's standing word chose
  "the diagnostic exists to name the cause; the collision checks decide
  if the post is held", and `FluteShort` (a cut deeper than the flute)
  is advisory for the same load-class reason.
- The generator still produces these dives at every tabbed contour
  window with a lead (#3678 owns the fix; PR #3684 pinned the later
  windows' 18 mm crescent dives on purpose). A blocking law would hold
  the post of every such program - including the example catalog case
  `2d/contour-tabs`, whose contract test regenerates and posts the
  committed NC - until #3678 lands. The Done means asks that the
  catalog case "gives the same finding until #3678 is fixed", not that
  it stops posting; the advisory form gives the finding with no red
  suite and no known-fail marker to remove later.
- The crescent case convicts on its own reading either way: at the
  dive station the samples beyond the swept band read the stock top
  (6.0 mm above the last level of the catalog case against its 2.0 mm
  stepdown), the samples inside it read the level above (one stepdown,
  clean), so the worst sample names the crescent exactly as the
  maintainer narrowed it - centre-cutting or not, the full-disc depth
  engagement is the reading, per the issue's own words.

**Considered and dropped.**

- **Blocking** (the context pack's first form): dropped for the
  reasons above. It would also have required known-fail markers
  (#2567) on `catalog_2d_contour_tabs::{regenerates, rebuilds}` and on
  `contour_whole_step`'s clean-verdict cases, all pinned to #3678 and
  all to be removed again by its fix.
- A generator-side peck guard in this PR: #3678 owns it; this PR is
  the verifier law only (the coordinator's scope split,
  `docs/decisions/3453-tabbed-contour-entry.md`).
- Judging the descent's full swept column (stations along a nearly
  axial move): a descent within `z_tol` laterally is one column in
  practice; the lateral and ramped feeds are the level law's, not
  this law's.
- A mode gate for `Undercut`: dropped - `Rest` takes no cut there, so
  the floor is the stock top and an undercut Z-dive names toward
  safety; the app wires no undercut kind, so the branch runs only in
  hand-built contexts.
- Wiring `Circular` (the ring pocket family): dropped with the context
  pack's list - `steps_mut` does not seed it, and the ring family's
  own hops dive in their own kerf; a later PR can add it with its own
  pinning tests.

**Reconciliation of the six red suites (the gate run after the first
commit).** The law as first committed read the whole column below the
rest floor and this op's own floor only; six pre-existing app suites
over-convicted. Two readings were wrong, both fixed in the law and its
wiring - no test's expected verdict was changed, and no slack beyond
the established `z_tol` was added.

1. `vertical_rapid_stock.rs`, `a_vertical_rapid_into_a_taller_block...`:
   the slot's entry rapid broke the raw top (law (d) names it and holds
   the post), and the feed below it took 2 mm. The law read the whole
   column, 5 mm. **The bite reading**: a plunge move takes only the
   material between its own start and end, so the bite's floor is the
   LESSER of the move's start and the rest surface. A dive from air
   keeps the old reading whole (the min is the floor there); a start
   the rest cannot read falls back to the start, an upper bound the
   bite cannot pass.
2. `vertical_rapid_stock.rs`, `a_dive_into_stock_an_earlier_face_removed...`:
   the face left Z 5 standing; the law read the raw top Z 10. Fixed by
   the bite reading (2 mm of its own descent) and by the prior acquittal
   below.
3. `carried_stock_entry.rs`, both `assert_safe_entry` failures: the 18
   trochoid fires were microlift-return plunges of 0.3 mm reading the
   raw 10.5 column (the bite reading acquits them); the one contour
   plunge read 10.5 where the face had left 10.0, 4.0 = its stepdown
   exactly. **The prior acquittal**: `acquit_plunge_by_prior`
   (sim `prior.rs`) re-reads each conviction against the prior
   program's surface, the same third term of the min
   `acquit_by_prior` gives a dive, conservative the same way (the
   HIGHEST column under the full radius, `None` keeps the conviction,
   `!(.. <= ..)` reads a NaN limit as no acquittal). The wiring
   (`generate.rs`) cuts the prior when either law convicted.
4. `clearance_in_stock.rs`, `the_same_plane_over_stock_an_earlier_face...`:
   the faced block acquits the dive; the prior acquittal removes the
   advisory, the first-traverse conviction stands as before.
5. `top_stock.rs`, `facing_to_top_removes_the_height_warning...`: the
   same face-acquitted dive; the prior acquittal removes the advisory.
6. The slack question: the dispatch direction offered a wider slack
   for the 0.5 mm trochoid overage. Dropped: the overage was not a
   tolerance problem but a floor problem (the raw top read where the
   face had cut) and a bite problem (the column read where the move
   takes 0.3 mm). The law keeps the established `z_tol` slack alone;
   widening it would weaken every conviction margin for no honest
   reading gained.

The crescent conviction is unchanged by both fixes: the reporter's
dive starts in air, so the bite is the full column below the uncut
floor, and no prior program cut the sheet at the tab window. The
`plunge_depth` sim binary pins the bite (`a_feed_that_a_rapid_already_
stood_inside_stock...`, `a_feed_plunge_buried_deeper...`), and the
`prior` unit tests pin the acquittal.

**Limits.**

- A plunge into a STEPPED stock's low step still reads the raw grid's
  top (`stock_top_z`): the solids-based re-read that `acquit_by_stock_top`
  gives a rapid (#1091) is not wired for the feed plunge, because a
  plunge removes material and "the whole stack stands above every
  facet" is not its reading - the flute disc's own local solid top
  would be the third floor term. No failing suite covers it; a later
  PR can add it with its own pinning tests.
- An over-deep dive still clears its own column in `Rest::cut`, so a
  later re-entry above the dive bottom reads that column as cleared.
  Whether an unsafe dive should stand as a proven floor for the later
  laws is filed as its own follow-up issue (P7).
- The reading is bounded by the footprint's own sampling bound (law
  (d)'s: stock between two samples by less than a ring spacing can
  pass), the same bound every disc law here shares.
- The tilted 3+2 frames stay unnamed until a pose-aware reading exists.

**Cost to change later.** The variant name `plunge_too_deep`, its
fields and the context field `plunge_stepdown` are part of the project
file and of the MCP output from this PR on. To make the finding block
the post, remove it from `blocks_post`'s exemption list and mark the
catalog case's suites known-fail until #3678 lands.
