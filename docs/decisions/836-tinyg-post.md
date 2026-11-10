## #836 - Synthetos TinyG post processor (3-axis mill)

**Decided:** a new built-in post, `assets/posts/tinyg.cps`, specced from the
G-code and M-code tables of TinyG's public wiki ("Gcode Support", "Homing and
Limits", "Flow Control and Footers", "Inch and Millimeter Units Mode") and the
LinuxCNC G-code reference TinyG states it follows (behaviour only; the
firmware's source is GPL and was not read). Structure from `grblhal.cps`
(#710), the other grbl-family dialect; every golden was replayed through the
edge-next firmware's own parser (build 450) off the board, with no refused
line.

- **No canned cycles at all.** TinyG lists G80 and nothing else of that
  group; G81-G89 are rejected like any unlisted G code. A CYCLE record writes
  nothing and the hole's expanded chain posts as plain moves (the Heidenhain
  pattern), with the dwell as `G4 P<seconds>` right after the move that
  reaches the hole's bottom (P is seconds, 0-4294, so a longer dwell is
  refused by name). A dwell whose chain never reaches the bottom is refused,
  not dropped. Tapping is refused by name: no G84, no spindle
  synchronisation, and a tap cannot be plain moves.
- **`T<n> M6` then `M0` on its own line, after the Z lift.** On TinyG M6 only
  makes the T word the current tool and moves nothing; the halt the operator
  needs to change and measure the tool is the program stop, queued with the
  moves and released by the sender's cycle start. The first tool gets the same
  stop, so the operator confirms the tool before the spindle starts. *Dropped:*
  `M60` (the wiki lists it as "same as M0", but the common senders - cncjs,
  Chilipeppr - react to M0/M1/M6 only); a sender-specific tool-change macro
  (the post writes what the board itself does). Because of this M0 the post
  is pinned by `tinyg_post.rs` for its after-operation stops instead of
  joining `after_op.rs`'s mill list, whose tests count every M0 in a program
  as the operator's stop.
- **Tool numbers 1-31**, refused by name outside that range: the board keeps
  the tool number in five bits and refuses a larger T with a status a sender
  may not stop on.
- **No tool length word** (no G43.1, no G49) - as #710 decided for grblHAL,
  and for the same reason: the offset belongs to the operator or sender that
  measures the tool at the M0, and a G49 would throw a measured length away.
  *Dropped:* G49 in the header to clear a stale offset (SolidCraft's TinyG
  post does this; here it would also cancel a length the sender set for this
  job). The decision is cheap to revisit as a post option.
- **Header `G21|G20 G17 G90 G91.1 G94 G64`**: the words a program end does
  not reset on TinyG (the wiki's "M2, M30 Program End": units, arc-centre
  mode and path-control mode persist), so a previous program's G90.1 or
  G61.1 would otherwise stand. No G40 (accepted and ignored) and no G80. No
  %, O word or N numbers; a line starting with % is a queue flush.
- **Lengths at four decimals in either unit.** TinyG checks that an arc's
  start and end sit at the same radius from its I J K centre; a tenth of a
  micron keeps a rounded centre inside that check where three decimals do
  not always. Feeds are whole mm/min (two decimals in/min).
- **Comment cleaning** also drops `!` (feed hold), `~` (cycle start), `%`
  (queue flush), `/` (block delete), `{` and `$` (JSON / settings when a line
  opens with them), wherever they stand - TinyG acts on the first two even
  inside a comment. A note opening with `msg` is led by "- " so it is not
  echoed as an operator message. Lines are kept to 191 content bytes (the
  receive ring has 248 usable bytes; a sender on the board's RTS marks has a
  192-byte window); a longer line is refused by name.
- **Z lift as grblhal.cps:** `G53 G0 Z<safe_retract>` only when the machine
  file states it (the statement "this machine homes, and up is here");
  otherwise `G0 Z<clearance>` in the work offset, refused by name when the
  height is unknown or not above the stock. An unhomed TinyG has its machine
  zero at the power-on position (`$home` 0). No G28/G30: their positions are
  set at the machine with G28.1/G30.1, not kept across a power cycle, and
  machine zero when unset.
- **Control compensation refused by name** (G41/G42 are rejected by the
  board); **go home refused by name** (see G28 above); **probing refused by
  the engine** (no probe hooks: TinyG has G38.2 but no parameters or
  expressions - a `#` in a line is stripped before parsing).
- **No spindle dwell after M3.** The wiki says M3 does not wait for the
  spindle and suggests `G4 P2` where needed. *Dropped for now:* a post option
  for it (needs the Setup pane's post-option row); the operator's M0 before
  the spindle start gives the same pause for a manual change. Cheap to add.
- **Generic machine `tinyg-3axis.mch` ("TinyG 3-axis")** mirrors the grblHAL
  generic file: a typical small router (300 x 300 x 100 mm written `-L..L`,
  1000-24000 rpm, Flood M8 / Mist M7 - TinyG notes the two share one output
  pin), no `safe_retract` (no homing assumed), no tool count, no power.
- **Registered as "TinyG"** after "grblHAL" in `BUILTINS`, before the lathe
  post; extension `.gcode`, the ending TinyG's own sample files use. The
  synth golden drops only the tap record, as grblHAL's does.

**Cost to change later:** low - a length-offset or spindle-dwell post option
adds words without moving a golden; a 4-axis variant follows #714's path.
