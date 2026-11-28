## #3192 - FluidNC post and a generic 3040 router machine

**What was decided.** A built-in post `FluidNC` (`assets/posts/fluidnc.cps`)
and a built-in machine `3040 router (FluidNC)`
(`assets/machines/fluidnc-3040.mch`) ship. The post is `grbl.cps` (#2481)
with the FluidNC differences below. Its header writes
`(FLUIDNC POST - NOT VERIFIED ON A MACHINE)` on the second line. That line
stays until the owner (Discord user PR-FPV, PiBot board) reports a clean dry
run.

**Sources.** Only FluidNC's public wiki (wiki.fluidnc.com) for behaviour,
and RS-274 / ISO 6983 for the shared words. The owner's own AI-made post
(made from a Vectric post) was not read. No other CAM post or output was
read. The wiki pages could not be fetched in the session that wrote this
post (no network permission). Each point below names the wiki page that
states it. The reviewer or the owner must check each point against that
page before the "not verified" line is removed.

| Decision | FluidNC wiki page | Status |
|---|---|---|
| Start line `G21`/`G20 G17 G90 G91.1 G94 G40 G80`: each word is in the list | "Supported G-Codes" | From the list; check `G91.1` |
| No canned cycles: drilling is G0 / G1 moves and `G4 P<seconds>` | "Supported G-Codes" (only `G80` of the cycle group) | Check the `G4 P` unit (seconds, as Grbl) |
| Tapping (`G84`) and boring (`G86`) are refused by name | "Supported G-Codes" | Same as Grbl 1.1 |
| No `G41`/`G42`: control compensation is refused by name | "Supported G-Codes" (`G40` only) | Same as Grbl 1.1 |
| Tool change is `M5`, a Z lift, a comment, then `M0`. No `M6`, no `T` word | "Supported M-Codes"; the tool change / ATC configuration page | See below |
| No `G43.1` / `G49` from the post: the operator or the sender sets the length offset | "Supported G-Codes" | Same as Grbl 1.1 |
| Spindle `S.. M3`, `M5`; no dwell after `M3` | "Supported M-Codes"; the spindle configuration page (`spinup_ms`) | The control waits `spinup_ms` when the YAML states it |
| Coolant `M7` / `M8` / `M9` from the machine file only | "Supported M-Codes"; the coolant configuration page | The 3040 states no coolant, so nothing is written |
| Probing (`G38.x`) posts nothing: the probe hook is not defined | "Supported G-Codes" | Same as Grbl 1.1: the engine refuses a probe op by name |
| Lines are 79 characters or less | The wiki does not state a line bound | Conservative: Grbl 1.1's bound |
| No `G28` / `G30` home; lift is `G53 G0 Z<safe_retract>`, or a work-offset `G0 Z` when the machine states no `safe_retract` | "Supported G-Codes"; homing configuration page | The 3040 states no `safe_retract` (no homing assumed) |

**Why M0 and not M6.** FluidNC's `M6` acts only through a tool changer or a
tool change macro in the machine's YAML configuration. The post cannot see
that file. A 3040 router has no tool changer, and the owner's YAML is not
known. `M0` is a stop on every FluidNC machine. The comment before it names
the tool and asks for the Z zero again.

**Why a copy of grbl.cps.** FluidNC is grbl-derived. Each word that
`grbl.cps` writes is in FluidNC's lists with the same meaning. A copy keeps
the safety laws of `grbl.cps` (first-traverse height, spindle-stopped
refusal, lift refusal, same-tool rule #3445) without new logic. The engine
has no include, so the file is a copy. The goldens differ from the Grbl
goldens only in the header line and in the post's name in refusals.

**The 3040 machine.** Typical values for the class, not a maker's sheet. The
`description` says so and asks the owner to confirm each value: X 300, Y 400,
Z 70 mm travel; an 800 W VFD spindle, 6000-24000 rpm; feed 2000 mm/min X/Y,
800 mm/min Z; rapid 2500 / 1000 mm/min; no coolant; no tool changer; no
`safe_retract`. The file names `FluidNC` as its post (`controller.post`).

**Dropped.**
- A post option to write `T<n> M6` for a FluidNC with a tool changer or a
  tool change macro. Dropped until an owner with such a configuration can
  confirm the sequence. A follow-up can add it as an opt-in option.
- A longer line bound (FluidNC may take more than 80 characters). Dropped:
  the docs do not state it, and a shorter line is never wrong.
- `(MSG, ...)` at the tool change stop. Dropped: the plain comment is the
  Grbl form, and the owner can ask for it after the dry run.
- Reuse of `grbl.cps` with a name alias. Dropped: the header must say "not
  verified", and the refusals must name FluidNC.

**Cost to change later.** A dialect change in `fluidnc.cps` moves its own
goldens only (`nc/fluidnc/`, `synth_fluidnc.nc`,
`crates/post/tests/recorded/golden/fluidnc/`). A change to the machine file
moves no golden.
