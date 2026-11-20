## #2649 - Sodick wire post: pass structure for LN1W / LN2W controls

**Decided.** `assets/posts/sodick-wire.cps` now posts a Wire Contour
with skim passes. Before this change, it refused them by name (#617, #538).
- The rough pass keeps its own C word, written after the threading
  (T91), before the first move.
- Each skim's C word is written after the previous pass's return to the
  threading point and before the skim's first move. The wire stays
  threaded. There is one T91 and one T90 for the section, and the pass
  links stay G01 (no rapid with the wire threaded).
- A note line names each skim and tells the operator that its condition
  must exist in the control's table.
- A skim condition id that is not `C` and three digits is refused by name.
- A new boolean post option, `approachCondition` (default off), writes
  `C000` before the first approach move (threading point to the
  contour). The operation's C word follows that move. Skims get no C000.
- With the option off, the output is byte-identical to the output before
  this change (the #617 goldens do not change).

**Sources.** Sodick Inc., "Wire EDM Machine Operation Training, LN1W /
LN2W Control" (AQ400L, AQ600L, AD360L; part number 6300015, January 2012),
public at fab.cba.mit.edu/content/tools/sodick_edm/sodick_user_manual_small.pdf.
Only behaviour is used, and no text or sample program is copied:
- the word list gives C001 for the first cut, C002 for the second and
  C903 for the third;
- the NC program example selects the next pass's C word after the pass
  returns to its start point, before the next pass, with the wire left
  threaded;
- the condition header rows name C000 / H000 "Approach".

**Why.** The #617 refusal was there because no public source showed a
condition change between passes. The LN manual shows it, so the refusal
is no longer necessary. A 2- or 3-pass contour is the normal LN job.

**Inferred, not stated (check at a control).**
- The manual names C000 "Approach" but does not show where a program
  writes it. We read "approach" as the move from the start point onto
  the contour (the manual uses the word for that move). This is why the
  option is off by default, and why only the first pass gets C000.
- Whether C000 is used at each start hole, or only on some controls.
- Flushing for skims: the manual's example changes flushing for the
  finish passes. This post writes the machine's one `edm.flushing` word
  after the threading only. It writes no flushing word for a skim. The
  operator sets it, as the header already says when no word is stated.
- The program writes no condition header table. The operator must have
  each C row in the control (the existing #617 note).

**Dropped.**
- C000 on by default: no source shows its placement, and a C000 row that
  does not exist on the control would stop the program.
- C000 before each skim: a skim starts at the threading point on the
  finished kerf, and the source does not show it.
- A flushing change for each skim: the machine file states one flushing
  word, and no operation parameter carries a skim's flushing.
- A per-pass offset word (H): the path is the wire-centre path with each
  pass's offset in it, as before (#1498).

**Cost to change later.** The skim C word placement is one function
(`startSkim`). The C000 placement is one branch in `startCut`. A change
of the default changes the #617 goldens.
