## #1199 - A post script that does not state its dialect is held to the lathe door on a machine with a work spindle

**What was decided.** Option (b) of the issue. The lathe door
(`app::lathe_voice::at_post`, #1190) now reads the post script's manifest
three ways through `post::dialect::declared_turning`:

| The script states | The door |
|---|---|
| `capabilities` with the turning bit (2, 3) | fires for every operation, as before |
| a mask without it (1, 0, 4) | passes, as before - a declared mill post |
| no usable mask (no `capabilities`, a string, a negative or non-finite number) | fires for every operation whose setup mounts a lathe or a mill-turn, and its refusal says the manifest line is missing and what to add (`capabilities = 1;` for a milling post, `= 2;` for a turning one); passes on every other machine |

So a user's own turning script without the manifest line can no longer
post a milling section on a work spindle unnoticed: on the only machines
that have one it is judged as a turning dialect. What a lathe runs (the
turning family, an on-centre Drill, a section stating C) still posts
through it.

**Why.** The Prime Directive: where the dialect matters - a machine with a
work spindle - an unstated dialect is not guessed to be the harmless one.
Reading it as turning there only ever refuses (never changes a posted
line), and the refusal names the fix. Where it does not matter (a mill, a
plasma table, an EDM - no work spindle to turn), nothing changes.
`speaks_turning` keeps its old meaning (undeclared = not turning) for any
caller that only wants the declared bit; the door no longer uses it.

**What changes for users.** A user's own *milling* script without
`capabilities`, posting a milling operation on a mill-turn or lathe setup,
is now refused by name with the one-line fix. Every shipped post states
its mask, so no shipped post, golden or registry pairing changes.

**Considered and dropped.**
- *(a) Refuse every script without `capabilities`, everywhere.* Safest on
  paper, but it refuses user mill scripts on plain mills where no work
  spindle exists - a refusal that protects nothing - and breaks the many
  small inline test scripts. Option (b) refuses exactly where the hazard
  lives.
- *(c) Document the line as required and warn once.* A warning lets the
  section post; the Prime Directive asks for a block, not a note.
- *Deciding by the setup's machine alone* (any milling op on a lathe or
  mill-turn refused regardless of the script): #1190 already dropped this -
  a mill-turn posting through a declared mill post mills correctly.
- *Treating a non-number `capabilities` (`"2"`) as declared-not-turning*
  (what `speaks_turning` reads): a mask the engine cannot read is not a
  statement, so it is undeclared here.

**Cost to change later.** If option (a) is wanted later, it is one arm in
`at_post` (refuse on `None` regardless of machine) plus adding the line to
the inline test scripts. The machine test is `work_spindle_machine` in
`lathe_voice.rs`.
