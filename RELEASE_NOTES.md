# CamCraft 0.18.0 (clean-room implementation)

Released 2026-11-12.

Breaking: none. The project file format stays at v58, so files saved by
0.17.0 open unchanged. **Posted
G-code does change for some machines**, so check the first program you post on
each: the Fanuc, Haas, Mazak and Doosan DNM posts now write `G21` and the
Siemens 840D post writes `G710` before any motion (#901); every mill post
writes an explicit spindle stop after a probe's tool change (#574); lathe
posts reset constant surface speed at the end of a section (#772); and the
post gate judges arcs and tool bodies more strictly in places (below), so a
program that posted before may now be refused by name.

A minor release headlined by **CamCraft in a browser** (a first web runner),
**C-axis unwind** for continuous live tooling, **default holders on machine
files**, **a large fixture and holder library** (plates, mod vises, 5-axis
vises, accessories, BT/CAT/ER/TTS holders), **theme as a user preference**,
a Trochoidal slot operation, Tormach microARC 4 and plasma posts, and
two new lathe and mill posts. Thanks to mkinney, vilts, tbrasselle, ahmetv10,
JSwetz and seelesturm for their contributions.

## This cut
The UI snapshot goldens were regenerated for this cut and accepted by the
maintainer without the usual manual look at the diff images (maintainer's
decision, 2026-11-12).

## New
- **Runs in a browser** (#31, #841, #847): the shell builds for wasm32 and runs
  on a web canvas with an awaited GPU start-up and a page built by
  `web/build-app-wasm.sh`. A first rung only, and **not part of the release
  download yet**: the release workflow does not build or attach it, only the
  manual CI workflow compiles it. The file picker is still owed.
- **C-axis unwind** (#33): continuous C-axis live tooling now unwinds C at
  clear links against the machine's stated C travel. Rotary Contour unwinds;
  Rotary Parallel and Rotary Offset refuse by name where C would need a
  rewind. Each rewind is an orbit the verifier and the post both see, and the
  lathe post caps every C block's feed by the machine's stated linear feed and
  C rate.
- **Machine default holders** (#828): a machine file can name a spindle mount
  and a default library holder (`tooling.spindle_mount`,
  `tooling.default_holder`; refused by name when unknown or mismatched). A new
  tool added in the GUI or over MCP starts with that holder in the same undo
  step. The Tormach MX mills default to BT30 ER20, the PCNC/M mills to TTS ER20.
- **Theme is your preference** (#877, vilts): the theme and colour-vision mode
  survive restarts, and New, Open and project files no longer change them. The
  light theme now has a light viewport with toolpath, overlay, view cube, hover
  and title-bar colours that read on it (#698, vilts).
- **Holder library** (#792, #793, #794, #799): BT30, BT40 and CAT40 taper
  holders in ER collet chuck and Weldon versions; standalone ER11-ER40 collet
  chucks and BT40 ER40 from one ER table, with the flange flare and Weldon
  clamp screw enveloped; Tormach TTS holders (ER20, ER16 and set-screw) with
  the collar face as the gauge line. MCP gains `set_tool_holder` and
  `list_holder_library`. A vendor tool STEP can be aligned by its ISO 13399 tool
  coordinate systems (#370, JSwetz).
- **Tool editor and simulation** (#93, #743): draw a milling tool's cutter,
  neck and shank as a spun outline (Profile tab, validated against the stated
  diameter and lengths, undoable, over MCP as `set_tool_cutter_profile` /
  `get_tool_cutter_profile`). The simulation draws the tool and holder as the
  same revolved body the collision laws judge.
- **Fixture library** (#787, #788, #789, #819, #820): parametric milling vises
  (3-8 in, 100-200 mm); 5-axis self-centering vises (52, 75, 80, 125 mm)
  centred on the stock; accessories (parallels, 1-2-3 and step blocks, strap
  clamps, soft jaws); fixture plates (12 x 12 in, 300/400 mm, 770MX table) with
  any fixture placed at a named grid hole or dowel pair; mod vises (fixed and
  adjustable jaw blocks on a plate, MCP `add_mod_vise_pair`). Each is placed in
  one undoable step and judged by the fixture mesh laws. Links and retracts
  that would strike a fixture are raised over the steel instead of only being
  reported; a clamp taller than the machine's Z stroke refuses generation by
  name (#339).
- **Trochoidal slot operation** (#220): an engagement-controlled trochoid along
  a typed centre line, with the measured contact angle held under a limit by
  adapting the step, microlift or retract returns, and named refusals. It also
  refuses by name a forward step past the loop cap, a microlift above the
  clearance plane and a bore past the helix-turn cap (#774).
- **Pocket and slot detection** (#511): the Model page detects pockets and
  slots and creates their operations with one click.
- **Posts and machines**: Tormach microARC 4 and PathPilot A indexing between
  operations on the new 770MX + microARC 4 machine (#785; simultaneous 4-axis,
  unreachable tilts and A past travel are refused by name); a PathPilot plasma
  post for the Tormach 1300PL (#671); a PathPilot lathe post (#612); a Doosan
  DNM + Fanuc post aligned with a real sample program (#614, JSwetz);
  documented Heidenhain drilling and tapping cycles with explicit floating
  holder intent (#51, seelesturm).
- **Plasma** (#672, #801): the through-cut laws judge every Plasma Profile at
  the post gate (cut order, cut crossing, a malformed record block the post by
  name); a slot or keyway narrower than the kerf is refused at generation; the
  laws warn on a rapid over a freed hole slug.
- **Refused generations show their moves** (#261): the moves built before a
  refusal are drawn as a dashed ghost in the refusal colour (display only;
  never cached, simulated or posted), and the script/MCP error names its move
  count.
- **UI polish** (ahmetv10): larger ribbon cells with readable labels,
  capitalised tab titles, no internal status text, an empty-viewport hint and a
  Z=0 ground grid. The Machines pane's kind dropdown filters the list (#816).
  The starter tool catalog is now a data file, `assets/tools/catalog.json`
  (#741).
- **Documentation** (#688, #291, tbrasselle): a Zola user-guide and developer
  site published to GitHub Pages at each release tag; a first-part example
  project with tutorial images.

## Safety and correctness
- **Posted programs**: the mm unit word (`G21`, `G710` on the 840D) is written
  before any motion on the Fanuc, Haas, Mazak, Doosan DNM and Sinumerik posts,
  so a control left in inch never reads an mm program as inches (#901). Every
  mill post writes an explicit spindle stop (`M5`; `MOTOFF` on Mazak) after a
  probe's tool change and length offset (#574). Lathe posts (Fanuc, PathPilot)
  no longer carry a section's G96 into a later section that states none (#772).
  The Carvera post names a per-operation smoothing ask with a one-time
  "SMOOTHING NOT WRITTEN" note instead of dropping it (#757).
- **STEP chord cap and arc fitting** (#689): a bore at a STEP hole's own size
  posts clean. STEP meshes are capped at 0.005 mm chord error, arc fitting
  keeps vertices within half the tolerance, closed rings fit whole, and the
  gouge law reads arcs at a quarter of its tolerance. A nominal arc round a
  convex wall now passes; an arc bowing past the floor between its old chord
  joints is now caught.
- **Reach and tool-body laws** (#903, #835): reach laws (c)/(c3) no longer
  excuse a leaning shank or holder standing in the part as cutter kerf. A tool
  body (imported or drawn) is judged by the same band stack the simulation
  draws, so a stepped holder whose flange stands clear is no longer warned;
  laws that still use the enclosing cylinder say so; a mid-turn swing through
  the part is no longer hidden behind a stock shank warning on the same move.
- **STEP files with empty parameter lists import** (#887, tbrasselle): files
  exported by HOOPS Exchange, anything with an empty `()`, no longer fail as
  corrupt.
- **4th-axis A tables** (#797, #804, #813): the machine check reads an A-about-X
  table's pose through the tool axis (the underside A0 B180 posts as A180 on the
  770MX + microARC 4). The program gate refuses an A index when the retracted
  tool tip cannot clear the swept stock, part and turning fixtures by 5 mm. The
  swing check also judges a work-turning A whose machine file states no axis
  vector (an A is about X), and refuses one with neither a vector nor an
  RS-274 letter. A machine whose work-turning A states no `nose_to_axis` has
  every operation posted on it refused by name at the program gate until the
  machine file states it.
- **Adaptive** (#850): the growth planner refuses by name a plan that has
  worked heavy ring geometry over a long walk, so an islanded area it cannot
  finish falls back to the named Offset rings in about 22 s instead of about a
  minute. Plans that finish today are unchanged.
- **Gouge law (b) in Boundary mode** (#905): an arc move in contour, trace and
  face is checked along its whole length against wall corners, not only at
  sample points, so a gouge between two samples is caught. Arcs in the XZ and
  YZ planes are still read at their chord joints (#946).
- **Picking over MCP** (#692): `locate_part` refuses a mate id no click could
  pick, like the other pick tools.

## Fixed
- **Fixtures**: a vise or accessory key this build cannot read is dropped with
  a load note and the project still opens; a strap clamp's stud must be
  narrower than its strap; the parallels fit marks the height it sets (#812).
  Dropped plate and mod-vise keys show in the load notes, a located plate's
  riders are clash-checked, a plate whose mesh changes drops its grid, and
  staggered plates with a clamp ear or chamfers are no longer refused as
  turned (#826).
- **Setup page** (#866, #872, #873): it names the turn that stops the stock
  carry and says what to do; the slow-frame log no longer blames time spent in
  a file dialog.
- **Plasma**: the profile refuses a zero-width slit in a picked contour instead
  of dropping it (#801).
- **Build**: the main branch compiles again after a Trochoid/Stock merge slip
  (#844).

## Changed
- The UI says "work offset (G54-G59)" plainly, and refusals say what a setup
  owns in plain words instead of "furniture" (#876, #878).
- The dev build profile is lighter and the gate can use a compile cache
  (#900); the release build is untouched.

## Known issues
- **#934 (P0)**: the Sinumerik post writes work offsets 5 and 6 as G58 / G59,
  which on the 840D are programmable offsets, not settable frames. Check
  offsets 5 and 6 by hand on that post.
- **#946 (P1)**: gouge law (b) in Boundary mode still reads arcs in the XZ and
  YZ planes only at their chord joints, so a sharp wall corner between two
  joints is not judged there.
- **#864 (P1)**: an over-budget STEP mesh is imported at reduced fidelity with only a
  log line; gouge verdicts on it can pass a real gouge on a convex wall.
- **#906, #885 (P1)**: a machine that states a C travel but no C rate gets an
  uncapped rewind feed, and a combined linear + C move on the Fanuc and Doosan
  posts is posted at the mill feed without a cap from the C rate.
- **#938 (P1)**: a tool tilted by a rounding error (1e-10 degrees) is judged as
  tilted, and a holder wider than the cutter then reports a false collision.

## Process changes
Contributor and agent process only; no effect on the program.
- **PROCESS_VERSION 11** (#765): the xtask test scanner ignores braces in
  strings, chars and comments.
- **12** (#846): the `classify-effort` skill, one way to choose an issue's
  `effort:*` label.
- **13** (#855): the docs-picture goldens move to the release snapshot lane
  (xtask plan gains a step; a fork re-syncs xtask).
- **14** (#881): a contributor's claim must be live; stale after 48 hours with
  no open PR, branch commit or comment, and an ended claim is never mirrored
  back.
- **15** (#913): `cargo xtask gate` uses sccache when present; a lighter dev
  profile; agents must not export `CARGO_TARGET_DIR`.
- **16** (#944): agents prefer `cargo xtask gate`, set `RUSTC_WRAPPER=sccache`
  only when it is installed, and the release build runs without a compiler
  wrapper.
