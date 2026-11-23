## #3542 - the pass-start fallback warning names its own cause

**Chosen.** `Warning::StrategyFallback` gets a `cause: FallbackCause`
field (`Adaptive` | `PassStart`, `#[serde(default)]`, default `Adaptive`).
The Adaptive text stays byte for byte. The pass-start cause has its own
text:

> Pass starts at: the selected start side would leave material uncut in
> N area(s), so the Auto order was used there instead, from move M: the
> notes name each one (the post is not held: the cut is verified)

The op sets the cause from the notes (`pass_start_fallbacks`), never from
the op kind. A guard note contains `PASS_START_NOTE` and ends with the
shared tail `PASS_START_FALLBACK_TAIL` ("this area in the Auto order
instead", used by `StartSide::fallback_note`). The match does not read the
start of the note: 3D pocket (#3618) writes the note after its level, as
`Z <level>: <note>`. No Adaptive note ends with that tail. 2D Pocket, 2D
Offset and 3D pocket reach the guard, and an Adaptive op on its
ring fallback can write an Adaptive note and a guard note on one path.
`PassStart` is set only when every fallback note is a guard note; any
Adaptive note keeps the Adaptive cause and text. The warning moved into
`operation::strategy_fallback(&Toolpath)` so that a test can pin it
without a generator run.

**Why.** A defaulted field reads in both directions: an old cache reads as
`Adaptive`, and an older build ignores the extra field. The MCP kind stays
`strategy_fallback`; the message carries the new text.

**Dropped.**
- A second variant (`PassStartFallback`): an older build refuses a cache
  with an unknown variant, and MCP clients would learn a new kind.
- The cause in the MCP finding: the finding DTO is flat (op, move, kind,
  message, blocks_post) and no other kind carries its fields, so it has no
  natural place. The message names the cause.
- A choice of the text by op kind: wrong for an Adaptive path with both
  notes.
- A true count in the mixed Adaptive case (one area writes an Adaptive
  note and a guard note, so `areas` counts 2): unchanged here, #3622.
- A fix of the `pocket_avoid` count ("Also seen"): each level writes one
  guard note for the whole level, and the de-duplication at
  `pocket_avoid.rs:159` drops a second level with the same text. The fix
  touches motion notes and is not a few lines; filed as #3622 (P7).

**Cost to change later.** One enum and one field, additive in the project
file.
