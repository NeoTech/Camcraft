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
  I<peak - drive line, diameter> J<2 x first depth> K<2 x height> R1 Q0
  H<spring passes>` after a G0 to the drive line (word order and grid
  law below). Only what the record
  states exactly is written: equal rungs (R1 = constant depth, the
  kernel's ladder), radial infeed (Q0), no finish allowance (G76 has no
  word for it), no pull-out (no E/L), an external thread from +X.
  Everything else is refused by name ("G76 threading cycle refused - ...").
  The #516/#517/#518 infeed and repeat-pass records are not on main; when
  they land, each needs its own mapping or refusal here.
- **The merged lathe fixes (#629 hand, #630/#641 control point, #632
  facing).** The post declares `threadHands = true` and writes M3, or
  M4 under `PARAM SpindleDirection reverse` with an operator note to
  mount the insert for reverse rotation (Tormach's M-code table lists
  M4 for the lathe). Each G33 half-turn's arc sense is checked against
  the section's spindle - clockwise seen from +Z is M3's - and a
  mismatch is refused by name ("would cut the other hand"), as on Fanuc
  Lathe. G33 and G76 both cut from where the tool stands toward their Z
  word (LinuxCNC G33/G76), so the stream's start-at-the-top, end-toward-
  the-chuck direction posts as is; a G76 record whose hand and Z
  direction disagree with the spindle is refused. The stream point is
  the tool's control point (the app moves a nose-radiused insert to its
  imaginary tip for its orientation, a blade to its corner); the post
  shifts nothing and writes `ORIENTATION n` in the tool note so the
  operator can match the PathPilot tool table's Q. Golden shift for the
  r0.8 orientation-2 insert: every point X-1.6 (diameter) Z-0.8 - the
  finish now stands on the Ø7 neck and the Z2 shoulder.
- **G33 K: one number per thread** (Hub-kit finding). K was each merged
  block's own lead, and a lead off the half-micron lattice (12 TPI:
  half lead 1.058333 mm) made each half-turn's rounded Z travel differ,
  so a pass split into several G33 blocks with different K (15 blocks
  for 3 passes before). Now K is the THREAD record's lead (six
  decimals), half-turns merge within a 2.5 µm lead allowance (two
  lattice roundings), and a half-turn more than that off the record's
  lead is refused. A stream with no record takes the pass's whole Z
  travel over its turns. Dropped: widening the merge tolerance alone
  (K would still be one rounded block's).
- **G76 on the control's grid** (Hub-kit finding). The record's
  diameters and height ride the stream lattice one by one, so the
  exact `major - minor = 2 x height` check refused a legal cycle on a
  1-2 µm mismatch. Now the checks allow 2.5 µm on a diameter (beyond
  that is a real disagreement, still refused), and I J K are computed
  once as whole grid steps (G21 0.001 mm, G20 0.0001 in): X = the drive
  line, I = peak - X, K = peak - root, with the root rounded OUTWARD
  (never inside the minor diameter - a deeper root gouges the thread
  form) and J rounded shallower, capped at K. So X + I and X + I - K
  are exactly the grid numbers chosen, never independent roundings
  (the inch golden's I-0.0039 / K0.0945 became I-0.004 / K0.0944:
  root Ø0.378 in, 1.2 µm shallow of Ø9.6 instead of 1.3 µm deep).
- **G76 word set** (LinuxCNC G-code reference, "G76 Threading Cycle",
  linuxcnc.org/docs/html/gcode/g-code.html#gcode:g76; Tormach's 8L
  programming page, G76 section, same words, "the values for I, J and
  K are diameter measurements" under G7, rear-tool-post signs). Words
  in PathPilot's conversational order `G76 P Z I J K R Q H` (the 8L
  v2.14.4 sample's `P Z I J K R2 Q30 D0.0000`, read for word order and
  syntax only). **D is not written**: neither the LinuxCNC G76 section
  nor Tormach's G76 table defines a D word (D appears there only for
  G41/G42 and G96), so its meaning cannot be pinned from public
  documentation and the sample's value is zero; G76 without it is the
  documented form. **R and Q stay the record's (R1, Q0), not the
  sample's R2 Q30.** The kernel cuts and simulates equal radial rungs
  (min depth = first depth, infeed angle 0): R1 is LinuxCNC's "constant
  depth", Q0 radial - the passes the control cuts are the passes the
  stock simulation showed. R2 would change the pass count (J x
  sqrt(n)) and Q30 offsets each pass along the drive line by an amount
  the LinuxCNC page does not tie to the end Z - a pass could run past
  the stated end toward a shoulder, unsimulated. Both are owed to the
  threading stack's infeed work (#516-#518): when the op states a
  compound angle or a degressive ladder, the record carries it and this
  post maps it (Q from the angle once the end-Z law is pinned on the
  Hub; R from the ladder). H = the record's spring passes (H0 here).
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
`crates/app/tests/golden/nc/tormach-lathe/` lock every word (suites
`lathe_post_tormach` and `lathe_post_tormach_thread`, fixtures shared
in `crates/app/tests/tormach_lathe_support/`); a change is
a regenerate plus a Hub run. Still owed by the issue's Done-means: the
contributor's PathPilot Hub (8L/15L simulator) run of a turning and a
threading program.
