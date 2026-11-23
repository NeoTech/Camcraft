## #1914 - RichAuto DSP A11 3-axis router post (`richauto.cps`)

**What was decided.**

- Added the clean-room post `assets/posts/richauto.cps`. It is registered as
  `"RichAuto DSP A11"` in `BUILTINS` (`crates/app/src/posting_files.rs`), after
  Mach4 and before the non-mill posts.
- The block format follows the program that the friend's Laguna Swift A11 runs
  (#1907, comment of 2026-11-18). The maintainer named that program as the
  conformance target on #1914:
  - extension `.mmg`;
  - a sequence number on each block, `N10` and up in steps of 10, no wrap;
  - no spaces between words;
  - lengths with three decimals, `F` in mm/min with one decimal;
  - `G00` rapid and `G1` feed; `M03S<rpm>` to start the spindle, `M05`, `M30`;
  - modal axis and feed words are left out when they do not change;
  - the program ends with a `%` line. Comment lines have no `N` word.
- The setup block is `G21G90G17` only. The sample program has no setup block.
  G21 and G90 stay because the issue asks for them and they guard against an
  inch or incremental controller state. G17 stays because the post writes G18
  and G19 on change.
- No `G54` is written. Offset 1 is the work origin that the operator zeroed on
  the pendant. The sample program writes none, so the pendant's `Read G54` /
  `Ign G54` setting cannot move the origin. Offsets 2 and up are refused by name
  (`workOffsetMax = 1`).
- Arcs use incremental `I`/`J`/`K` (`AbsCntr = Off`). An arc block always writes
  X, Y and Z. Arcs above 180 degrees are split.
- Kept from the held branch (`mkinney:issue/1914-richauto`, 9fb271c2d): one tool
  per program, drilling expanded to plain moves (floor dwell, bore spindle stop
  and restart), the `SpindleDwellMs` option (`G04P<ms>`), metric only
  (`programUnits = "mm"`, `endUnits = "note"`), and named refusals for tapping,
  G41/G42, probing, rotary moves, G96, a second tool, a bad coolant, a bad rpm
  and a missing or low clearance plane.
- Added `onComment`: header notes (#2996) and WARN records are written as
  `(...)` comments with the delimiters removed.

**Why.**

- A program that the real controller runs is stronger evidence than a general
  RichAuto manual. Each word that the sample proves is used as it is.
- Leaving out `G54`, `G40` and `G80` removes words that a basic router firmware
  can reject, and nothing in the program engages compensation or a canned cycle.

**Considered and dropped.**

- The held branch format (`G21 G90 G17 G40 G80`, `G54`, spaces, no `N`, no `%`,
  `.nc`): dropped, because it does not match the program that the A11 runs.
- No setup block at all (an exact copy of the sample): dropped, because the issue
  asks for G21 and G90 and they cost nothing on an A11.
- `axisInversion = true`: not declared. A machine with an inverted axis is then
  refused by name by the engine, which is safe. Declaring it needs the
  `wcs_flip_dialects` trace verification. Grbl 1.1 also does not declare it.
- CRLF line ends (the sample has CRLF): not written. The posts write LF; the
  A11 behaviour with LF is not confirmed.

**Cost to change later.**

- Local to `assets/posts/richauto.cps`, its `BUILTINS` row and its goldens
  (`crates/app/tests/golden/nc/richauto/`, `synth_richauto.nc`,
  `crates/post/tests/recorded/golden/richauto/`). An air cut on the friend's
  machine can still change the setup block, CRLF or the extension.
