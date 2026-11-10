## #612 - a Tormach PathPilot (LinuxCNC) lathe post

**Sources (clean room, behavior only).** The LinuxCNC G-code reference
(linuxcnc.org/docs/html/gcode/g-code.html: G7/G8, G18, G33, G33.1, G76,
G95, G96 D/S, G97, G53, G28/G30, G64, G81-G83 plane rule), the LinuxCNC
"Lathe User Information" page (linuxcnc.org/docs/html/lathe/lathe-user.html:
under G7 X is a diameter but the arc's I offset stays a radius), and
Tormach's public PathPilot lathe programming pages
(knowledgebase.tormach.com/15l/programming-15l, and the 8L pages: T word,
no M6 on a lathe, G7 only / G8 unsupported, G18 only, G20/G21 before
motion, supported G/M tables, G76 I J K diameter values, G30 before tool
changes). No vendor post text and no CAM output read.

**What was decided.**

- **Dialect.** Header `G7 G18 G20|G21 G90 G91.1 G40 G95` and
  `G64 P0.001|P0.025`, units from `machine.units`. No `%` fences, no O
  word (LinuxCNC reads O as a subroutine label - the Tormach mill post's
  law). Comments parenthesized and cleaned of `( ) ; %` (no nested
  comment in LinuxCNC); a WARN comment is cleaned the same way.
- **Feeds and spindle.** G95 per-rev feed for every cut, F = mm/min / rpm
  (inch: / 25.4, five decimals). G97 S M3, or G96 D<machine max rpm>
  S<surface speed> M3 when the stream states a SurfaceSpeed (m/min under
  G21, ft/min under G20 - the documented G96 units). No G94 is ever
  needed: no live tooling is declared, so C-axis sections are refused by
  the engine by name.
- **Tool change.** `G30` then `T<tt><tt>` (T0101). Tormach's lathe page:
  "no M6 tool change command for lathes"; the T word's first pair picks
  the tool and geometry offset, the second the wear line; T > 99 is an
  error there, so tool numbers outside 1-99 are refused by name. G30 is
  Tormach's documented "safe place before tool changes" (the operator
  presets it). Also used for Go home (#110) and at program end, then M30.
- **Arcs.** G2/G3 X(diameter) Z I(radius increment) K, the Fanuc lathe
  post's view-from-+Y handedness. Chosen over R-format because the
  LinuxCNC lathe page states I's meaning under G7 explicitly.
- **Threading default: G33 passes.** `G33 Z K<lead>` (K = distance per
  revolution, six decimals so an inch TPI lead is not rounded away). The
  kernel's half-turn arcs of one pass are merged into ONE G33 per pass so
  synchronization starts once per pass on the spindle index - dropped:
  one G33 per half-turn (the Fanuc G32 shape), since the docs do not say
  a following G33 keeps the index lock.
- **Threading option "G76 cycle": one LinuxCNC block** `G76 P<lead> Z<end>
  I<peak - drive line, diameter> J<2 x first depth> R1 K<2 x height> Q0
  H<spring passes>` after a G0 to the drive line. Only what the record
  states exactly is written: equal rungs (R1 = constant depth, the
  kernel's ladder), radial infeed (Q0), no finish allowance (G76 has no
  word for it), no pull-out (no E/L), an external thread from +X.
  Everything else is refused by name ("G76 threading cycle refused - ...").
  The #516/#517/#518 infeed and repeat-pass records are not on main; when
  they land, each needs its own mapping or refusal here.
- **Drilling on centre: plain moves.** PathPilot's lathe tables list no
  G80/G81/G83, and LinuxCNC's cycles under G18 drill along Y. The stream's
  own expanded hole chain posts as G0/G1 on X0, with `G4 P<seconds>` at
  the floor when the cycle asks a dwell. Tapping refused by name (G33.1
  is documented but not written yet - follow-up), back-bore and off-axis
  holes refused.
- **Control nose-radius comp** refused by name, as on Fanuc Lathe.
- **Registry.** `("Tormach PathPilot Lathe", ...)` in `BUILTINS`, listed in
  `LATHE_POSTS` (second, so `default_post_for(Lathe)` stays Fanuc Lathe).
  The UI threading picker now shows for both lathe posts (labels "G33
  passes" / "G76 cycle" on this one).
- **Part load / unload** stay comments (the PRELUDE's intent law), the
  advance in the program's own unit.

**Considered and dropped.** G8 radius mode (unsupported in PathPilot);
G53 G0 X/Z retracts (the lathe's machine-coordinate home direction is
not stated in any public Tormach page we read - G30 is the documented
safe spot); G28 for home (LinuxCNC warns it needs a repeatable home and
it is not Tormach's advice); G81/G83 (not in the PathPilot lathe tables).

**Cost to change later.** Goldens under
`crates/app/tests/golden/nc/tormach-lathe/` lock every word; a change is
a regenerate plus a Hub run. Still owed by the issue's Done-means: the
contributor's PathPilot Hub (8L/15L simulator) run of a turning and a
threading program.
