## #894 - Thread-mill tool kinds and a reworked thread-milling operation (first slice)

**What was decided**

1. **Two tool kinds, one new field.** `ToolKind::ThreadMill` (multi-form: rows
   of teeth at one pitch) and `ToolKind::ThreadMillSingle` (single-form: one
   row, a pitch range). They read the existing `diameter` (across the tooth
   crests), `tip_angle` (the form's included angle), `pitch` (the multi-form's
   pitch, the single-form's coarsest), `flute_len` (cutting length; rows =
   flute length / pitch on a multi-form), `flutes` and the shank fields, plus
   one new `Tool` field, `pitch_min` (the single-form's finest pitch). File
   words: `thread_mill`, `thread_mill_single`, key `pitch_min` (written only
   when set; no project-version step - additive, like `orientation`).
2. **A neck is the drawn cutter outline, not a new field.** The spun cutter
   profile (#93) already draws cutter, neck and shank; a single-form mill's
   narrow body is its `shank_d` until an outline says more.
3. **A Tap is no longer a thread mill.** `tool_law::THREAD_MILL` was `[Tap]`
   ("the thread-mill cylinder is modeled as a Tap"). It is now the two
   thread-mill kinds. An existing Thread operation that names a Tap shows the
   tool law's after-state ("...'s tool is a tap; a Thread operation cuts with
   a thread mill or single-form thread mill") and refuses to generate or post
   until the tool is re-kinded. Breaking, on purpose: a rigid tap on a helical
   orbit is a crash, and one cylinder cannot say whether it has one row of
   teeth or twenty.
4. **The kernel was rewritten, not extended.** The M2.4 C ride stepped the
   radius out by `infeed` on every turn while it descended, so the threads
   near the top were shallower than those at the bottom; it retraced itself
   (the crossing law convicted every pair) and stood past the hole's rim (the
   tangency law convicted every move) - no Thread operation could post. Its
   climb setting also wound counter-clockwise while descending, which is a
   LEFT-hand helix (`cl::ThreadHand::of_turn`). The new ride:
   - radial passes are full helices over the whole thread, equal steps, the
     last on the thread depth (`infeed` is now "radial depth per pass");
   - a single-form mill (one row) winds whole turns; a multi-form mill makes
     one revolution per axial level, levels one cutting length apart;
   - direction follows the handbook rule under an M3 spindle: climb orbits a
     hole counter-clockwise and a boss clockwise; the hand then fixes the
     axial direction (right-hand: counter-clockwise rises). Right-hand
     internal climb starts at the floor and winds up;
   - lead-in and lead-out arcs are tangent to the centre circle and travel
     axially at the helix's own rate per unit of path. In a hole the lead is
     the semicircle from the axis, which is exactly on the helix (inscribed
     angle theorem); beside a boss it is a quarter arc of the tool's radius;
   - every pass meets angle zero at the same height, so later passes deepen
     the same groove;
   - the tip never goes below the floor (a through hole: one pitch past the
     exit, unless the author states a bottom); no rapid runs below the
     feature's top;
   - the feed is stated at the cutting edge and programmed at the tool centre
     (x centre circle / edge circle) - in a tight hole the centre moves far
     slower than the edge, and programming the edge feed at the centre would
     overload the cutter several times over.
5. **The pitch belongs to the thread, with the tool as its judge.** The
   operation gains `pitch: Option<f64>` and `hand`. Unstated reads a
   multi-form mill's own pitch (what every older file meant); a single-form
   mill needs it stated and inside its range; a multi-form mill refuses any
   other pitch. Only the 60-degree form is cut - another form angle is
   refused by name.
6. **Two readings of the tool for the gouge laws.** A thread is cut into the
   wall the model shows (CAD draws a tapped hole at its minor diameter), so:
   - the **crest law**: the tangency law reads a region whose hole rims stand
     at the thread's MAJOR circle with the tool's full radius - a crest past
     the major diameter is a gouge;
   - the **root law**: the solid law reads the cutter narrowed by the thread
     depth - the body under the teeth may not enter the part, radially at the
     minor wall and with no allowance at all at the tip (a blind floor).
   The program simulation sweeps a thread mill at its tooth roots, as it
   already does a tap (#266).
7. **The crossing law stands down for Thread.** Radial passes and levels
   retrace one helix, and successive passes' lead arcs leave the axis tangent
   to one another; "feed moves cross in XY at shared depth" is the operation.
   Dropped in the app (`thread_op::stand_down_crossings`), not in the sim
   crate.
8. **Doors at command creation.** `CreateOperation`, `EditOperationParams`,
   the kind retype and the tool swap all ask `thread_op::door`: form angle,
   pitch against the tool, infeed against the thread depth, tool against each
   hole's minor diameter. The generator asks it again.
9. **External threads: kernel only.** The kernel cuts a boss (depth 0.6134 x
   pitch, clockwise climb, quarter-arc leads) and is pinned by tests and a
   golden, but the operation passes `external: false`: there is no boss
   recognition or pick yet, and the static footprint region cannot judge a
   tool standing beside a boss. Filed as a follow-up.

**Why**

The issue's "Done means" is a thread mill that can be stored and used by an
operation that verifies clean and posts helical G2/G3 with Z. The stored half
needed the kinds; the "verifies clean" half could not be met by the old ride
under any tool, so the ride and the two gouge readings had to come together.

**Sources (public)**

- ISO 68-1 (basic profile), ISO 724 (basic dimensions: internal depth
  5/8 H = 0.5413 P), ISO 965-1 / DIN 13 (external design root: 17/24 H =
  0.6134 P), H = 0.866 P.
- Machinery's Handbook, "Thread milling" (single- and multiple-cutter
  milling, helical interpolation, climb direction, feed at the cutter centre
  for internal and external circular paths).
- Peter Smid, "CNC Programming Handbook" - helical milling and thread milling
  chapters (lead = pitch, tangential arc entry, right- and left-hand
  direction tables, feed-rate correction for circular motion).
- RS-274 / ISO 6983: G2/G3 with a Z word is helical interpolation.
- Elementary geometry: the inscribed angle theorem (a point on a circle
  through the axis sees the axis at half the arc's central angle), which is
  why the semicircular lead-in from the axis keeps a constant axial rate per
  degree about the thread axis.

No HSMWorks / Fusion material, output or documentation was consulted for the
ride; the operation's name and place in the catalog predate this PR.

**Considered and dropped**

- *Reusing `width` or `nose_radius` for the finest pitch* (the house habit of
  re-reading inert fields): the tool library is a user-facing file, and a
  vendor library that writes a pitch under `width` is a trap. One honest key.
- *One kind with a "rows" number*: the issue asks for two kinds, the two
  behave differently at the door (range vs one pitch), and the tree and the
  tool law name them apart.
- *Keeping Tap as a legacy thread mill*: it would silently change what an old
  Thread operation cuts (one revolution instead of a spiral) - refusal by name
  is the Prime Directive's answer.
- *Negative stock-to-leave as the thread allowance*: it would also let the tip
  enter a blind floor by the thread depth. The root cutter keeps the tip law
  exact.
- *Judging the tool at its core everywhere (`Tool::geom`)*: fixture, rapid and
  reach laws would under-read the cutter by the tooth height.
- *A new verify `Mode` without the crossing law*: touches the sim crate's law
  table for one kind; the app-side filter is one line and says why.
- *Lead-in arcs staggered so they never touch*: needs air the tightest holes
  do not have, and leaves the helix.

**What it would cost to change later**

- `pitch_min` is in the tool file grammar; renaming it needs a reader alias.
- The Thread digest now carries pitch and hand, so every Thread operation
  made before this reads Stale once.
- Other forms (55-degree Whitworth, NPT taper, trapezoidal) need a depth law
  per form and, for taper threads, a different ride; the form door is the one
  place that refuses them today.
- External threads at the operation level need boss recognition or a pick, a
  depth-aware region for the crest law, and the `external` flag in the params
  and the file.
- The simulation shows the thread as its minor (root-swept) cylinder; drawing
  the helical form needs a new sweep atom.

## The crest allowance needs a round wall (merging #1205, 2026-11-12)

**Decision.** The two thread readings - the crest region with each rim moved
out to the major circle (`crest_region`) and the solid law at the tooth
roots (`root_cutter`) - apply only when every threaded hole is a round wall
in the model: each wall facet the crests can reach (minor radius + thread
depth + `WALL_SAG`) within the thread's Z span has its sagitta about the
hole's axis at most `WALL_SAG` (0.05 mm), measured by #1205's own
`facet_sag` (`verify::hole_is_round` in `hole_wall.rs`; the constant is not
duplicated). Otherwise (an octagon, a D-bore, any flat in reach) the
operation keeps its region's own rim and the full tooth radius, so a crest
cutting into a flat is a gouge. The stricter reading wins, consistent with
#1205.

**Why.** Main's #1205 law (no hole-wall credit for a polygon's flats) and
this PR's thread laws were each right alone; merged, the crest region
erased the octagon's rim and the root cutter excused 0.32 mm of the crests
in the flats - 0 warnings where the kernel helix with the full cutter
finds 14 gouges. A thread is cut into a round hole; there is no thread
form in a flat to excuse.

**Pins.** `hole_wall_flats::a_thread_mill_cutting_into_an_octagons_flats_is_a_gouge`
(convicts again), `hole_wall_flats::a_thread_op_in_a_finely_faceted_round_hole_stays_clean`
(96-gon, clean), `hole_wall::a_hole_is_round_only_while_its_reachable_wall_is_a_fine_circle`,
and the root cutter's unit test (full radius when not round). The #1005
thread goldens regenerate byte-identical.

**The thread golden is judged, not UNJUDGED.** `thread_mill_m8_multi_form`
runs through the real operation in `posted_gates/thread.rs` (proved equal
to its CL golden, then the generator's blocking warnings plus the post's
`verification_gate`): it passes, so it joins `every_posted_golden_is_a_program_the_gate_passes`
with a real verdict (`the_thread_golden_passes_its_gate`).

**Considered and dropped.**
- *Intersecting the crest region with the model's own hole*: fixes the
  region law only; the root-radius solid law would still excuse the flats.
- *A crest-radius solid check beside the root one*: convicts every thread
  in a round hole (the crests are meant to pass the drawn minor wall).
- *Per-hole rather than per-operation*: one Thread op with one polygonal
  hole judges all its holes at full radius - stricter, never looser; per
  hole needs a cutter per move, deferred until an op threads mixed holes.

## The refresh against main at 0ad525d8 (27 files; #1071, #1220, #532/#541, #890/#898, #1093)

Main moved under this PR: it fixed the same Thread verdict on the old kernel
(#1071, PR #1099: `thread_rims::major_rims`, `verify::thread_bore`,
`solid_hits_threaded`), split the tool digest out of `stamp.rs` (#532), and
added the sinker electrode kind, the tool catalog fields and the cutting
rows. Most of the 27 conflicts were both-sides-added lists and are kept
both; these are the judgements.

**1. Region law: main's `major_rims` replaces this PR's `crest_region`,
behind this PR's round-wall gate.** Both moved a threaded hole's rim out to
the major circle. Main's grows it from the model's own rim loop at its
nearest point (a chord's middle, credited at most `CHORD_CREDIT` 0.01 mm)
when that stands nearer than the declared circle; this PR's grew it from
the declared radius only. Main's circle is never wider, so it is the
stricter reading. `walls_round` (#894 x #1205) still gates it: a thread
whose reachable wall is not a fine circle keeps its region's own rim.
`crest_region` and its unit test are removed (nothing calls it).

**2. Solid law: main's thread grant replaces this PR's `root_cutter`.**
This PR narrowed the cutter by the thread depth, which credits that depth
against every facet. Main's law keeps the full cutter and excuses a hit
only on the thread's own wall or mouth, while the swept piece stays inside
the major cylinder of the model's own wall (`verify::thread_bore`). Main's
excuses strictly less: a nearby wall of another feature, a floor, a hole
declared wider than or off the centre of the model's, and a polygon side
that dips more than `CHORD_CREDIT` (a 24-gon on r 5 dips 0.043 mm: this
PR's 0.05 mm `WALL_SAG` test read it round and credited it whole) all
convict. The two are not stacked: a narrowed cutter under the grant would
let crests pass the major circle by another thread depth. `root_cutter`
and its unit test are removed; the verify context reads the full cutter,
as on main.

**3. The grant reads the thread's own pitch, and only over a round wall
(`thread_op::bores`).** Main called `thread_bores(&params, tool.pitch)`.
With this PR's kinds a single-form mill's `pitch` is the coarsest of its
range, and the thread's pitch is the operation's: a 1 mm thread cut with a
0.5 to 1.5 mm mill would have been granted a 1.5 mm thread's depth
(0.27 mm too much). The grant now resolves the pitch the way the kernel
plan does (`thread_mill::resolve_pitch`), and is empty when `walls_round`
is false or the pitch does not resolve. Pinned by
`thread_op::tests::the_thread_grant_reads_the_threads_own_pitch_and_only_over_a_round_wall`.

**4. The crossing stand-down stays.** Main's #1071 made touching chords no
crossing, which cleared the old kernel's retraces. Tried on the merged
tree with the stand-down off: this PR's rides still convict (two
`thread_mill` tests and five of main's `thread_mill_rim` tests fail), since
successive radial passes' lead arcs do cross in plan at one depth. The
stand-down is kept as this PR designed it, for Thread only.

**5. Tool kind numbers: main's Electrode keeps 13; the thread mills take 14
and 15.** In the stamp digest (`stamp_tool.rs`, where the kind-scoped
`tip_angle` / `pitch_min` clause moved with main's split), the tool page's
kind list and `state.rs`. Main's numbers are on main already; this PR's
were never released. Catalog order: `EDM-E10X6`, then the two thread mills.

**6. Main's Thread tests move to a thread mill.** `thread_mill_rim.rs` (11
tests) and `coarse_rims.rs` built a Tap-kind tool for a Thread operation,
which this PR's tool law refuses (the breaking change). Their tool is now
the 6 mm multi-form mill at pitch 1 and their `OpParams::Thread` literals
state `pitch: None, hand: Right`; no assertion changed, and all of main's
pins (a wider or off-centre declaration, a D-bore, an octagon, a 16-gon, a
floor, a nearby wall convict; matched threads stay clean) hold on the new
kernel.

**Not changed.** No golden moved (the thread goldens regenerate
byte-identical), no snapshot PNG, no process file; `PROCESS_VERSION` is
main's.
