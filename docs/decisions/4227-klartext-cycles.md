## #4227 - Heidenhain Klartext: cycles 200, 206, 207 and the cycle 9 dwell

**Decided.** The Klartext post (`assets/posts/heidenhain-klartext.cps`)
writes the drilling cycles of the iTNC 530 NC 340490-03 (the only profile
that the post accepts), at the parameter set of the resolver's
`drillC200` level `q211` (no Q395):

| Stream cycle | Posted |
|---|---|
| DRILL, SPOT | `CYCL DEF 200 DRILLING` with Q202 = the whole depth, Q210 = 0, Q204 = Q200, the dwell in Q211, then `CYCL CALL` |
| PECK, DEEP | the checked chain as `L` blocks, as #4224 decided for DIN/ISO; no cycle 205 or 203 |
| TAP (rigid) | `CYCL DEF 207 RIGID TAPPING NEW`: Q200, Q201, Q239 (pitch = feed / rpm), Q203, Q204 = Q200 |
| TAP (floating holder) | `CYCL DEF 206 TAPPING NEW`: Q200, Q201, Q206, Q211, Q203, Q204 = Q200 |
| a dwell of PECK or DEEP | `CYCL DEF 9.0 DWELL TIME` / `CYCL DEF 9.1 DWELL <s>` after the feed to the depth |

The cycle is defined again only when its words change (one definition
for a row of equal holes). Each hole gets one `CYCL CALL`, with the tool
at R over the hole: the stream rapids there first, and the post refuses a
cycle that starts elsewhere.

**Why: the native cycle must do the checked motion.** The simulation and
the gouge check read the stream's hole chain. The post writes a native
cycle only where the cycle's own motion (iTNC 530 Cycle Programming, NC
software 340 49x) is the chain, and it compares each replaced move with
that motion. A chain that differs is refused by name ("the hole chain is
not the motion of cycle 200").
- Cycle 200 with Q202 = the whole depth: one feed to the depth at Q206,
  the dwell Q211, FMAX to Q200 above the surface (= R). Q204 = Q200, so
  the move to the second set-up clearance is no move. The DRILL / SPOT
  chain is: feed to the bottom, rapid to R.
- Cycles 206 and 207: feed to the depth, the spindle reverses, feed back
  to Q200 (= R); Q204 = Q200 again. The TAP chain is: feed to the bottom,
  feed to R, both at the pitch feed. Cycle 206 feeds out at Q206; the
  207 retract is synchronized with the spindle at the same pitch. The
  path is the same vertical line in the same hole, so a difference in the
  retract speed cannot move the tool to another position.
- PECK and DEEP: #4224 applies without change. The native peck
  re-approach is not provably the chain, and the 340 49x cycle 205 has no
  Q208 (its retract rate is not stated). The chain is the checked motion
  by construction.
- The dwell: the engine sends a dwell only for a cycle. For DRILL, SPOT
  and the floating tap it is Q211 of the cycle. For PECK and DEEP it is
  cycle 9, which the resolver's `dwell` flag confirms (`cycle9`) for this
  profile. A dwell that the cycle did not write is refused.

**Refusals that stay (each pinned in
`crates/post/tests/klartext/cycle_tests.rs`):**
- BORE, BACKBORE, REAM and FINEBORE (no cycle of this issue).
- A tapping cycle with a tool that is not a tap; a tap or a drill that
  cuts outside its drilling cycle (a drill feeds only along the hole axis).
- A cycle that does not start at R over its hole; a chain whose depth or
  moves differ from the cycle.
- A rigid tap with a dwell (cycle 207 has none); a floating tap dwell
  above 0.5 s (as the DIN/ISO post); a rigid tap on a machine that states
  `rigid_tapping: false`.
- A cycle 200 level other than `q211`, and a dwell form other than cycle 9
  (not reachable while the post accepts only NC 340490-03).

**The Picomax machine file states no `rigid_tapping`.** The post writes
cycle 207 when the machine does not say `false`, the same as the DIN/ISO
post's G207 on this machine. If the spindle cannot tap rigidly, set
`rigid_tapping: false` in the machine file: the app then refuses the tap
path, and the post refuses cycle 207 by name.

**Dropped.**
- `L X.. Y.. Z.. R0 FMAX M99` as the call. The tool is already at R over
  the hole, so `CYCL CALL` alone adds no positioning block.
- A native cycle 205 or 203 for PECK and DEEP (#4224).
- The control's own parameter comments (`;SET-UP CLEARANCE`): the control
  adds them; the post writes the Q values only, as its cycle 247 does.

**Not exercised.** The program was not run on the control. The open
Klartext reader (PR #4361) reads cycle 200 and `CYCL CALL`, but it is not
on `main`, so there is no round-trip test yet. It does not read cycles
206, 207 or the old-style cycle 9 blocks.

**Cost to change later.** A native peck needs the proof that #4224 asks
for; then only the PECK / DEEP branch of `onCycle` changes. Another
control level (Q395) needs its own Q list in `onCycle` and a profile that
the manual quill procedure accepts.
