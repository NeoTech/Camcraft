## #3244 - Mikron WF3 DCM (LinuxCNC): no rigid tapping in the shipped file

**Decided.** The shipped `mikron-wf3-dcm-linuxcnc.mch` states
`spindle.rigid_tapping: false`. The LinuxCNC post refuses `G33.1` by name
on this machine, and the program gate (`tap_gate`, #786) refuses a rigid
tap cycle by name. A floating-holder tap still posts as `G84`.

**Why.** The review of PR #3982 read the published HAL of the retrofit
(`paulsvc/linuxcnc-mill2-config`, `mill2.hal` lines 255-281 and
`custom.hal` lines 61-64 at `ea22af1a`). The spindle index signal goes to
motion and to the spindle PID only. It does not go to the encoder's
index-enable pin. `G33.1` waits for the encoder to clear the index request
before the synchronized move starts. On this HAL nothing is shown to clear
that request, so the tap is not known to be safe. A refusal is safe; a wrong `G33.1` is
not (Prime Directive).

**Dropped.**
- Keep `rigid_tapping: true` (the owner asked for rigid tapping on
  #3244). Dropped: no evidence in the published configuration that the
  index path supports `G33.1`.
- Omit the key (unstated). Dropped: the post would still refuse `G33.1`,
  but the program gate would say nothing. A stated `false` gives the
  refusal earlier and by name.

**Cost to change.** One line in the machine file and two test asserts.
When the machine owner connects the index-enable pin, the file can state
`true` again, with the HAL lines named in the PR.

## #3244 - Mikron WF3 DCM (LinuxCNC): keep `tool_change: manual` under the manual tool change law

**Decided.** The machine file keeps `tooling.tool_change: "manual"`. After
the merge of main, the manual tool change law (#4085,
`camcraft_post::post::manual_change`) refuses each LinuxCNC program for
this machine by name, because `linuxcnc.cps` does not declare
`manualToolChange`. The pinning test asserts that refusal. A follow-up
issue owns LinuxCNC support for a manual tool change.

**Why.** The published `mill2.hal` (lines 339-348) connects the M6 request
to `hal_manualtoolchange`, so this control stops at M6. But the generic
LinuxCNC post serves other configurations, and in some of them the
tool-change request loops back with no stop. A declaration in the post is
a post-logic change under the safety floor. It is not a review fix of a
machine-file PR.

**Dropped.**
- Remove `tool_change` from the file (unstated). Dropped: it hides a true
  fact to pass a safety law.
- Declare `manualToolChange = ["manual"]` in `linuxcnc.cps`. Dropped here:
  it changes the post for every LinuxCNC machine and needs its own review.

**Cost to change.** When the follow-up lands, the Mikron test changes from
the refusal to a posted program with the stop.
