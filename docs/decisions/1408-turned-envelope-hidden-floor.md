## #1408 - a turning nose path's hidden rest is read as the turning work sees it

**What was decided.** The envelope verdict (law f) of a turning op cut by a
nose (Facing, the Profile kinds, Chamfer, Taper, Multi-Pass, Boring - every
`Family::Turn` op whose tool states a nose radius) now hands the scan the
spindle its kernel fitted (`VerifyContext::lathe_axis`, from
`turning::Frame::about`, the same reading as the lathe sim's). With it, only
the hidden-rest half of the scan changes, by four clauses
(`stock_sim::scan_turned`):

1. **The work turns.** A hidden top is read on its orbit about the spindle:
   the top itself and its two turns into the path's meridian plane. Each
   sample is a point of the orbit, so the reading is at or above the true
   orbit distance - an over-read rest, never a hidden one.
2. **The band.** A hidden top farther along the spindle than every nose
   centre by more than the nose, the scan floor and the arcs' bulge is out
   of reach of every ball of the pass: excused (the groove under a Facing).
3. **The crease.** A hidden top the nose cannot touch at all - the ridge
   law's own accessibility test (`ridge::Access::reachable`), the tangent
   ball along the top's normal cutting a neighbouring face - is excused (a
   groove floor's inner corner under a 0.8 nose: 0.33 mm, `r(sqrt2 - 1)`).
4. **The leave.** A rough's `stock_to_leave` is its promise: a hidden top up
   to the leave outside the nose is not rest, and the crease test reads the
   ball grown by the leave (a 2 mm groove a 0.8 nose plus 0.25 leave cannot
   enter is the finish's).

The gouge half, the ridge (visible) half, and every mill path are
untouched: without a spindle the scan is bit for bit the old one (pinned).

**Why.** The issue's three Facing refusals were the far side of the groove
floor read across the axis from a scan that never turns the work, and then
(once turned) the near side, 14 mm below the faced band. The Profile half
(austin62jake's comment) was the same far-side reading; once turned, two
smaller readings stood - the inner corner a nose cannot reach, and the
rough's own leave on the groove floor. Neither is rest the op owes: the
visible half already lets both stand (its band never reaches them), and
only Z-occlusion sent them to the hidden half, an occlusion that means
nothing to a tool approaching radially.

**Considered and dropped.**
- *Switch Facing/Profile to `Mode::Revolve`* (the Groove rows' pattern, as
  triage suggested). The Revolve verdict is the BLADE's boxcar meridian
  scan; the registry comment on Facing already says the nose kinds keep law
  (f). Moving them would change every nose kind's verdict law, not just the
  hidden half that misread. What we took from the Groove rows is their
  premise: a band op is judged on its band.
- *Rotate about world Z.* The turning kernels fit their axis; a bar not on
  the world axis would be turned about the wrong line and could under-read.
  The fitted axis is passed instead; a source no axis fits gets `None` and
  the old (refusing) reading.
- *Drop the hidden half for turning paths.* It is the only part of law (f)
  whose rest can exceed the gate (the visible band caps at `tol`), so it is
  what convicts a pass that bridged a groove inside its own band. Kept.
- *Read the hidden top's distance in the meridian against every segment.*
  Exact, but linear in segments per top and outside the `SegGrid`'s work
  budget; the three orbit samples are exact for an in-plane path and never
  under-read for any other.

**Cost to change later.** The clauses live in one 140-line module and are
switched by one context field; a later meridian-exact reading replaces
`TurnedBand::turned` without touching callers.

**Left open.** Whether a profile insert's real geometry (trailing angle,
not only its nose circle) can enter a given groove is not read: a groove
the nose fits but the insert's back cannot is still judged as reachable
(errs refusing).
