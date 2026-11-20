## #2686 - Sodick wire post: the flushing of a skim pass

**Decided.** A wire machine file can state `edm.skim_flushing` (`high` or
`low`). It is the flushing of each skim pass of a Wire Contour.
- `assets/posts/sodick-wire.cps` writes its T word (T84 / T85) before
  each skim's C word. The order is the #1499 order: the T word, then the
  C word.
- The rough cut keeps `edm.flushing`, written after each threading
  (#1499). The next operation's rough cut selects it again, so a skim's
  word does not stay active.
- `edm.skim_flushing` needs `edm.flushing`. The machine file parser, the
  parsed-machine check (`Edm::is_sane`, `Edm::refusal`) and the post
  refuse a skim flushing without it, by name.
- A skim on a machine that states `edm.flushing` and no
  `edm.skim_flushing` is refused by name. The app writes no file.
- A machine that states no flushing posts its skims as before (#2649).
  The header tells the operator to set the flushing at the control. The
  three existing Sodick goldens do not change.
- The key is in the post's machine mirror only when stated. The machine
  editor offers it only while a flushing is stated. The MCP
  `edit_machine` tool takes it through the same machine patch.
- `assets/posts/wire-iso.cps` has no word for it. It writes a note that
  the stated skim flushing is not programmed.

**Why the machine file.** The issue permits the machine file or an
operation parameter. `edm.flushing` is a machine fact already (#1499),
and the rough cut's word comes from it. One more key of the same type
keeps the two values together and needs no new command, no project file
change and no new MCP tool.

**Why the refusal.** T84 / T85 is modal. After #2649, a program with
`edm.flushing: high` and skims cut each skim on high pressure, and the
program did not say so. That value was not stated by the user. The
issue's "refuses by name when the value is not known" is read as this
case. This is a behaviour change: such a program posted before and is
refused now. The message names the key to state.

**Sources.** The T84 / T85 words are from the T-code table of the 1998
Sodick "Wire EDM Machine Operation" manual (#617, #1499). The issue
states that the program example of the 2012 "Wire EDM Machine Operation
Training, LN1W / LN2W Control" manual (part number 6300015) changes the
flushing for the finish passes. This PR did not read the manual again.
The position of the word (immediately before the skim's C word) is
inferred from the #1499 order, not read from the manual. The output was
not run on a Sodick control.

**Dropped.**
- A flushing for each skim pass (`WireSkim.flushing`, an operation
  parameter): it needs a project file field, a command change, an MCP
  change and a property panel field. One value for all skims covers the
  manual's example. It can be added later without a change to this key.
- A flushing field on each condition row (`edm.conditions[].flushing`):
  it ties the flushing to the C word, which is closest to the control's
  own table. It is a larger change of the condition table and its
  editor.
- A default (low pressure for a skim): the post never guesses a value.
- Skipping the T word when the skim flushing equals the rough flushing:
  the word is always written, so the program states each pass's flushing.

**Cost to change later.** A per-skim or per-condition value can
override `edm.skim_flushing`. The key then stays as the default for
skims that state none.
