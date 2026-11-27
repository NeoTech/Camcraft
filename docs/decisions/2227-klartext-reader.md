## #2227 - Klartext reader, first slice

**What was decided.** `camcraft-gcode` gets a separate Klartext entry,
`parse_klartext` / `parse_klartext_with(program, &KlartextOptions)`, in
`crates/gcode/src/klartext/`. It produces the same `ParsedProgram` /
`ParsedSection` / `cl::Toolpath` output as the RS-274 reader and reuses its
section builder, arc resolver (`crate::arc`), numeric domain checks and the
peck bound (`cycle_expand::peck_count`). The RS-274 word pipeline is not
reused: Klartext has keyword blocks (`BEGIN PGM`, `TOOL CALL`, `CYCL DEF`),
not address words, so a shared lexer would need a mode switch in every step.

The slice reads: `BEGIN PGM <name> MM` / `END PGM` (names must match; a
missing `END PGM` is refused as a truncated program), `BLK FORM 0.1 Z` /
`0.2` (checked, not part of the toolpath), `TOOL CALL` (number or `"NAME"`,
axis Z, `S`, `F`; `DL`/`DR` only when zero), `TOOL DEF` (no-op), `L` (X/Y/Z,
IX/IY/IZ, `R0`, `F` modal, `FMAX` for the block only), `CC` (both axes or
none), `C` with `DR+`/`DR-`, `CR` with `R+`/`R-` (minor/major arc, the RS-274
R-sign rule), `CT` (tangent to the previous element), `CYCL DEF 247` (preset
Q339 = 1..6, read as work offset G54..G59), `CYCL DEF 200`/`203`/`205` called
by `CYCL CALL` or `M99`, `FN 0: Qn = <literal>` and `Qn = <literal>`, Q
references in values (`X+Q1`, `X-Q1`), `STOP`, `;` comments, `*` structure
blocks, `~` continuation lines, and the M functions 0, 1, 2, 3, 4, 5, 8, 9,
13, 14, 30, 99, 137 and `M140 MB MAX` / `MB <n>`. All other blocks and words
are refused by name with the physical line, and the message starts with
`block N:`.

**Why, item by item.**
- *Line vs block number.* `GcodeError` keeps the 1-based physical line (the
  same field as the RS-274 reader, so callers map errors the same way). The
  block number goes into the message text, because the issue asks for it and
  a continuation line makes the two numbers differ.
- *`RL`/`RR` are refused.* The output is the tool-centre path. Reading a
  compensated contour as if it were the centre path would cut in the wrong
  place by one tool radius.
- *`RND`/`CHF`, `LBL`/`CALL LBL`, polar `LP`/`CP`, `APPR`/`DEP`, FK are
  refused.* Rounding and chamfer change the element before and after them
  (look-ahead); label calls need a jump interpreter with its own loop bound.
  Each is a follow-up, not a silent approximation.
- *INCH programs are refused.* The TNC reads `F` in an inch program in units
  of 0.1 inch/min; a wrong scale would be a wrong feed. MM only until a test
  pins the inch rules.
- *Helix (`C`/`CR`/`CT` with a Z change) is refused.* On the iTNC 530 a helix
  is written with `CP` and `IZ`; the plain-arc Z change is control dependent
  (docs/research/heidenhain/differences.md, helix row).
- *`M140 MB MAX` needs `KlartextOptions::tool_axis_max_z`.* The end of the
  traverse range is a machine value. Without it the reader cannot know the Z
  of the retract, and every later move would start from a guessed height. It
  is refused as `MissingReferencePosition`, the same way the RS-274 reader
  refuses `G28` without a configured reference.
- *`M0` switches the spindle and the coolant off.* The iTNC M function table
  lists M0 as program stop, spindle stop, coolant off. `M1` records only an
  optional stop, because a skipped M1 keeps the spindle on.
- *Preset Q339 = n is read as work offset n.* CamCraft's own post writes
  `Q339=+<work offset>` for work offsets 1..6; preset 0 and rows above 6 have
  no `WorkOffset` and are refused.
- *Drilling cycles.* Q meanings come from the public TNC cycle manuals and
  general machining knowledge (our research notes list only the Q numbers):
  Q200 set-up clearance, Q201 depth (negative), Q206 plunge feed, Q202
  plunging depth, Q203 surface, Q204 2nd set-up clearance, Q211 dwell at
  depth. Each infeed goes Q202 deeper from the surface (Q203), the tool
  retracts at rapid to the set-up clearance between infeeds, and ends at the
  higher of the set-up clearance and the 2nd set-up clearance. Options that
  change the motion and that this slice does not model are refused when they
  are not zero: Q210 (dwell at top), Q212 (decrement), Q213 (chip breaks),
  Q257 (chip-break depth), Q379 (deepened start), Q395 (depth reference).
  Accepted without effect: Q205, Q208, Q253, Q256, Q258, Q259 (they change
  only the feed or the re-approach distance on the same line in the hole
  axis, or need a non-zero Q212/Q213/Q257 to act).
- *The re-approach after a retract is one feed move from the set-up
  clearance to the next depth.* The TNC goes down at rapid to an advance
  stop distance above the last depth and feeds from there. The reader's
  line is the same line in the hole axis, so it cannot gouge; only the time
  differs. The `cl::Cycle` record is `Deep` when there is more than one
  infeed and `Drill` for one.
- *`ParsedSection` gets a `tool_name` field.* A named `TOOL CALL "FLAT-6"`
  has no number, so two named tools would merge into one section without
  it. The RS-274 reader sets `None`. No crate outside `camcraft-gcode` reads
  the type yet.

**Dropped.** A Heidenhain `Dialect` preset for the RS-274 reader (wrong
syntax model); a guessed default for `M140 MB MAX` (silent wrong height);
reading `RL`/`RR` as `R0` (silent wrong path).

**Cost to change later.** Each refused function is one new branch in
`klartext/exec.rs` dispatch or `klartext/words.rs`, plus a test; no output
type changes. `LBL` calls need a block list with a program counter instead
of the single pass.
