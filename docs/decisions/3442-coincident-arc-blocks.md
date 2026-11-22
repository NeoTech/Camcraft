## #3442 - No arc block whose written end is its written start

**Decided.** A shipped post never writes an arc block (G2/G3, or the
dialect's arc block) whose end point, as written, equals its start point
in the arc plane. "As written" means the script's own word: the output
unit, the precision, the work origin and the modal words applied. The
comparison uses the text of the words, never the stream's millimetres.

One rule, in one place: `arcFate(word)` in the PRELUDE
(`crates/post/src/post/arc_guard.rs`, appended by `Script::load`). Each
shipped script calls it at the top of its arc path with its own word
function and obeys the answer:

- `"arc"` - the written in-plane ends differ: the arc posts as before.
  Also when the ends coincide on a record that turns more than half a
  circle (its sweep from its direction and centre), or whose ends are
  equal in the stream (a full turn stated as one record): the full
  circle is that arc within the output resolution, and the dialect
  writes it as before. No distance band: a true short arc with a chord
  under any band is still short (a band was tried after the second read
  and dropped after the re-read: under it, a true short arc became a
  full circle).
  Also when the record states no centre: the script's own refusal by
  name (sodick-wire, heidenhain DIN, tinyg) still runs.
- `"linear"` - the ends coincide on a short arc, and the plane's normal
  axis (a helix) or a rotary still moves as written: the record posts
  as the script's own straight feed move (`onLinear`) to the end point.
- `"none"` - the ends coincide on a short arc and nothing changes as
  written: no motion block. The feed word and the modal state are not
  touched, so the next block writes its feed as before. A script that
  tracks the last programmed point updates it (Klartext `motion()`,
  the lathes' radius memory).

**Why.** RS-274 / ISO 6983 controls read an arc block whose end is the
current position as a full circle. A short arc (the reporter's 1 um arc,
0.007 degrees) under the output step therefore cut a full circle. The
tool moves less than the output step in the plane on such an arc, so a
straight move (or no move) stays within the output resolution of the
programmed path. The input stays legal: no input refusal was added.

The sweep test (more than half a turn) separates a near-full turn from a
short arc: for a near-full turn the full circle is the right block, and a
straight move would leave the circle. A centre that is on the start as
written (a radius under the output step) is never a full circle.

**Klartext.** `heidenhain-klartext.cps` refused every arc whose ends
coincide after rounding. It now takes the same rule: a short arc is a
straight `L` move or no block. Its refusal stays only for a near-full
turn whose written ends coincide, which the Klartext `C` block is not
specified for in the manual that the script cites.

**Other dialect points.**
- The lathes (`fanuc-lathe`, `tormach-lathe`) apply the rule in their
  meridian (XZ / YZ) arc path, on their own X (radius to diameter) and Z
  words. Their XY half-turns are thread passes and are not arcs here.
- `mach4` applies it after its trunnion transform, on the machine-frame
  words that it writes.
- `tinyg` refused a collapsed XY segment by name; a short arc is now a
  straight move or no block, as for the other posts.
- `sinker-iso` refuses every arc and is unchanged.
- The R-word mode (`useRadiusArcs`) is after the guard: a collapsed arc
  never reaches it, and a short arc that the output can show keeps its
  minor (positive R) solution.

**Dropped.**
- A refusal of the arc for all posts (the old Klartext behaviour): it
  blocks a legal program for a move under the output resolution.
- An engine-side downgrade before `onCircular`: the engine does not know
  each script's word format (origin, unit, precision, lathe diameter,
  trunnion frame), and a second formatter could disagree with the
  script's own words.
- A per-script copy of the helper: twenty copies would drift.
- A minimum chord or sweep in the CL or the post reader: an arc of
  0.007 degrees is legal input and its post is now safe.

**Cost to change.** The helper is one function; a dialect with a
different full-circle rule changes its own call site only.

**The stream writer (after the re-read).** The app posts through
`cnc::serialize`, which writes each position as whole micrometres. A
generator arc with a chord under one micrometre (#3463: a fillet can
reach 0.09 um) rounded to a record whose end equals its start, which the
reader takes as a full turn: every post then wrote a full circle, on a
large-radius fit arc a large circle through the part. Only the writer
still knows the true sweep, so the writer decides
(`crates/post/src/post/cnc_arc_lattice.rs`, called by `serialize`):
an arc whose in-plane ends round to the same micrometre and whose true
sweep is half a turn or less is written as a `Feed` to its rounded end.
A near-full turn keeps its arc record (equal ends: the full turn within
one micrometre). A true full turn reaches the writer as two halves (the
CL `fit_arcs` split; `push_arc` drops a zero-sweep arc), or as one
record with exactly equal ends, which is the stream's stated full-turn
form.

- The record stays, also when nothing moves after rounding (a
  zero-length `Feed`): the engine's line map counts stream records as
  toolpath moves, and a dropped record would shift the viewer's sync.
  The post writes a legal zero-length `G01` for it.
- The reader is not made stricter. An arc record whose ends are equal is
  the stream's full-turn form (`full_circle_raw`, with a golden on every
  post), and with the writer fixed our own streams never use it for a
  short arc. A refusal would break that legal form; a zero-sweep reading
  would turn a stated full turn into no motion. Dropped.

