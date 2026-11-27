## #1977 - Haas NGC safe start, end retract, reverse spindle and coolant-off codes

Source for each behaviour: the Haas Mill NGC Operator's Manual, 2025
edition (public on haascnc.com). The page numbers are the printed page
numbers of that manual. No vendor post text was read.

**What was decided.**

- **End retract, G28 mode (the default).** `retractHome()` writes `G90`
  directly after the `G91 G28 Z0` (and `G28 X0 Y0`) blocks, at each
  call: the program end, a setup-change stop and an after-operation
  trip home. Before this change, the program end left `G91` modal at
  `M30`. Source: the M88/M89 sample program, p. 449 (`G91 G28 Z0`,
  `G90`, `M30`); G28, p. 337.
- **End retract, G53 mode.** The machine-Z block is now
  `G53 G49 G0 Z<safeRetractZ>`, and a home ask (the program end, an
  after-operation trip home) adds `G53 G0 Y0.`. A setup-change stop
  lifts Z only, as before. Source: the "Completion" blocks of the
  sample program, pp. 202 and 204 (`G53 G49 Z0`, then `G53 Y0`). The
  next section's tool call writes `G43 H<n>` again (the stream writes a
  `TOOL` record for each section), so the `G49` is not left active
  under a cut. Pinned by
  `haas_g53_retract_cancels_the_length_offset_and_brings_y_home`.
- **Retract choice.** The default stays `G28` with `G91`, now followed
  by `G90`. `G53` stays an option, because it needs
  `machine.safeRetractZ` (the shipped Haas machines state `0`).
- **G94.** The safe-start block is `G17 G90 G94 G5x G40 G49 G80`. G94
  is the reset default and cancels G93; a program that an earlier
  program left in G93 or G95 would read the F words wrong. Source: G94,
  p. 386; the safe-start lines in the manual's own samples carry G94.
- **M4.** `millSpindleReverse = true`; a section with
  `PARAM MillSpindle reverse` starts `S.. M4`; the next section starts
  M3 again (reset on `OperationName`, the Fanuc and Mazak pattern).
  Source: M03/M04/M05, p. 436. `Haas` is removed from
  `tool_gate.rs::M3_ONLY_POSTS`, as decision 4031 says to do when a
  post gains a reverse path.
- **G84 under M4 is refused.** Haas G84 is the right-hand tap and G74 is
  the left-hand tap (Tapping cycles, p. 224; G74, p. 371; G84, p. 381).
  A G84 in a section whose spindle runs reverse is refused by name.
  Today the app does not make such a section; the refusal is a guard.
- **Coolant-off codes.** Each start code gets its own off code: M88 ->
  M89 (p. 449), M83 -> M84 (p. 449), M73 -> M74 (p. 447). M9 stays for
  M8 and for M7, which the NGC manual calls shower coolant and stops
  with M09 (p. 438). Any other start code keeps M9. The map reads the
  start code number from the machine file, so a machine that states
  Haas codes gets the Haas off code.
- **Explain.** `explainWords` gives the Haas meaning of G76 (fine bore,
  p. 371; the shared table says "threading cycle") and of M7, M73,
  M74, M83, M84, M88 and M89.

**Already done on main before this PR.** `explains = "paren"` and
`explain(words)` (PR #1814, #1388); `G90` after the setup-change and
trip-home retracts in `afterOp()` (now moved into `retractHome()`).

**Considered and dropped.**

- **G74 left-hand tapping.** Not in this PR. The app refuses a
  left-hand tap before the post runs (`posting_family.rs`, #1453): the
  stream has no record of the tap hand, and no post declares a
  left-hand tap cycle. G74 needs a new stream word, a manifest flag
  and app changes, so it is a separate issue. The refusal is safe in
  the meantime.
- **M88 before the spindle start.** The manual (p. 449) says to command
  M88 before the spindle speed; after it, the spindle starts, stops and
  starts again. The post writes the coolant after `S.. M3`. This is
  slower, not unsafe, so it is left for a follow-up.
- **`G53 X0. Y0.`** at a home ask: the manual's completion blocks move
  Y only, to bring the table to the operator.

**Cost to change later.** One function each in `assets/posts/haas.cps`
(`retractHome`, `startPreamble`, `coolantOff`) and the Haas goldens.
