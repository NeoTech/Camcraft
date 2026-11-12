## #1174 - The workholding rpm limit judges the work spindle, from the post's own derivation

**What was decided.** `setup_workholding::program_problems` judges each
operation by what the lathe post will do with its section, read through
one function the stream itself uses: `post::section_spindle::of(path)`.
`post::dump` writes `PARAM LiveTool` (and the A/B pose words) from the
same function, so the gate and the stream can never disagree. No list of
operation kinds is consulted (the first version of this PR fell back to
`spindle_law::turns`, which counts an on-centre Drill only on a pure
lathe - a mill-turn's on-centre drill at 8000 rpm posted `G97 S8000 M03`
under a 2000 rpm chuck; the review caught it).

On a setup whose machine is a lathe or a mill-turn:

| `section_spindle::of` | What `fanuc-lathe.cps` writes | Spindle the S word drives | What the chuck limit judges |
|---|---|---|---|
| `Work` - no move states C or a leaning axis (every turning kind, the on-centre Drill / Bore / Thread / tapping cycles, Probe, any mill path on the meridian) | `G97 S<rpm> M03` / `G50 S` + `G96` - whatever the kind | the WORK spindle | G97: the stated rpm, refused by name above the limit (at it is allowed); G96: the clamp, min(machine, tool, chuck) - unchanged |
| `LiveTool` - a move states C (`PARAM LiveTool`: the C-axis rotary kinds) | `G98`, the C-on word, the live-spindle word `S<rpm>`; no `M03` | the LIVE tool; the work is in C mode, fed as an axis | the machine's C rate: the larger of its stated C feed and C rapid (deg/min) / 360. No C rate stated -> refused by name while a limit is stated |
| `Posed` - a non-C move states a leaning tool axis (the cross drill's radial plunges, the 5-axis kinds) | A/B words, refused by both lathe posts ("posed move ... no rotary to command") | - (the post refuses it) | nothing - the post's refusal is the guard |

On a mill (or any other machine kind) the spindle turns the tool: the
limit judges nothing there.

**Why.** A chuck's rating limits the spindle that holds the work, and
which spindle that is in a section is decided by the post, not by the
operation's kind: the lathe post opens every non-live section on the
work spindle. Judging from the same derivation the stream uses closes
both directions - the false refusal of #1174 (a 10000 rpm live cross
drill under a 100 rpm chuck) and the fail-open the review found (any
kind `turns()` did not name posted the chuck over its rating).

**Considered and dropped.**
- A second list of kinds (extend `turns()` with Bore, Thread, ...): it
  can drift from the post again; the moves are what the post reads.
- Judging a C section by the speed C actually reaches per move: exact,
  but a second copy of the post's composite-feed arithmetic. The stated C
  rate is the ceiling the post and the control hold C to, so it is the
  stricter reading.
- Letting an unrated C pass: a limit that is skipped is a hazard.
- Judging a `Posed` section at its rpm: the cross drill's S is a driven
  radial tool's (the work is held still); no shipped lathe post voices a
  posed section at all. If a post ever does, this row needs re-reading.
- Changing any post script: not needed (and #1190 owns the post side).

**Known over-reach.** A mill-turn posted through a MILL post (which
gives S to the tool spindle) is judged as if the lathe post ran: a
refusal that is too wide, never a skipped limit.

**Cost to change later.** `post::section_spindle` and `work_speed`; pins
in `lathe_workholding_live` and `lathe_workholding_kinds`.
