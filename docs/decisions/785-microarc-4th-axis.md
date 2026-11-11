## #785 - Tormach microARC 4: A indexing in the machine file and the PathPilot post

Sources (behaviour only): Tormach TD10749 *Owner's Guide: microARC 4* (1223A),
TD10552 *4th Axis for M/MX* (0626A), TD10374 *4th Axis, PCNC 1100 / PCNC 770*
(0626A); the LinuxCNC G-code reference (rotary axes, G93/G94); Tormach's
PathPilot post-processor guidelines. The issue comment of 2026-11-10 holds
the three answers in full.

**Indexing only; simultaneous 4-axis refused by name.**
- **Decided:** A moves alone, as `G0 A..`, after `G53 G0 Z<safe>`, at a
  section's first motion. An A that changes inside an operation is
  refused with "simultaneous 4-axis motion ... not confirmed on PathPilot".
- **Why:** the sources leave both open questions unsettled. TD10749 says
  nothing about rollover (wrapped A) or feed, and LinuxCNC leaves rollover
  to each machine's configuration, which PathPilot doesn't publish. An
  index lands on the same angle under either reading, with the tool up.
  A wrapped cut would not.
- **Dropped:** posting wrapped moves under G93 or G94 now. That waits for
  the PathPilot Hub and the user's hardware, in #796.

**The A word is read through the tool axis, not the letters.**
- **Decided:** the post computes the tool axis from the stream's A/B/C
  (the pose law, Rz(C)·Ry(B)·Rx(A)·Z). It accepts any axis with no X
  component and takes a = atan2(sin, cos) in -180..180. C must be 0.
- **Why:** the 3+2 tilt stores canonical letters (A in -90..90), so the
  face underneath arrives as A0 B180. An A about X reaches it as A180,
  and refusing it as "a B" would be wrong.
- **Limit found:** the app's machine check (`machine::check`) judges the
  letters themselves and does no IK. A0 B180 is therefore still refused
  there ("states no B axis") before the post is reached. The test part
  uses the opposite pair A-90 / A90 instead of 0 / 180, and the gap is
  filed as #797.

**The indexed frame.**
- **Decided:** each tilted tip is posted relative to the work offset,
  turned by -a about X, so the tool stands up Z.
- **Why:** this needs WCS Y0 Z0 on the A centerline. Nothing in the
  stream can check that, so it is an operator note at the first index and
  a README setup step.
- **Dropped:** a machine-file centerline offset. The setup states the
  offset; the machine file doesn't.

**Direction sense = the sign of the rotary's `axis` vector.**
- **Decided:** `[1, 0, 0]` follows RS-274 / ISO 841: a + A turns the tool
  right-handed about +X relative to the part, so the stream's angle posts
  as itself. `[-1, 0, 0]` posts it negated.
- **Why:** the parser refuses a rotary `direction: -1` ("a reversed rotary
  is not supported"), and the vector is already the rotary's stated
  geometry.
- **Not confirmed:** the microARC 4's real sense isn't in TD10749. The
  test kit's first step checks it with an MDI `G0 A90`.

**No clamp code.**
- **Decided:** the post writes no clamp code and adds an operator note.
- **Why:** none of the three guides names an M-code. The microARC 4 (a
  harmonic drive) has no clamp at all. The older tables clamp by hand and
  must be released to rotate.

**Travel -360..360, rates unstated.**
- **Decided:** the machine file states travel -360..360 and no rates.
- **Why:** TD10749 states neither a travel stop nor a rate. The tool angle
  is always in -180..180, so the travel never binds. A machine stating less
  travel is refused by name. The unstated-limit law keeps the rates absent
  rather than invented (TD10374's 4 RPM is the older worm tables, not the
  microARC).

**Upright sections state A0 on a 4th-axis machine.**
- **Decided:** a program for a machine with an A always writes `G0 A..` at
  its first motion, `A0` for an upright job. A machine without an A writes
  no A at all, so every existing golden is byte-identical.
- **Why:** the control's A after the last run is unknown.

**One file, the 770MX + microARC 4.**
- **Decided:** a separate machine file rather than an option on the
  existing ones.
- **Why:** the schema has no option concept. Other mills follow the same
  pattern by copying their file and adding the one rotary part.

**Parser fix found on the way.**
- **What:** `parse_mch`'s head search stopped at the first part with no
  children (`get("parts")?`). A rotary leaf listed before the head lost the
  spindle range, belt ranges, rigid tapping and coolant, silently.
- **Fix:** the search now passes leaves over. It is pinned by
  `the_microarc_machine_states_an_a_about_x`.

**A is restated after every G30 and at every section (review of PR #798).**
- **Decided:** the post forgets the A it holds (`aNow = null`) when it
  writes `G30`, and again at every section end. Every section therefore
  opens with `G53 G0 Z<safe>` / `G0 A..`, even on the same face as the
  section before, with or without a tool change between.
- **Why:** a `G30` with no axis words moves every axis to the stored G30
  position (LinuxCNC, params 5181-5189), A included, and the app re-issues
  the tool (so `G30 / T.. M6`) before every operation. Two operations on
  the same face posted the second with no `G0 A..`, cutting wherever G30
  left the table. The same-tool continuation (no G30) moves nothing in A,
  but a section boundary is where an operator may stop (M0/M1, a probe
  check) and jog, so the cheaper rule - keep A across same-tool sections -
  was dropped: one extra Z lift is the whole cost.
- **Dropped:** changing the `G30` itself so it leaves A alone (axis words
  on a G30 add an intermediate-point move and change PathPilot's own
  tool-change convention for every Tormach program), and a swept-radius check before an index (filed as a
  follow-up; the kit README now asks the operator to confirm it).
