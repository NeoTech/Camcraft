## #671 - a PathPilot plasma post for the Tormach 1300PL

Sources (clean room, behaviour only): Tormach's *1300PL Operator's Manual*
UM10720 v1223A - §6.2.1-6.2.2 (the test program and its code breakdown,
pp. 50-51), §6.3-6.3.1 (AutoFS, the hand-stated block, pp. 51-52), §7.1
(G15, G16, p. 54), §7.2 (M200-M213, pp. 54-55) - and Tormach's *PathPilot
Post Processor Guidelines* knowledge-base page (its Plasma column). No
Autodesk/Fusion post, no vendor post script and no CAM output was read.

**1. Every word, from its source.**

| word | meaning | source |
|---|---|---|
| `G20` / `G21` | the unit, from the machine (`machine.units`) | [PG] G20/G21 supported on plasma |
| `G90 G91.1 G94 G17 G40 G49 G80` | absolute, incremental arc centres, feed/min, XY, no comp, no length offset, no cycle - stated once | [PG] each supported on plasma; [UM] §6.2.2 "resets the modal state" |
| `G64 P0.005 Q0.005` | path blending, P in INCHES even on a G21 program | [UM] §6.2.1 (the numbers); [PG] "G64 must be specified in inch units" |
| `G54`.. | the work offset | [UM] §6.2.1 |
| `G43 H1` | the torch's length offset, always H1 | [PG] "For plasma, this should always be set to H1"; [UM] §6.2.2 Tool 1 is the torch |
| `G30` | the operator's G30 height (and position) - after the preamble, after every `M205`, and at the end | [UM] §6.2.1-6.2.2 ("Initial Move", program end) |
| `G0 X Y` | rapid to the pierce point - X and Y only | [UM] §6.2.1 `G0 X0 Y0`; §6.3 the control owns Z |
| `M200` | AutoFS: feed, THC volts, pierce height, pierce delay, amps from the material picked in PathPilot | [UM] §6.3, §7.2 |
| `M207 P` / `M208 P` / `M209 P` | pierce height / cut height (program unit) / pierce delay (s) - written after `M200` only when the op states them | [UM] §6.3.1, §7.2 ("Set by M200: Yes") |
| `G15` | probe down to the plate; the WCS Z is set to 0 there | [UM] §7.1 |
| `G16` | rise to the pierce height, fire, wait the delay, cut height, THC on | [UM] §7.1 |
| `G1` / `G2` / `G3` X Y [I J] [F] | the cut, XY only | [PG] G1/G2/G3 supported; F in/min or mm/min per G20/G21 |
| `G41.1 D` / `G42.1 D` / `G40` | control compensation, D = the kerf (diameter), on its own line ahead of the straight lead | [PG] G41.1/G42.1 supported on plasma (its G42.1 "left side" text is a copy slip) |
| `M205` | torch off, Z back to the G30 height | [UM] §7.2 |
| `M30` | program end; no `%` fence, no `;` | [PG] M30 supported; "no $ or %" |

**2. M200 always, the stated numbers after it.** `M200` also supplies the
THC voltage and the amperage, which the op never states, so it is written
for every contour; a stated pierce height, cut height or delay follows it
as `M207`/`M208`/`M209` (M200 sets all three, so the later word wins).
*Dropped:* the §6.3.1 hand-stated block alone (M210/M211/M207-M209 + F) -
it needs THC volts and amps the op does not have. `M203` (Start Torch) is
not used: `G16` strikes the arc after the probe. `M210`-`M213` never.

**3. No M6.** The manual's test program writes `T1 M6 G43 H1`; the
guidelines list M6 (and T only as M6's argument) as unsupported on
plasma. The post writes `G43 H1` alone - nothing to change on a one-torch
table. *Open, for the Hub run:* does PathPilot plasma accept or need
`T1 M6`?

**4. G30 after every M205.** As the issue specified and the manual's
program end does. M205 already returns Z to the G30 height; whether the
bare `G30` also moves X and Y to the stored G30 position (LinuxCNC's
G30 does) is for the Hub run - a per-contour XY trip home would cost
time, never safety (the torch travels at the G30 height). *Dropped:*
omitting it between contours (keeps the issue's block exact until the
Hub says otherwise).

**5. G41.1/G42.1 D<kerf>, not G41/G42.** The dynamic form states the kerf
the CL was planned with, so the tool table's torch diameter can never
silently disagree with the path; the engage and `G40` cancel each stand on
their own line before the straight lead #670 guarantees (≥ the kerf
radius, PathPilot's entry rule). Wear compensation is refused by name.

**6. Z is the control's.** No Z word is ever written: rapids name only the
next pierce point (a rapid that changes only Z is dropped), and a lit move
that changes Z is refused by name ("a Z cutting move"). The `.cl` cut
plane is the generator's model, not a program word.

**7. Small holes under AutoFS.** With the op's feed blank (`AUTOFEED`, the
default), the lit moves carry no F and the small-hole rule's 65 % cannot
be applied to a feed the program never knows; the hole posts at the
AutoFS feed with a comment saying so (`SMALL HOLE AT THE AUTOFS FEED`).
*Dropped:* refusing it (it would block every default plate with a hole -
a quality loss, not a gouge); reading AutoFS's feed variable (not public).
With a stated feed the CL already carries the reduced F.

**8. The stream carries the torch.** New `.cnc` records `PLASMA ...` and
`TORCH ON|OFF ...`, spelled exactly as the `.cl` v5 records minus the
`; ` - one reader (`cl::parse_*`) takes both, so the committed `.cl`
golden posts as it stands. `dump` writes the header before `SECTION` (a
plasma post knows at `onSection`) and wraps each span around its records,
the ON ahead of a G41/G42 engage and the OFF after the cancel's lead-out.
New hooks `onPlasma`, `onTorchOn`, `onTorchOff`; the engine keeps the torch
law (no torch without a header, no ON while lit, no rapid while lit, no
section left lit) and refuses a script without the torch hooks - which is
how every milling post declines a plasma path. *Dropped:* PARAM-only
headers (a PARAM cannot carry the per-contour spans) and a new
`ENGAGE`-style motion record (the moves stay the torch-centre truth).

**9. The machine says plasma.** `machine.kind` (new script property, the
declared kind's word through either door) must be `plasma`, or the post
refuses by name; a stated rotary, an rpm, a coolant ask other than Off,
a cycle and a tilted move are refused by name too. The 1300PL .mch states
no `safe_retract` - this post never writes G53; G30 is its retract.

**10. Registry.** "Tormach PathPilot Plasma" is appended to `BUILTINS`
(entry 0 stays Fanuc) and is `PLASMA_POSTS`' only entry: offered for a
plasma table (`default_post_for`, the Setup pane's offer) and never for a
mill, lathe or mill-turn; no milling post serves the 1300PL.

**Not exercised:** a PathPilot Hub (1300PL simulator) run and a real cut.
The program must be loaded on the Hub before anyone trusts it on a machine.
