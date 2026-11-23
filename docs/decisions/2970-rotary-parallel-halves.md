## #2970 - Rotary Parallel plans the bar in two halves

**Cause (measured).** Rotary Parallel ran the 3-axis swarf raster once,
in a frame that lays the rotary axis down with local z up. That raster
finishes the bar from above only. On a laid bar of radius 3 with a 1 mm
ball, 126 degrees of the turn stayed uncut, centred on the bottom, at a
stepover of 0.8 mm and of 0.25 mm alike. Two stages lose the band:

1. The ring ladder stops at `bottom_z`, read as a tip height in the laid
   frame. With `bottom_z = -R` the last ring has its ball centre 30
   degrees under the horizontal.
2. With `bottom_z` lower, the rings stop 45 degrees under the
   horizontal. `build_sups` flips each facet normal to the sky, so the
   bottom quadrant counts as shallow (raster) ground. The raster takes
   the top tangent and never reaches it, and the rings skip shallow-only
   loops. On the example part (`bottom_z` 9.525 mm under the axis) stage
   2 holds: about 90 degrees, a quarter of the turn.

A third fault hid it: the path dropped the raster and ring notes on the
default path (only the ramp's own notes survived).

**Decided.** The rotary path plans the bar twice. Half A is the laid
frame as before. Half B is the same frame turned half a turn about the
axis. Each half keeps only the cut whose ball centre stands in its own
upper half. Half B is carried back into A's frame by the same rigid half
turn (a centre (x, y, z) of B is (x, -y, -z) of A). The seam: A keeps
centres down to half a ball radius under the axis plane, and B keeps
only centres strictly under A's lowest kept centre on each side (read
over the turned surface, not round the end faces). So no pass is cut
twice, and no seam step is wider than a ring step of the flank. Then the
existing held-end clip, picked-face clip, radial relink and radial pose
run as before. All base notes now reach the path.

Stage 1 is moot after the fix: each half needs centres only down to the
axis plane. Half A keeps its old ladder law (vertical tips not under
Bottom, in the laid frame). Half B is held to Bottom where the radially
posed ball touches the bar: the contact is not under Bottom (within the
tolerance). So a Bottom at the bar's underside (the example: -0.375 in)
cuts the full turn, and a Bottom above the axis keeps the cut to the
upper arc, as before. A path that cuts under the axis carries the note
"Rotary Parallel: the passes go all the way round the bar - ...".

**Maintainer decisions (owner's standing word 2026-11-22: choose the
recommended option while he sleeps), taken by the coordinator.**
1. The example program and project are regenerated with the test's own
   tool (`WRITE_GOLDEN=1`). Dropped: hand edits.
2. `operations::rotary_parallel_generates_radial_axes_and_verifies_clean`
   now reads the Bottom law for each half in its own frame, and it also
   asserts that each 30-degree sector holds a pass. Dropped: keeping the
   old one-frame law, which pinned the band.
3. On a machine whose rotary travel cannot turn the tool under the bar
   (`tilthead.mch`, B -120..120), a full-turn path is refused by the
   machine check: axis, angle and travel are named, and nothing posts.
   The way to the reachable arc is a Bottom above the axis (the existing
   fixture keeps posting clean). Dropped: clipping the cut to the
   machine's reach inside the generator (the generator does not know the
   machine). A follow-up issue asks for an angular-range parameter.

**Review rule: no cut of the single frame is lost (review of PR #3650).**
The plane seam alone dropped cuts on a part that is not round. On a keyed
bar (R 10, keyway walls at z = +-5, 6 mm ball), half A cut the lower
keyway wall with its ball centre 2 mm under the axis plane, under the
seam, and half B cannot see that wall: 52 centre samples that the
single-frame path cut (stations 23.7..36.3 mm, at 347 degrees) were
lost. The rule now:

- A cut of half A under the seam stays where no kept cut of half B
  stands within half a pass spacing at the ball centre ("the same
  place"). The pass spacing is the stepover, or the scallop step
  `sqrt(8 r tol)`, the larger.
- A cut of half B over the seam (a wall that faces down, which A cannot
  see) is added where no kept cut of A stands within half a pass
  spacing.
- The threshold for "the same place" is half a pass spacing. So each
  place is cut once.
- Round bars: the output of the first head stays byte-identical for the
  fixtures that were run (`rotary_coverage`, the example program, the
  microARC wrap golden). This is a measured result, not a law. On those
  bars every A cut under the seam has a B sample within half a pass
  spacing. On another bar an A cut under the seam can stay when no B
  sample is that close. Such an extra cut is one that the single-frame
  path makes too.
- A reach reading names the places no pass reaches (follow-up #3658:
  its cost on a dense mesh, the document unit, a frame the user can
  see). It uses a fixed
  grid of at most 48 stations and 72 angles. A ray from the axis finds
  the outermost surface. A place under Bottom is skipped. A place where
  a ball cannot fit (a corner) is skipped. A place with no ball centre
  within one pass spacing gets a note with its arc (degrees about the
  axis, from the top of the laid bar toward its +y side) and its
  stations (mm along the axis). Only a path with no such place says
  "all the way round". The reading sees the outermost surface only: a
  wall hidden behind other material along the radial is not read.
- The test reference is the generator as it stood before the two halves
  (`rotary/single_frame_tests.rs`, test only). On the keyed bar the new
  path loses 0 samples and cuts no place twice.
- An eccentric collar (a round section off the axis) was planned as a
  second fixture. The axis fit refuses every variant that was tried
  ("every axis fits this model equally"), so no such part reaches
  Rotary Parallel. That case has no fixture; it is a gap in the
  evidence.
- If half B fails with an error other than `Empty` (a refusal of its
  ladder), the whole operation is refused, also where the single-frame
  path would post. The refusal is visible and names its reason.

**Why.** A ball sphere clear of the model stays clear under a rigid
rotation, and the radial pose keeps every ball centre. So each half is
as safe as today's single run. The change stays in
`crates/motion/src/multiaxis/rotary.rs` and its new child
`rotary/halves.rs`. No shared finishing helper changes.

**Dropped.**
- The reporter's plan: classify with the unflipped normal in
  `build_sups`. That function has 15+ callers (every 3D finishing
  pattern), and the split would then depend on the mesh winding: an
  inward-wound mesh would mark its top steep. Not allowed here.
- A true radial drop-cutter for Rotary Parallel (passes spaced in angle,
  not in local y). That is a new algorithm (effort:max). The two halves
  reuse the proved 3-axis ladder.
- More than two frames (quarters). Two halves are the least that covers
  the turn; more frames add seams.

**Cost to change later.** A radial drop-cutter would replace
`rotary/halves.rs` and the second `swarf` call; nothing else depends on
them. The planning time of Rotary Parallel is about twice what it was.
