## #1399 - a post states its own word meanings for Explain G-code

- **Chosen:** two optional manifest globals, read by the PRELUDE's
  `explain()` only when Explain is on. `explainWords` maps a G / M word
  to the meaning it has on that control: a text, `""` to leave the word
  unexplained, or `{ said, takes }` for a word whose meaning needs another
  word of the block (Fanuc lathe `G50 S`: the spindle speed limit) - said
  only beside that word, with its value folded in, and silent without it
  (`G50 X Z`, the coordinate setting, is never given the limit's meaning).
  `explainTool = "station-offset"` reads a T word as tool number then a
  two-digit offset number (T0101: tool 1, offset 1; T0100: no offset);
  any other T word on such a post is left unexplained. `fanuc-lathe.cps`
  states G98 / G99 as the feed modes, G50 with S, and the T reading;
  `tormach-lathe.cps` states the T reading (it writes G95, not G98/G99).
- **Why:** the issue's root cause is one table keyed by word with no way
  for a control to say what a word means there. A per-post table keeps
  the one PRELUDE helper (its guards stay in one place) and puts each
  control's reading in the post that already documents its dialect. The
  meanings come from RS-274 / ISO 6983 and the controls' publicly
  documented lathe vocabulary (Fanuc 0i-T G-code system A, the LinuxCNC /
  PathPilot lathe T word), as the posts' own provenance lines state.
- **Refusal:** a malformed table (`explainTool` other than
  `"station-offset"`, `explainWords` not an object, a key that is not a
  G / M word without leading zeros, a value that is neither a text nor a
  `{ said, takes }` with one value letter) is refused by name before any
  hook runs - only when Explain is on, so a program posted with Explain
  off never reads it and stays byte-identical.
- **Dropped:** a separate lathe table chosen by `capabilities` (a turning
  bit does not say which lathe dialect, and mill-turn posts mix both);
  `""`-only skips for G50 and the T word (the issue asks that the right
  meaning be said where one is known, and a skip loses tool 1 / offset 1).
- **Not done here:** the lathe posts still declare no `explains` form and
  their `block()` does not call `explain()` - that wiring is #1387 /
  #1388. The pinning test opts fanuc-lathe in with a test shim.
