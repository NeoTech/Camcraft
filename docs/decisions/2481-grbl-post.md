## #2481 - Grbl 1.1 post (3-axis mill)

Sources: the public Grbl 1.1 wiki pages "Commands" and "Interface"
(supported G and M words, the serial line limit, the M0 program pause).
Behaviour only. The script is derived from our own `grblhal.cps`. No
vendor post or vendor output was consulted.

- **Decided: drill cycles post as G0/G1/G4 moves, never G73/G81-G89.**
  **Why:** Grbl 1.1 has no canned cycles. The engine already sends the
  expanded hole moves next to the CYCLE record, so the post passes them
  through, as `tinyg.cps` does. A dwell (P > 0) is a G4 at the hole bottom.
  **Dropped:** an expansion in the script - it would repeat the engine's
  expansion and could drift from it.
- **Decided: BORE and TAP are refused by name.** **Why:** an expanded bore
  needs a spindle stop before the rapid retract, and a tap needs a
  synchronised spindle. Grbl 1.1 has neither as a cycle. A refusal is
  safer than a program that drags the tool out of the bore or breaks a tap.
- **Decided: tool change is M0 with comments, no M6, no G43/G49.** The
  first tool gets `(LOAD T<n> AND SET Z ZERO BEFORE THE START)`. Each
  later tool gets a retract, `(TOOL CHANGE - LOAD T<n> - SET Z ZERO -
  RESUME)` and `M0`. **Why:** Grbl 1.1 ignores M6 and has no tool table;
  the operator sets Z again after each change.
- **Decided: a line is at most 79 characters.** Comments are cut to fit.
  A motion line that does not fit refuses the program. **Why:** the Grbl
  1.1 serial line buffer is 80 bytes with the terminator; a longer line is
  rejected by the control. Cutting a motion word is never safe.
- **Decided: no G41/G42 control compensation.** Grbl 1.1 has no cutter
  radius compensation, so an op set to control or wear compensation is
  refused, as on TinyG.
- **Machine:** `grbl-3axis.mch` is a copy of `grblhal-3axis.mch` with the
  model "Grbl 3-axis". The user edits the limits for the real machine.

Not exercised: no reading on a real Grbl 1.1 control. The pinning tests
check the goldens and the dialect (`grbl_dialect.rs`: line length and
banned words) only.

Cost to change later: low. The script and the machine are two assets and
one registry line each.
