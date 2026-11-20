## #2474 - WCS flip: a flipped Z stays refused, with the exact reason

**What was decided.** A flipped Z does not post. The refusal stays at the two
doors (the verification gate, `app::setup_wcs_flip`, and the post engine,
`post::wcs_flip`). It now names the exact reason instead of a general one.
The two doors read the reason from one constant (`post::wcs_flip::Z_REASON`).
The issue permits this result: "or the refusal names the exact reason and the
decision is recorded".

**The exact reason: the tool-length word.** A flipped axis reads the same
physical point with the other sign. For X and Y the mirror is complete in the
program: signs, G2/G3 and G41/G42 follow from geometry. For Z, one word does
not follow from geometry: the tool-length word.

- The tool is on the physical "up" side of its tip. On a control that reads Z
  reversed, "up" is the control's -Z.
- A register that holds a preset length (a positive number from a presetter)
  then needs the subtracting word, G44.
- A register that holds a length touched off on that control already has the
  control's sign. It needs the adding word, G43.
- The project states only "Z is flipped". It does not state how the tool
  lengths were measured. The two cases need opposite words.
- The wrong word moves the tool by twice its length, toward the work in one of
  the two cases. A note in the program cannot make that safe (Prime Directive).

**The second reason: canned cycles.** A drilling cycle states an R plane and a
hole bottom. A mirrored Z puts the R plane on the far side of the hole bottom.
The public standards (RS-274 / NIST RS274NGC, ISO 6983) define the cycles with
the R plane above the bottom, and some controls refuse the other order. The
post engine names this only for a section that has a canned cycle.

**The open questions of the issue.**

- "Is a flipped Z a real machine setup?" Yes, it can be: a machine whose Z+
  moves toward the work (some routers), or a fixture frame that points down.
  The first is a machine property (`"direction": -1`, #374), and that door
  refuses Z for the same reason. The decision does not depend on the answer:
  the two cases have the same unknown tool-length word.
- "Which dialects can state a negative tool-length direction?" Fanuc and Haas
  have G44. LinuxCNC has G43 only. Heidenhain and Siemens do not use the word
  pair. This is from general machining knowledge and was not checked against
  each manual. A carry by G44 alone cannot cover each built-in mill post.

**What was dropped.**

- *Mirror Z and keep G43, with a program note.* Wrong by two tool lengths for
  preset registers.
- *Mirror Z and swap G43 to G44 (the geometric rule, as G41/G42).* Wrong by two
  tool lengths for touched-off registers, and not available in each dialect.
- *Expand the canned cycles into plain moves for a flipped Z.* It solves the
  second reason only. It has no use while the first reason refuses.

**What a carry needs (the maintainer decides).** One new statement from the
user: how the tool lengths of the flipped setup are measured (preset, or
touched off on the control). With it, the post can select the word, or refuse
a dialect that cannot state it. That is a new parameter with its command, MCP
tool, project-file field and one equivalence test for each dialect
(`crates/post/tests/wcs_flip.rs`, a reader with Z flipped that models the
tool-length word). The setup sheet, the travel check and the viewport envelope
need no change while Z is refused.

**Cost to change later.** Low. The refusal is one branch in each door and one
shared constant. The X and Y mirror (`invert::translate_rules`) already
negates Z coordinates when asked; the cycle branch and the tool-length word
are the open parts.
