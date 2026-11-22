## #3360 - Bore takes the shared "Pass starts at" parameter

**What was decided.**

- Bore gets the field `pass_start` with the shared type
  `machining2d::PassStart`, the project key `pass_start`
  (`PassStartDto`, `auto` never written), the generic MCP door
  `edit_operation`, and the shared Passes-tab row `Pass starts at`. No new
  type. This PR is stacked on #3353 (#3316), which adds these parts.
- Bore has a pass order: the concentric rings of each level. The motion
  entry point `bore_started(p, start)` orders them from the shared
  `PassStart` value. The old `bore(p)` calls it with Auto.
- Ring `k` (1 is the innermost, `count` is the wall ring) is the offset
  index `count - k` from the wall. The cutter-center disk of a hole has no
  hole of its own, so there is no inner family.
  - Auto: the centre-out walk. This is the old loop order, so Auto output
    is byte-identical.
  - Outside: the wall ring first, then each ring in to the centre.
  - Inside: the disk has no inner wall, so the centre-out order. Inside is
    the same path as Auto. The UI shows a Bore hint that says so.
- Outside cuts the wall ring first, a full tool width along the hole wall
  in uncut stock. The family law of #3402 (#3316) names that cut in the
  path notes through `StartSide::wall_first_note`, so the op page shows
  the warning. Auto and Inside start at the centre in open air and post no
  such note. The note does not change the moves.
- Each level still dives at the centre and sweeps to the first ring
  start. The entry law (plunge, ramp along the radial sweep, the
  centre-cutting law) does not change. Under Outside the radial sweep
  goes to the wall ring, so it is longer.
- The digest writes the value only off Auto (`0xC1` Inside, `0xC2`
  Outside), the same bytes as 2D Offset. Every earlier Bore keeps its
  cache stamp.

**Room for the #1448 start point.** The start point and the exit point go
beside `pass_start` as their own fields. For Bore, the start point
rotates each ring to its entry angle; today each ring starts at angle 0.
`bore_started` can take the pair when #1448 lands.

**Why.** The maintainer asked that every kind whose passes or rings have
an order gets the same option (#3316, 2026-11-22). The ring ladder of a
bore has an order, so the row is live, not a "no pass order" row.

**Considered and dropped.**

- A `pass_start` field on `motion::BoreParams`. Many struct literals in
  motion tests would change for one value. An argument on a new entry
  point keeps every other caller byte-identical, as #3316 did for the
  pocket.
- A Bore-only order (for example "wall last" and "wall first"). The
  maintainer asked for one shared parameter.
- Dive at the wall ring start under Outside. That changes the entry law
  (the dive point and the ramp path). The centre dive is kept, so the
  centre-cutting and ramp laws stay as they are.
- Disable Inside on Bore. Inside is a valid word for the shared law; it
  gives the centre-out order. The hint names this.
- The `camcraft-mcp` skill row for Bore. The edit needs an approval for
  `.claude/` in this session. The generic row of #3316 already shows the
  key. A follow-up can add the Bore sentence.
