## #1199 - A post script that does not state its dialect posts only C-axis sections on a machine with a work spindle

**What was decided.** Option (b) of the issue. The lathe door
(`app::lathe_voice::at_post`, #1190) now reads the post script's manifest
three ways through `post::dialect::work_voice` (main's #1174 reading, which
this PR adopted on merging main; an earlier head of the PR had its own
`declared_turning` and read masks 0 and 4 as a declared mill post):

| The script states | The door |
|---|---|
| `capabilities` with the turning bit (2, 3) | fires for every operation, as before |
| the milling bit alone (1) | passes, as before - a declared mill post |
| no mask naming 1 or 2 (no `capabilities`, 0, 4, a string, a negative or non-finite number) | on a lathe or mill-turn setup, refuses every operation but a C-axis live-tool section (one whose path states C): a milling section *and* a section that turns the work (a turning kind, an on-centre Drill); the refusal names the script when the caller knows it, says the manifest line is missing and what to add (`capabilities = 1;` for a milling post, `= 2;` for a turning one), and for a turning section says why (an unstated script is not known to be a turning post); passes on every other machine |

Cell by cell, for an undeclared script:

| Machine | Turning section / on-centre drill | Milling section | C-axis section |
|---|---|---|---|
| lathe | refused by name | refused by name | passes the door |
| mill-turn | refused by name | refused by name | passes the door |
| mill | not creatable (family door) | posts | posts |
| none (reads as a mill) | not creatable (family door) | posts | posts |

So a user's own turning script without the manifest line can no longer
post a milling section on a work spindle, and a user's own *milling*
script without it can no longer post a turning section as milling (a tool
change, `S M3` on the tool spindle, the work standing still). A C section
is the one section both dialects open alike, so it passes. A genuine
undeclared turning script is refused too, with the one-line fix.

**Why.** The Prime Directive: where the dialect matters - a machine with a
work spindle - an unstated dialect is not guessed to be the harmless one.
An unstated script may be either kind of post, so there it posts only
what both kinds post alike, and the refusal names the fix. A section that
turns the work posts only through a turning post (maintainer, 2026-11-12,
on #1232 and PR #1265), on a mill-turn as on a lathe; an undeclared
script is not known to be one. (An earlier head of this PR held it to the
turning rules, which let a turning section through a user's undeclared
milling post on a lathe or mill-turn - the review of head `17f6ce33`
showed it posting `T01 M6`, `S1500 M3` for a Facing op.) Where it does not matter (a mill, a
plasma table, an EDM - no work spindle to turn), nothing changes.
`speaks_turning` keeps its old meaning (undeclared = not turning) for any
caller that only wants the declared bit; the door no longer uses it.

**What changes for users.** A user's own script without `capabilities`,
posting anything but a C-axis section on a mill-turn or lathe setup, is
now refused by name with the one-line fix. Every shipped post states
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

**Residual, not covered.** A document with no machine or a mill
machine, posted through an undeclared *lathe* script, meets no door here:
no work spindle is mounted, so the script is not judged. Only the script's
own meridian check stands in the way (it refuses a Face pass "off the
spindle axis meridian"). That is a mislabelled document rather than a
lathe one, and #1265 does not cover it either (a mill accepts any
non-turning mask). Option (a) below would close it.

**Considered and dropped (continued).**
- *Holding an undeclared script to the turning rules on a lathe or
  mill-turn* (the earlier head): it lets a turning section through a
  user's milling post without the line - dropped for the stricter rule.

**Cost to change later.** If option (a) is wanted later, it is one arm in
`at_post` (refuse on `None` regardless of machine) plus adding the line to
the inline test scripts. The machine test is `work_spindle_machine` in
`lathe_voice.rs`.
