## #786 - Rigid tapping as a machine capability, refused at the program gate

**What was decided.**

- **Explicit, not default-false.** `spindle.rigid_tapping` stays optional:
  absent is "not limited" (the unstated-limit law every other `.mch` key
  follows), and every Tormach mill and router file states it. The MX
  mills (770MX, 770MX ML, 1100MX, 1500MX) state `true`; the R8 M and M+
  mills, the PCNC series and the routers (24R, xsTECH, xsTECH Pro) state
  `false`. These values were already in the files (#170, #618, #620);
  this PR pins them all in one test
  (`every_tormach_mill_and_router_file_states_rigid_tapping`).
- **The refusal moved up to the program gate, beside the post's own.**
  The machine check (`machine::check_ops`, the gate every post path
  runs) now refuses a path carrying a tap cycle on a machine that states
  `false`, naming the op, the count and the machine. Before, only
  `tormach.cps` read `machine.rigidTapping`, so a 770M with the Fanuc or
  Haas post posted a G84 with no warning. `tormach.cps` keeps its own
  refusal for the standalone script path. The cycle is never converted to
  another cycle; the message points at a tapping head or thread milling.
- **The schema is validated.** A stated value that isn't `true`,
  `false` or `null` refuses the file by name at both doors: the reader
  (`parse_mch`: "spindle.rigid_tapping must be true or false") and the
  Machines pane's validator (key `spindle.rigid_tapping`). Before, a
  string like `"no"` was read as unstated, which means "not limited". A
  capability that gates G84 shouldn't fail open.
- **The pane says what it does.** The Spindle group's "Rigid tapping"
  row reads `yes`, `no - tap cycles are refused`, or
  `not stated - not limited`, instead of a bare JSON boolean.

**Why.** The issue asks for a refusal by name, never a silent G84, and
for other posts and machines to be unaffected. Putting the gate in the
machine check covers every post with one rule. It affects only machines
that state `false`, and only Tormach files do that today. The 770MX
G84 golden is untouched, because the gate is silent on `true` and on
unstated.

**Considered and dropped.**

- *Default false, MX files set true.* That would refuse taps on every
  non-Tormach built-in (VMC, tilt-head, mill-turn) and on every user
  file that says nothing, changing their output. The issue lists "other
  machines unaffected" as a goal, and the unstated-limit law holds across
  the whole `.mch` grammar.
- *Converting a refused tap to a peck or a floating-holder G84 variant.*
  The issue forbids silent conversion. A tapping-head program is a
  different operation the user picks.
- *Stating it on the Tormach lathes (8L, 15L and 15LX Slant-PRO).* The
  public lathe material in hand doesn't settle it. PathPilot release
  notes mention a G84 tapping cycle, but no manual page found here ties
  it to a lathe model and spindle. Stating either value without a source
  would be a claim the clean-room rule can't back. The lathe files stay
  unstated (unchanged behaviour), and #795 tracks stating it
  from the lathe manuals. The 1300PL plasma table can't state a spindle
  key at all (#669's plasma rules).

**Cost to change later.** Making it default-false is one line in the tap
gate plus a `true` in every built-in that can tap; the gate and its tests
stay as they are.
