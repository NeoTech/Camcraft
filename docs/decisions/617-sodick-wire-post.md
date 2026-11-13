## #617 - Sodick wire EDM post: a 2-axis slice from the 1998 operation manual

**What was decided.** `assets/posts/sodick-wire.cps` ("Sodick Wire EDM
(Mark 21)") ships as the second `posting::WIRE_POSTS` entry, never the
default. It writes a vertical 2-axis Wire Contour only, in the words of
one public source: Sodick Inc., "Wire EDM Machine Operation" (copyright
November 1998; A320D, A530, A530D, AP350, AP500, A600, A750, A1000; Mark 21
/ EX21 control), read at manualslib.com/manual/2816797/Sodick-A320d.html,
plus RS-274 / ISO 6983 for the motion words. No other CAM product's post,
post library or output was read; the sample program attached to #617 is
another CAM's output and was not read for this work either.

What the post writes, each with its place in the manual:

| Word | Source |
|---|---|
| `G21` / `G20`, `G90`, `G54`-`G59`, `G00`, `G01`, `G02` / `G03`, `G40` | G code table |
| `I` `J` as increments from the arc's start to its centre | word list; arc examples |
| a decimal point on every coordinate | word list ("X5." against the implied-decimal form) |
| the condition as `C` and three digits, on its own block | word list; program example (`C411`) |
| `T91` thread the wire, `T90` cut the wire | T code table; AWT sequence |
| `G40` on a move, not alone | program example (`G40 Y-.02`) |
| `G40` in the same block as `G00` | RS-274 / ISO 6983: G40 is in its own modal group, so it may share a block with a motion word, and cancelling an offset that is already off is legal |
| `M00`, `M01`, `M02` (program end) | M code table; program examples |
| comments in parentheses | program example |

**Why.** The issue asks for a Sodick post; the launch brief limits the
dialect to public sources and asks for one reviewable slice. The 1998
manual is the only Sodick-published programming reference found on the
open web; no free official reference for the later LN controls was found.
The post therefore names the control family it was written from, says in
its own output that it was not verified, and refuses what it cannot
express.

**Decisions inside it.**
- The operation's condition id IS the `C` word (`C110`). An id that is
  not C and three digits is refused by name; the post reads no
  `edm.codes`, so a machine file needs none for it.
- The stream is the wire-centre path (the kerf is in it, #532), so the
  control's wire offset is never engaged: no `G41` / `G42`, no `H`.
  `G40` is written once per section, on the section's first motion,
  the `G00` to the threading point, so an offset left active by an
  earlier program never shifts the positioning, the threading point or
  any cut (review of PR #1506; first written on the first cut, which let
  the `G00` and the threading run under a stale offset). A move is the
  form the manual shows; G40 beside G00 is RS-274's modal-group rule. A
  section whose first cut is an arc is still refused: an arc start from
  the threading point is not shown anywhere in the source.
- The program carries no condition table, so the header says every C
  condition named below must exist in the control's condition table
  before start, and each section names its own (`CONDITION C110 MUST
  EXIST IN THE CONTROL CONDITION TABLE - CHECK ITS VALUES`) before its
  first motion (review of PR #1506). A list of every condition in the
  program's first lines was dropped: the post writes as it reads, and
  splicing lines into the header afterwards would shift the section line
  ranges the engine records.
- No `G92` and no `G29`: positions are in the setup's `G54`-`G59`, which
  the operator sets at the control. A setup without one of those six is
  refused. The header says so.
- Tank fill, flushing and wire-drive T codes (`T84`, `T85`, `T94`, ...)
  are not written: which of them a job needs depends on the machine
  (submerged or not) and the document holds no parameter for them. The
  header says they are set at the control.
- With `edm.awt: false` the operator threads and cuts at an `M00` the
  program states, as in the generic post.
- Taper (`WireTaper`), 4-axis (`WireFourAxis`), a leaning move, a posed
  move, a rotary, a Z change, a rapid while threaded, a tool change, an
  rpm, a canned cycle, a non-XY arc, an arc without a centre, a trip
  home and a non-wire machine or operation are each refused by name.
- No `%` fences, no `N` numbers, no `M30`: the manual's examples carry
  none and end with `M02`.

**Unconfirmed (not run on a Sodick control; check at the machine).**
- Whether the LN1W / LN2W controls of the AQ / AD / ALN series (the
  issue's machine is an AQ600L) read these Mark 21 words unchanged.
- `G40` beside `G00` on the move to the threading point, and a program
  that never states `G41` / `G42`.
- A program with no `G92` / `G29`, relying on the work offset register.
- Cutting with no flushing or tank word in the program.
- The absence of `%`, of a program number and of `M30`.
- The condition written as a bare `C` word with no condition table or
  header rows in the file (later controls keep conditions in the file);
  the program's header and each section tell the operator to check it.
- `M02` leaving the wire cut and the machine safe after `T90`.
- The source was read through a third-party-hosted copy by a
  summarising fetch, not page by page from Sodick.

**Considered and dropped.**
- Reading Sodick words from `edm.codes` like the generic post: that is
  already the generic post; a Sodick post whose words come from the
  machine file adds nothing and pins nothing.
- Using the machine owner's sample program as the reference: its
  generating CAM is unknown, so it is another product's output
  (Clean-Room Protocol).
- Writing `G41` / `G42 H` with a programmed (not wire-centre) path: needs
  a part-path stream and offset registers the document does not have;
  filed as a follow-up.
- Writing `T84` / `T94` unconditionally from the manual's example: wrong
  on a non-submerged job; filed as a follow-up with parameters.

**Cost to change later.** One script and its goldens
(`crates/app/tests/golden/nc/sodick-wire/`,
`crates/post/tests/recorded/golden/sodick-wire/`). A confirmation from a
machine owner changes words in the script; projects store the post by
name, so renaming "Sodick Wire EDM (Mark 21)" would need a migration.
