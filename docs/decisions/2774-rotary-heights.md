# #2774 - the rotary kinds' heights are radii about the fitted axis

## #2774 - rotary heights read as radii about the fitted rotary axis

**What was decided.** A rotary operation can read a height as a RADIUS about
the fitted rotation axis, not only as a world-Z plane. The stored form is
`RadialHeight { source, offset }` with `RadialSource` = stock OD, model max
radius, or absolute radius. The fields are optional: `clearance_r`,
`retract_r`, `top_r`, `bottom_r` on Rotary Contour and Rotary Parallel, and
`clearance_r` + `top_r` on Rotary Offset. `None` keeps today's Z-plane law
byte-for-byte; the old `*_z` fields stay. One resolver
(`app::operation::radial_heights`) fits the SAME axis the generators pose
about, and the gate (`radial_gate`) refuses at Create, Edit and ChangeKind
when a clearance or retract radius stands inside the work, or when a stated
band crosses (top below bottom). Refusals name the field and the radius.

**Why.** On a turned bar the tool must keep its distance from the axis all
the way round; a Z plane says nothing there. Keeping the fields optional
means an old project file, and every rotary op that never sets a radius,
posts exactly what it posted before. The refusal at the command follows
"Validate early" (Prime Directive): the generator's own law is stated at
the command, so no silent gouge rapid can reach the post.

**Seeds - corrected during the audit.** New contour and parallel ops seed
`clearance_r = stock OD + 5.0` and `retract_r = stock OD + 10.0`; the band
radii stay `None`. Rotary Offset seeds `clearance_r = stock OD + 5.0` with
`top_r: None`. The WIP commit seeded the band radii from stated numbers
too; that changed roughing output on stock out of the box (a regression we
could not pin as wanted), so the seeds drop the band. The panel's unchecked
rows let a user state a band radius on purpose.

**Project version - the current step is v85.** The ladder has stepped
past the v74 rung this entry first recorded: the fields now ride the
v85 rung (`crates/app/src/project/file.rs`'s ladder and
`apply.rs`'s reader), after v79-v84 landed on main between the held
branch and this PR. The migration is a comment-only no-op all the
same: the new fields are `Option` and serde-defaulted, so a v84 file
reads with every radius `None` - exactly the Z-plane behaviour it
wrote - and steps up unchanged, while a v84 reader refuses a v85 file
rather than drop the stated radii. One consequence: an `OpParams`
change moves the operation stamp, so an old project's cached toolpaths
read STALE after the upgrade and regenerate on the next generate. No
data is lost; the user sees a regenerate, not a wrong program. The
current-format example (`examples/first-continuous-4axis-part.camproj`
/ `.nc`) is advanced to the current version in a test-only commit: the
`.camproj` diff is the version number and the stamp, and the committed
`.nc` posts unchanged from the refreshed project. The compatibility
break is declared with a `BREAKING CHANGE:` footer on that commit.

**Viewport law - rings are ADDITIVE.** The radial rings the Heights tab
states draw into the viewport as extra rings about the fitted axis; they
never replace the Z-plane rectangles. The existing `law_tests.rs` viewport
assertions needed zero changes.

**The band law, re-read as a rho-band (user window test).** The maintainer
tested the built GUI on one rotary parallel operation with the Top radius
and Bottom radius rows ticked: the path still cut where the Z heights cut,
not where the two radii bound it. The band radii are therefore a window on
DISTANCE from the rotary axis (rho) - the annulus between two cylinders -
not two planes along the planning axis. This entry supersedes the dropped
option "Replacing the Z planes when a radius is set" for the two band
fields: the band replaces the Z cut window; Z still orders the stations
along the axis. What was chosen:

- Two shared helpers in `motion::multiaxis::radial` (`band_window`,
  `band_select`, `band_trim`), each reading the TOOL TIP's radius (rho)
  about the planning axis; a band that keeps no cut refuses by name
  (`GenError::Empty`, both radii and the reach printed) - never an empty
  path, never the uncut one. Arcs are linearized first at the pattern
  tolerance. A probe of the generators on the test bars shows why there
  are two: on Rotary Parallel each azimuth pass follows the skin across
  the part's WHOLE radial span, so only a per-pass trim can window it;
  Rotary Contour rings each ride one near-constant radius, so whole-ring
  select is the clean law there.
- Rotary Parallel -> `band_trim`: trims each Feed cut run to the band,
  bisecting each edge at the band cylinder (the tip read is convex along
  a straight segment), and drops every link move - the radial relink
  that follows rebuilds all entries, and each band edge becomes a plunge
  point. The old override of the ladder's `top_z`/`bottom_z` by the band
  radii is REMOVED: the ladder plans the whole work; the trim runs after
  `rotary_ends::clip` and before the radial relink. The laid-frame tip
  read is the kind's retip law: the ball centre stands one tool radius
  up the frame's own z, and the read is that centre's radius minus one
  ball - the ball's closest reach to the axis.
- Rotary Contour -> `band_select`: keeps or drops whole runs (rings) by
  their rho range against the band, and writes a separator Rapid between
  kept runs so the relink reads them apart; a stated band also forces
  the radial relink. The contour tip read is the stored radius minus one
  ball. The axial stations stay the Z ladder's. The select never chops a
  ring at an edge: the finest kept ring can stand a step inside the band.
- Rotary Offset: `bottom_r` floors the develop ladder's descent, so the
  unrolled ladder stops at the Bottom radius cylinder; `band_window`
  still answers a wrong ordering by name.
- The gate: the band ordering refusal is now `top <= bottom` ("stands
  below or at Bottom radius"), and a new refusal catches a top radius at
  or above the Clearance radius ("the rapids would run through the
  band"). Both name the fields and the numbers.

Dropped: keeping the height reading of the band fields (the user's test
shows it cuts outside the cylinders the radii state); and clipping after
the relink (the relink's rapids would already have crossed the band);
and using select for Rotary Parallel (a probe shows every parallel pass
touches every radius the skin reaches, so select is a no-op there).
Known cost of the parallel trim, stated to the maintainer: a pass that
crosses a band cylinder many times along the axis - a twisted or
hexagonal part whose skin moves in and out across the band - yields one
kept stretch per crossing, and the relink makes each crossing a retract,
index and plunge. That "comb" of radial moves is what an annulus window
means for a parallel pass; the maintainer tests the trimmed build in a
window before it ships.
Pinned by `the_parallel_band_cuts_between_two_cylinders`,
`a_parallel_band_that_crosses_no_skin_is_refused_by_name`,
`the_contour_band_windows_its_rings_by_radius` and
`a_contour_band_that_crosses_no_skin_is_refused_by_name` in
`crates/motion/tests/rotary.rs`, and by the rewritten document-level pin
plus `a_band_at_the_clearance_cylinder_is_refused_by_name` and
`a_band_that_crosses_no_skin_is_refused_by_name` in
`crates/app/tests/rotary_verify.rs`.

**Panel row scoping (review fix on this branch).** The four radial slots
share one row helper, and its fields share one id salt. Two enabled slots
collapsed into one egui id: the tree showed ONE From combo carrying the
last drawn row's selection. `radial_row` now scopes its fields with
`ui.push_id(title)` - the same law `height_editor` follows with its tag.
The new structural test `crates/ui/src/egui_app/radial_rows_tests.rs`
pins it: one combo per enabled slot, one echo per enabled slot, and the
checked/unchecked state per slot.

**MCP surface.** No new command was added: the radial fields are new
optional members of the existing operation parameters, and the MCP
create/edit tools round-trip them through the same params JSON (serde
defaults keep old scripts working). If a script verb turns out to need a
dedicated radial entry, that is a `P5` `source:agent` follow-up, not a
blocker for this PR.

**Considered and dropped.**
- Required radial fields (breaking): dropped - it would force a migration
  and re-gate every rotary program for no safety gain.
- Replacing the Z planes when a radius is set: dropped at the time - the
  gate's band law and the generators need both readings (Z still orders
  stations along the axis), and the additive viewport law keeps the two
  pictures honest. (Superseded for the two band fields by the rho-band
  entry above.)
- A second axis fit in the UI echo: dropped - the echo reads the app
  resolver through the `height_memo` cache, so the panel, the rings, the
  gate and the generator answer for one and the same line (frame-cost law,
  #2238; counter test in `crates/ui/src/height_memo_tests.rs`).

**File-size note.** `crates/ui/src/operations.rs` stands at 654 lines; it
was 597 on main. The 400-line law asks for a split along the domain seam;
the echo helpers (the twin pair `height_echo` / `radial_echo` and
`resolve_all`) are the natural seam. The split is deferred to keep this
PR's diff to the feature; it owes a small refactor follow-up.

**Test-environment note.** The `camcraft-ui` lib suite in parallel threads
crashes one run in two on this machine (STATUS_ACCESS_VIOLATION). It does
the same on origin/main, so it is pre-existing and not this branch. The
full lib suite was verified with `--test-threads=1`: 1280 passed, 0 failed,
7 ignored (main serial: 1277 + 1 earlier radial cache test + 2 new radial
row tests).

**Cost to change later.** The stored form is the expression, so moving the
default source set or the seed offsets is a one-line change in
`kinds::seed`. Making the band radii required, or changing what
"stands inside" floors on, touches the gate and its pinning tests in
`crates/app/tests/rotary_verify.rs` only.

**Review rework (PR #4379, nine findings).** The laws the rework fixed,
each pinned by its own test:

- **Tip-to-center conversion.** The contour's radial relink
  (`radial.rs::radial_relink_z`) treats the stated radii as TIP radii and
  places each link's ball center one ball radius outside it - the same
  conversion `rotary_links::radial_relink` already applied to the parallel
  twin - so the posed tip stands at the stated radius, never one ball
  under it.
- **One-sided bands.** `band_window` honors a Bottom radius alone (the
  band runs from the floor cylinder out to the skin's end) and a Top
  radius alone (from the axis out to the ceiling cylinder); no stored
  radius goes unread.
- **Closed band edges.** `each_band_span` keeps a span that starts, ends
  or sits ON a band cylinder with one closed-interval rule for constant
  and tangent spans alike.
- **The judged axis.** The resolver (`radial_context`) reads the radii
  about the SAME axis generation turns about - a stated axis judged
  against the model's faces, the free fit only when nothing is stated -
  and the display cache keys the axis statement, the machine's
  work-turning rotary lines and the work offset origin.
- **Radial input validation.** `OpParams::is_sane` checks every set radial
  field (finite offsets, an absolute radius never under the axis), and
  `radial_gate` surfaces resolution failures and range/ordering refusals
  it used to discard. The same door runs on create, edit, paste and
  retype.
- **Ring basis.** The viewport ring's second perpendicular is `dir × u`
  (an orthonormal basis), never re-perpendicularized through `off_axis`.
- **Zero display.** The panel echo displays the resolved zero for the
  field that permits it (the Bottom floor, the checkbox default).
