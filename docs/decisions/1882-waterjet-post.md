## #1882 - Waterjet post seams and the generic ISO waterjet post

**What was decided.**

- The post engine reads a `lineEnding` manifest entry: `"lf"` (the
  default) or `"crlf"`. Any other value is refused by name. The engine
  reads it when it writes the program, after `onClose`, so a script can
  set it in `onOpen` from a declared option.
- `script_props` publishes `machine.travel`: `{X, Y, Z}`, each the
  stated stroke `[min, max]` mm or `null`. The mirror now carries a
  linear axis's stated `travel` beside its rotary travel. The raw door
  reads a vendor tree through `parse_mch`, as the rotary mirror does.
- The engine publishes `programBounds` before `onOpen`: the XY box of
  the cutting records (feeds, plunges, arcs at their swept quarter
  points through `machine::move_extremes`, cycle centres). Rapids do not
  count. `null` when nothing is cut.
- New stream records `WATERJET [ABRASIVE] [DWELL<ms>] [LAG<ms>]` and
  `JET ON C<id> <ROLE>` / `JET OFF C<id>`, with the cl types
  `cl::WaterjetCut` / `cl::JetSpan`. Both spellings are read (`.cnc`
  and `.cl` comment form), like the plasma records. The engine keeps a
  `JetLaw` of its own (no jet without a header, no ON while open, no
  rapid while open, nothing left open) and fires `onJetOn` / `onJetOff`.
  It shares no state with the plasma `TorchLaw`.
- `linearizeArcs = true` (read after `onOpen`) posts each arc record as
  chords through `onLinear`, at `chordTolerance` mm (0.01 when unstated,
  a bad value refused by name). The chords come from
  `cl::Move::segments3`, which is the 3-D twin of `Move::chords` (same
  `arc_chords` sagitta law). A helix keeps its Z ride, which the XY-only
  `chords` would drop. The last chord ends on the record's own end point.
  All chord lines map to the arc record, so the trace and the line map
  stay one record per arc.
- `assets/posts/waterjet-iso.cps`: a generic ISO abrasive-waterjet post.
  XY only (a waterjet machine states X and Y). Per contour: water on,
  `G4` valve lag, abrasive on, `G4` pierce dwell, cut, abrasive off, water
  off. Declared options: the four M numbers, the G4 word form
  (`P` seconds, `X` seconds or `P` milliseconds), G2/G3 or chords, LF or
  CR LF. It refuses by name: an rpm (S, G95), a surface speed (G96),
  coolant, a section with no `WATERJET` header, a machine not declared
  `kind: waterjet`, a rotary, a cycle, a Z cutting move, and a cut outside
  the stated `machine.travel`.

**Why.** The issue asks for these seams so that the WAZER path (#1886)
and the WaterjetProfile operation (#1883) can build on them. The
waterjet records copy the plasma record pattern, because one reader for
both spellings keeps a `.cl` golden a legal post input.

**Considered and dropped.**

- A `Toolpath::waterjet` field and its `.cl` dump. Each `Toolpath`
  literal in five crates would change, and no generator writes waterjet
  records yet. The WaterjetProfile operation (#1883) adds the field with
  its generator; the line format and the reader are ready here.
- Registering the post in `posting::BUILTINS` and a `WATERJET_POSTS`
  list. No operation makes a waterjet section yet, and
  `crates/app/src/posting.rs` is changed by several open PRs. A user can
  still post with it as a file of their own (`serves = ["waterjet"]`).
- An `onWaterjet` header hook. The header's properties reach
  `onSection` and `onJetOn`; the issue names only the two jet hooks.
- Fixed M numbers for water and abrasive. No standard fixes them, so
  they are declared options. The defaults (M10/M11, M12/M13) are
  placeholders, and the post header comment says so.
- Writing Z on rapids. The waterjet machine kind states X and Y only.
- `lineEnding` and `linearizeArcs` read before `onOpen`. Then a declared
  option could not choose them.

**Cost to change later.** The record words are new; a rename touches
`cl/jet.rs`, `cnc_jet.rs`, the two docs and the goldens. The default M
numbers are in one place in the post and in its golden.
