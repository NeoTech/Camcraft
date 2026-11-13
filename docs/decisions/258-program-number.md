## #258 (part 2) - the program number and program comment

**What was decided.** The document states its program identity in a new
`ProgramInfo` singleton - `number: Option<u32>`, `comment: Option<String>`,
`None` meaning "not stated" - set only by the undoable `SetProgramInfo`
(from the Post page's Program number / Program comment fields, and the
new MCP tool `set_program`) or a document load. The engine hands it to
every script as the `program` global `{ number, numbered, comment }`
(`engine::run_program_with`); `programNumber` stays, holding the same
number. Nothing stated posts byte-identical to before (O0001 and the
machine's name), so no golden changed.

**Validation is split in two.** `SetProgramInfo` refuses by name, at
creation, everything no shipped dialect can hold: a number outside
1-9999, a comment longer than 40 characters (the posts' comment width,
#257), non-printable or non-ASCII characters, and `( ) % ; *` (they end a
comment or a block somewhere: RS-274 `( )`, the tape mark `%`, the DIN
`;` comment, the TNC block end `*`). The comment is trimmed and a blank
one is none. A dialect's own narrower rule refuses by name at post time
(a SINUMERIK name's first letters, an O-word past its range). A project
file holding a value the command would refuse is a corrupt file.

**Refused, not sanitised, at the command; sanitised by name rules at the
post.** The user's characters that would break every dialect's comment
are refused where they are typed, so a comment never posts silently
different from what was typed. Name-shaped dialects (Heidenhain,
SINUMERIK) turn spaces and the rest into `_` - their names are file
names, and the comment still reads as typed in the header comment line.

**Per dialect** (sources: each control's own published programming
manual, behaviour only; the citation sits beside the code in the .cps):
- Fanuc, Fanuc Lathe, Mazak: `O####` (O0001-O9999, four digits - the
  FANUC operator's manuals' program-number range; the 8-digit option and
  Mazak's wider work numbers are not assumed) followed by
  `(<comment or machine name>)`. Past O9999 is refused, no longer clamped.
- Haas: `O#####` (five digits, O00001-O99999 per Haas's mill operator's
  manual) only when a number is stated - the default header has always
  been the bare comment and stays so.
- Tormach PathPilot: no O-word ever - on the LinuxCNC interpreter an O
  word opens a subroutine or flow-control block. A stated number rides
  the header comment: `(PROGRAM 1234 - <comment or machine name>)`.
- Heidenhain (ISO mode): the TNC addresses a program by name, so the
  comment IS the name (`%<name> G71 *`, file-name characters A-Z a-z 0-9
  . _ - per the TNC 640 ISO manual), else a stated number names it
  (`%1234 G71 *`), else the machine's name as before. With both stated,
  the number follows as `N10 ; PROGRAM 1234*`.
- SINUMERIK 840D: a stated comment becomes the program name in
  `%_N_<NAME>_MPF` (letters, digits, `_`; the first two characters letters
  or `_` and a letter; at most 24 characters - Siemens' Fundamentals
  programming manual, "program names"); a name the rules refuse is
  refused by name, never renamed into a different program. A number
  has no word there; it rides the `; PROGRAM 1234 - ...` comment.

**One file per operation counts up from the stated number** (#111's
1, 2, 3 become N, N+1, ...), refused by name past O9999. A multi-machine
post (#388) numbers each mount's file with the document's number, as it
wrote O0001 for each before.

**Project v61.** The new `program` key is a version step, not additive:
an older reader would drop it and post O0001, which on the control can
overwrite a different program held under that number. v60 and older
files step up unchanged (absent = nothing stated). The step was v60
until #902 (PR #981) took v60 for the document-unit law first; #564
renumbered to v61 when it merged main, so the two laws keep separate
rungs and a v60 reader still refuses a file that states a program.

**Considered and dropped.** One range per dialect at the command (the
command does not know which post - setups may post through different
ones); the shared 1-9999 range fits every O-word dialect shipped.
Sanitising `( )` silently at the command (a typed comment would post
different from what the field shows). Writing an O-word on PathPilot
(illegal there) or a numeric SINUMERIK name (the name rules refuse a
leading digit; an invented prefix would be our convention, not the
manual's). Writing the `;$PATH=/_N_MPF_DIR` line: optional, and it fixes
the directory on the control.

**Cost to change later.** The range and refused characters are three
constants in `app::posting`'s program module; a dialect's rule is its
`.cps` header block. Widening to the 8-digit O-number option is a
per-post change plus the command's range.
