## #4341 - Z retract before a later tool change (Mazak, Fanuc, Doosan)

**Decided.** The Mazak, Fanuc and Doosan DNM posts now lift Z before the
preamble of each section after the first. The lift uses the `safeRetract`
method of the post: `G91 G28 Z0` (default) or `G53 G00 Z<safeRetractZ>`.
The lift comes after the operation notes and immediately before the
`G90 ... G49 G80` block, so that block's `G90` ends the `G91` of the
`G28` lift. Then `T M6` and `G43 H` follow as before.

- The first section gets no lift. The issue asks for "each tool change
  after the first", and the first-section output (all single-tool
  goldens) stays byte-identical.
- The lift is skipped when Z is already up: after a Go home, and after
  a setup-change lift before its stop. A flag `zUp` records this. The
  `retractHome` helper sets it, and each preamble clears it. A second
  lift would only add a line.
- Fanuc and Doosan: a tilted section leaves `G43.4` armed until the next
  `G49`. A reference return under TCPM is a move that many controls
  alarm on (the Go-home code of the posts says so). So when the ended
  section left TCPM armed (`tcpmLeft`), the post writes `G49` first,
  then the lift. This is the same order as the existing Go home and
  setup-change paths. The upright return after a tilt (#3466) no longer
  writes its own lift, because Z is already up.
- A setup-change stop (`M0` / `M1`) after a tilted section: the stop
  lifts Z before it halts, and sets `zUp`. The upright return of the
  next section then writes no second lift. Before this fix, it lifted
  again after the stop. Z is already at the reference height, and no
  move comes between the stop and the return, so the second lift moved
  nothing. The operator can jog during the stop, but that is true of
  each stop of the program, and the post does not lift again after the
  other stops either.
- Main's #4348 (TCPM cancel before the program-end return) also reads
  `tcpmLeft`. The two changes share the one flag: the section end sets
  it, the next preamble or `onClose` reads it. The take-over merge keeps
  one declaration.
- Fanuc and Doosan are fixed, not only Mazak. A FANUC-family control can
  move Z on a `G49` block with no axis word (the tool length offset
  cancel vector). The "Done means" of the issue asks to fix the two
  posts or to record them as not affected. A lift before the tool
  change is the safe output on each control, so the posts are fixed.

**Why.** The Mazak EIA/ISO manual says that `G49` moves the axis to
remove the length offset. Before this fix, `G49` ran at the clearance
height of the previous operation, so Z could move down by the tool
length near the part. The `safeRetract` option text already says "Z
retract command before tool changes and at program end".

**Dropped.**
- Lift on `G49` in the same block (`G91 G28 Z0 G49`): not a form that
  the three posts write anywhere, and the order of the two actions in
  one block depends on the control.
- Lift only when the tool number changes: the engine sends a `TOOL`
  record for each section, and each later preamble writes `G49` and
  `T M6` again, also for the same tool. So the hazard is the same.
- A lift without the TCPM cancel after a tilted section: a `G28` under
  `G43.4` can alarm.

**Not changed.** Haas, Tormach and LinuxCNC also write `G49` in each
section preamble (Mach4 writes it only in the program header). The issue does not name them. Their controls are not in
the scope of the Mazak audit (#4130). A follow-up issue asks for the same
check on them.

**Cost to change.** Small: one helper (`retractBeforeChange`) in Fanuc
and Doosan, two lines in Mazak, and the goldens of multi-tool programs
(one line each).
