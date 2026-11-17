## #1445 slice 2: boring (G86) per post, and the dwell door

- **Chosen:** the second slice of #1445 is the boring cycle (G86: feed
  in, spindle stop, rapid out). The `Bore` cycle kind was already on
  main; this slice pins it per post and closes two dwell gaps.
  **Why:** the slice is small, and it uses the pattern of the ream slice
  (PR #1482). **Dropped:** G82 dwell at depth as the slice. G82 already
  posts on every canned post and the `synth_*` goldens pin it.
- **Chosen:** a bore with a dwell is refused by name on the Fanuc-family
  posts (fanuc, haas, mazak, doosan-dnm) and on sinumerik. **Why:** the
  published Fanuc and Haas G86 formats are `X Y Z R F` (Fanuc also `K`).
  P is a dwell word only on G82/G88/G89 there. The posts wrote `P<ms>`
  on G86, and the control does not read it, so the dwell was dropped
  without a word. The Sinumerik ISO mode is not verified for a G86 P,
  so it refuses the same. **Dropped:** posting G89 for a bore with a
  dwell. G89 feeds out, G86 rapids out: that is a different cycle, and
  #1480 owns G89.
- **Chosen:** tormach and grblhal keep `G86 ... P<seconds>`. **Why:**
  LinuxCNC's G86 takes P as a dwell in seconds. grblHAL follows the
  LinuxCNC form (the post's own comment, from the grblHAL core list).
- **Chosen:** the expanding posts (heidenhain, tinyg, carvera) refuse a
  bore by name: "bore (G86) requires a spindle stop before rapid
  retract - expanded boring is not supported by this post; refused, not
  posted". **Why:** main has this safety refusal. These posts cannot
  stop the spindle before the rapid retract, and the expanded chain
  would pull a turning bar out of the hole. The test pins the refusal
  and no longer pins an expanded chain or a Heidenhain golden.
  The engine warning blocks app posting; rejected intermediate program
  text can still contain fallback moves. The app pin
  `refused_expanded_bores_write_no_nc_file` checks that all three posts,
  with and without a dwell, refuse by name and write no NC file.
  **Dropped:** restoring the expanded bore. A spindle stop at the bottom
  is the follow-up #1819.
- **Chosen:** a drilling dwell is a time from 0 to 3600 s
  (`MAX_DWELL_S`). `CreateOperation` and `EditOperationParams` (and so
  MCP `edit_operation`) refuse other values by name. **Why:** validate
  early (Prime Directive); 3600 s is the bound the Heidenhain post
  already enforces. **Dropped:** a check in `is_sane`/`named_refusal`.
  The project loader asks those, and a project saved with a larger dwell
  must still open. `PasteOperation` copies an operation that is already
  in the document, so it does not ask the new door either.
- Sources: NIST IR 6556 (The NIST RS274NGC Interpreter v3, G86);
  LinuxCNC G-code reference (G86 with P in seconds); the public Fanuc
  30i operator manual (B-63944EN) and Haas mill programming workbook for
  the G86 address words, found by a literature search. No vendor post
  file or CAM output was read.
