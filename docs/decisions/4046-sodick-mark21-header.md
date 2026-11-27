## #4046 / #4044 - Sodick Mark 21 post: the header names its control and states its threading

**Decided.** The header of every program from `sodick-wire.cps` changes in
comment lines only:

- `(SODICK MARK 21 / EX21 CONTROL WORDS - NOT VERIFIED ON A CONTROL - CHECK BEFORE CUTTING)`
  names the control. The old line said "MARK 21 WORDS" only.
- A new line follows it:
  `(LN1W/LN2W: NOT SUPPORTED - G21 ALARMS E00170 AND THE TAPER WORDS DIFFER - USE THE SODICK WIRE EDM LN2W POST)`.
- The threading line states what the program does (#4044). With AWT
  (`edm.awt: true`) it is `(PROGRAM THREADS THE WIRE WITH T91 AT START)`.
  Without AWT it is `(WIRE NOT THREADED AT START - THE OPERATOR THREADS IT AT THE M00)`,
  because that program stops at an M00 for the operator.

The post's `description` says the same: "Mark 21 / EX21 control" and
"LN1W/LN2W: not supported - G21 alarms (E00170) and the taper words differ
- use the Sodick Wire EDM (LN2W) post". The text before the first " (" stays
"Sodick wire EDM, 2-axis", so `units::post_name` names the post in a
refusal as before.

**Why.**

- An LN2W owner read our only Sodick post as one for their control. On
  LN2W, `G21` stops the program with `E00170` (owner report on #4046), and
  taper is `G50`-`G52` with `A`. The header must say which control the
  words are for before the first code line.
- The alarm number has no parentheses in the G-code comment: the first ")"
  ends a comment on the control. The post description is not G-code, so
  it keeps the wording that the maintainer gave.
- The old threading line contradicted the `T91` some lines below it.

**Dropped.**

- "Taper programs repeat it beside the first G74/G75": this post writes no
  `G74`/`G75` and no taper words. It refuses a tapered or 4-axis section by
  name (`sodick_taper.rs`, `taper_and_four_axis_are_refused_by_name`), so
  no taper program exists to carry the line. The header line is in every
  program, the refused ones included.
- Adding the LN2W hint to the taper and 4-axis refusal messages: the
  maintainer limited this change to comments and the description.
- Renaming the post in the list ("Sodick Wire EDM (Mark 21)"): the name is
  stored in project files, and a rename needs a migration. The name
  already says Mark 21.
- The same threading line in `wire-iso.cps`: that post has its own
  threading codes; #4044 is about the Sodick post only.

**Cost to change later.** Low: comment text and goldens only. Each golden
that carries the header moves by three lines.
