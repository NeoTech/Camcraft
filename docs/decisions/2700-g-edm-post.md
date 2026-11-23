## #2700 - G-EDM 2-axis wire EDM post (g-edm.cps)

**What was decided.** `assets/posts/g-edm.cps` ("G-EDM Wire EDM") is the
third `WIRE_POSTS` entry. "Generic ISO Wire EDM" stays the default. The
post writes Grbl-family words only: `G21` / `G20 G90 G17` on one line,
`G0` to the threading point, `G1`, `G2` / `G3` with incremental `I J`, and
`M30`. It writes no `%` fence, no `O` word, no `G40` / `G41` / `G42`, no
`F`, no `S`, no `Z`, no `U V`, no `M0` / `M1`, and no return move. It reads
no `edm.codes`. Each condition id is written in a comment only. The header
says that the spark power is set at the G-EDM controller, that the feed is
set by the gap servo, that the output is not verified on a control, and
that the wire stays threaded at the end.

Sources: the public Grbl_Esp32 G-code documentation, and the public G-EDM
pages (gedm.org, github.com/G-EDM/G-RBL) as summarised on #2697. No
G-RBLv3 source file was read. The output was not run on a G-EDM
controller. #2697 has no new answer from the reporter at the time of this
PR.

**Refused by name** (no cut is written for the section):

- a machine that is not `wire_edm`, a machine with a rotary, and a machine
  with `edm.awt: true` (no G-code word threads or cuts the wire);
- `WireTaper` not 0, `WireFourAxis`, `WireU` / `WireV`, `WireAngle`;
- each `COMP` record (G41 / G42, wear and OFF included);
- a skim pass (a second `EdmCondition` while the wire is threaded);
- a work offset other than `G54`;
- `SpindleSpeed` above 0, a tool change, a canned cycle, a `Z` change;
- `AfterOperation` `home`, `stop` and `optional-stop`;
- a second wire section in the same program.

**Why.**

- No `F` word. gedm.org states that wire mode ignores `F` and that the gap
  servo sets the rate. Stock Grbl gives `error:22` on a `G1` with no modal
  feed. If G-RBLv3 keeps that check, the controller stops at the first
  `G1`, before any cut. That is a visible stop, not a wrong cut. A dummy
  `F` value could be read as a real feed limit on a build that reads it.
- No work offset word. `G54` is the Grbl default after a reset, and G-RBLv3
  support for `G55`-`G59` is not confirmed. A setup with another offset is
  refused, not posted into the wrong offset.
- No `G40`. Grbl_Esp32 reads it as a no-op, but G-RBLv3 is not confirmed.
  The post never engages compensation, so there is nothing to cancel.
- One wire section per program. Without a confirmed `M0`, the program
  cannot stop to cut the wire and thread it at the next start point. A
  `G0` to a second start point would pull the threaded wire through the
  work.
- The operator threads the wire at the threading point before the start.
  The program keeps its `G0` to that point. If the wire is at that point,
  the move has zero length. If it is not, the program moves to the correct
  point instead of cutting from the wrong place.
- `stop` and `optional-stop` are refused because `M0` / `M1` are not
  confirmed (#2703 owns confirmed M-codes and pauses).
- Explain G-code is not declared, so the engine refuses it by name.

**Dropped.**

- Writing `G54`, `G40` or a dummy `F`: each is unconfirmed on G-RBLv3.
- An `M0` threading pause, skims with `M0` power-change pauses, and a final
  return to `X0 Y0`: these wait for the reporter's answers (#2703).
- Taper and 4-axis `U V` output: #2702.
- A machine template: #2701.

**Cost to change later.** Each dropped item is a local change in
`g-edm.cps` plus its golden. The registry name "G-EDM Wire EDM" is stored
in project files: a rename needs a migration.
