## #1540 - Setups that differ in End units refuse the whole-program post

**Decided.** The whole-program post refuses by name when an unsuppressed
operation's setup has other End units than the active setup. This is the
same clause (`setup_post_options::program_problems`) that already refuses
a disagreement on the Output and the thread output. The refusal now names
each option that differs, for example `(end units)` or `(output, end units)`.

**Why.** End units is one word for the whole program: the unit mode that
the control is left in after the program ends (#596). Before this change
the program took the word from the active setup, so the posted word
depended on which setup was active, and nothing said so. A named refusal
is the Prime Directive answer: the user posts the setups one at a time,
or sets the same End units on every setup.

**Agreement with #902.** The program's own unit (G20 / G21 at the start,
and every coordinate) is still the document's one mm / inch setting
(`posting_units::program_options`, `postOptions.Units`). No machine file,
post, setup or operation chooses it, and this change does not touch it.
End units is a post option, not the program unit. With this change the
program also has exactly one End-units word: every setup must agree, so
the word never comes from "whichever setup is active".

**Considered and dropped.**
- A documented rule (for example "the first setup's End units") with the
  word stated in the output header. Dropped: it posts a program that one
  setup's owner did not ask for, and it needs header text in every post
  dialect. A refusal is smaller and cannot post a wrong word.
- Moving End units to a document-only setting, like the program unit.
  Dropped for now: #1535 made post options per setup on purpose, and the
  file format keeps End units in each setup's options. The refusal gives
  the same one-word result without a format change.

**Cost to change later.** Low. If the maintainer wants a document-level
End units, the clause removes one comparison and the setup options DTO
drops one field.
