## #419 - threading: in-pass chip-break dips inside a continuous G32 chain

- **The citation came first, as the issue required.** The primary Fanuc
  source is the *FANUC Series 0i-TD/0i Mate-TD USER'S MANUAL (For Lathe
  System)*, B-64304EN-1/01, Part II. I read it from a public mirror of
  Fanuc's PDF:
  https://www.slideshare.net/slideshow/fanuc-0itd-users-manualpdf/259106078.
  The sections it gives us:
  - §3.4 "Continuous Threading" (p. 34): consecutive threading blocks
    keep "the synchronism with the spindle" at "the joint between
    blocks", and threads whose "lead and shape change midway" may be
    cut that way.
  - §3.2 "Constant Lead Threading (G32)", WARNING 6 (p. 32): a threading
    block that follows a threading block starts "without waiting for
    detection of the one-spindle-rotation signal".
  - §3.2, pp. 29-31: G32 cuts tapers (worked example `G32 X41.0 Z29.0
    F3.5`). Up to a 45° taper the lead F is taken along Z, and past 45°
    along X.

  The Haas lathe G32 reference (sync "stays in effect for each
  subsequent G32 line") corroborates this. LinuxCNC G33 is still
  unconfirmed: its reference page does not say whether a chained G33
  waits for the index again. The citation sits in the header of
  `fanuc-lathe.cps`, beside the new flag.
- **Chosen: a variable-radius thread half-turn as the new CL segment.** A
  dip is an XY half-turn about the spindle axis whose end radius differs
  from its start radius. Its axial travel is still exactly half a lead,
  so the CL needs no new move kind. The arc struct already allows
  unequal end radii: chords take the larger radius, which is the
  shallower side and so conservative, and the timeline length is
  unchanged. Dropped: a new `MoveKind` or a per-move "synchronized"
  field. Either would touch every CL consumer (text, dump, project,
  sim, verify) and carry the same two endpoints anyway.
- **What marks a dip: the thread record plus a section PARAM.**
  `ThreadCycle.chip_break_dips` is written as `DIP 1` in the stream,
  property `ChipBreakDips`, and as a project cache key only when set
  (the #417 / #418 precedent). Whenever a path's record carries that
  flag, the dump also writes `PARAM ChipBreakDips 1.000000`. The engine
  refuses that PARAM by name for any script that does not declare
  `continuousThreading = true`, the same way `liveTooling` works. If
  dips are on and the record cannot be attached, the kernel refuses, so
  a dip is never laid unmarked. Dropped: having the engine spot dips by
  geometry (unequal arc radii). Fitted milling arcs can differ in
  radius by up to their tolerance, so that check would refuse good mill
  programs.
- **Only fanuc-lathe declares the flag.** Each other post needs its own
  manual citation first. The mill posts never see dips, because the
  engine refuses the section.
- **Dips are radial, not along the flank vector.** The issue says "along
  the infeed vector", and for radial infeed that is the same thing. For
  flank or alternating infeed, a radial lift keeps the pass's own
  station, and a V on the same station with a higher tip lies inside
  the V the pass already cuts. So a dip can never take more than the
  pass, whatever the infeed. A lift along the flank would shift the
  station, so the dip's Z travel would no longer be half a lead and F
  would no longer be the lead. Dropped for that reason.
- **Parameters (`ThreadInfeed.dips`):**
  - interval: axial, rounded up to half-turns, at least two leads, and
    at least one whole lead longer than the dip. 0 means off, which is
    the default.
  - height: radial, at least 0.01 mm so a post can tell it from the
    micron grid, and at most every dipped pass's depth of cut.
  - length: axial, out-ramp to in-ramp, rounded up to half-turns. It is
    never shorter than its two ramps; 0 gives the shortest dip.

  Each ramp takes the fewest half-turns that keep the diameter change
  0.01 mm under the axial travel. That stays under 45° whether the
  control measures the taper in radius or in diameter, and the margin
  keeps the post's micron-rounded check on the same side as the
  kernel's. Dips start at one interval, then every interval after that.
  A dip must end before the last lead, and the first lead is always
  clear, because the interval is at least two leads.
- **Where dips may stand:** roughing passes only. Not on the finishing
  pass or the spring passes at the root, not on the repeat of a depth
  (#418: a repeat exists to clean that depth whole), and not in a
  chain's first or last lead.
- **Refusals by name:**
  - At the command door and the kernel: a dip higher than the first
    pass's depth, too small, or not a length.
  - At the kernel: higher than a later roughing pass's own depth (the
    message names the shallowest), an interval that leaves no whole
    lead between dips, a thread too short for any dip outside its first
    and last lead, and a thread with no roughing pass (a one-pass
    thread).
  - At the post: G76 chosen ("chip-break dips are not a G76 cycle"), and
    a post without the flag (engine).
  - In `fanuc-lathe.cps`, as a backstop per segment: a dip that does not
    continue a G32 chain (a fresh G32 would wait for the index and
    restart mid-thread), one steeper than 45°, and one deeper than the
    chain's plunge.
- **Refused versus silently skipped.** A thread too short for a dip, or
  one with no roughing pass, is refused rather than laid with no dips.
  The user asked for chip breaking, and a silent no-op is a wrong
  answer they would only find at the machine.
- **The verdict judges the dip geometry.** `thread_warnings` now also
  re-reads each chain, meaning a plunge followed by its unbroken
  half-turns. It convicts, with a gouge at the move:
  - a half-turn whose axial advance differs from the chain's first
    (the phase law);
  - any half-turn standing deeper than the plunge;
  - a variable-radius half-turn that is steeper than 45°, inside the
    first or last lead, or on the deepest chain (the finishing or
    spring passes).

  A gouge holds the post. The existing wave judge reads a dip
  half-turn at its deeper end, and that end is never deeper than the
  pass. With the new laws switched off, the first-lead conviction test
  fails, so the convictions come from these laws and not from the wave
  judge (checked by hand while writing the PR).
- **Simulation is unchanged.** The lathe stock simulation refuses threads
  by name, as it did before. Dipped threads are covered by the same
  refusal, so nothing new is shown wrongly.
- **Defaults are byte-identical.** With dips off:
  - each half-turn's radius is the pass's tip, copied bit for bit;
  - there is no stamp byte (`0xD6` plus three numbers only when dips are
    on);
  - the `dips` key is written only when not the default, and the record
    key only when set;
  - there is no `DIP` word and no PARAM.

  No existing golden moved. The new golden is
  `nc/fanuc-lathe/thread_g32_dips.nc`.
- **File sizes:** `post/engine.rs` (439 lines before this PR) and
  `post/mod.rs` (523 before) were already over the 400-line target, and
  this PR adds 4 and 6 lines to them. The refusal text lives in
  `cnc_thread.rs` to keep the growth small. Splitting the engine is its
  own refactor.
- **Cost to change later:**
  - A LinuxCNC or other post sets `continuousThreading = true` once it
    has its own citation.
  - A flank-vector lift would need F recomputed per segment.
  - A dip count or placement other than "every interval" changes only
    `thread_dips::radii`.
