## #1174 - The workholding rpm limit judges the work spindle, not a live tool's speed

**What was decided.** `setup_workholding::program_problems` sorts each
operation by which spindle its section's stated speed drives, read from
what the lathe post does with the section (not from kind names):

| Section | What `fanuc-lathe.cps` writes | Spindle the S word drives | What the chuck limit judges |
|---|---|---|---|
| Turning family (turn, face, groove, part, thread, ...) | `G97 S<rpm> M03` / `G50 S` + `G96` | the WORK spindle | G97: the stated rpm (refused above the limit); G96: the clamp, min(machine, tool, chuck) - unchanged |
| Lathe Drill, axial holes (on centre) | `G97 S<rpm> M03` + G83 face drilling | the WORK spindle (drill held still in the turret) | the stated rpm, as before |
| Any section whose moves state a C angle (`PARAM LiveTool`: the C-axis rotary kinds on a mill-turn) | `G98`, the machine's C-on word, the live-spindle word `S<rpm>`; no `M03` | the LIVE tool; the work is in C mode, fed as an axis | the machine's C rate: the larger of its stated C feed and C rapid (deg/min) / 360, the most the work can turn in the section. A machine stating no C rate is refused by name while a limit is stated |
| Drill with cross (radial) holes, lathe or mill-turn | a posed section (B 90, C fixed); no shipped lathe post voices it (`fanuc-lathe.cps` refuses posed moves) | the driven radial tool; the work is held in place (no C move, fixed pose) | nothing |
| Everything else (`spindle_law::turns` false, no C) | not judged before, not judged now | - | nothing |

**Why.** A chuck's rating limits the spindle that holds it. The old check
used `spindle_law::turns` ("works about the spindle axis", the axis law's
question), which also counts the cross drill, so a 10000 rpm live drill
was refused under a 100 rpm chuck with the false text "turns the work at
10000 rpm". The C-axis rotary kinds were not judged at all, although the
work does turn (slowly) during them; judging them by the C rate closes
that without touching the post scripts.

**Considered and dropped.**
- Judging a C section by the speed C actually reaches per move (chord,
  feed, cap): exact, but it repeats the post's composite-feed arithmetic
  in the app. The machine's stated C rate is the ceiling the post and the
  control hold C to (G01 capped by it, G00 C at the rapid), so it is the
  stricter reading and needs no second copy of the post's math.
- Letting an unrated C pass: its speed cannot be held to the limit, and
  a limit that is skipped is a hazard; refused by name instead (only
  while a limit is stated - `None` posts exactly as before).
- Changing any post script: not needed; the rule is a program-gate rule.

**Open.** A Mill-family operation on a mill-turn whose path states no C
(no `LiveTool` param) would be opened by the lathe post with `G97 S M03`,
the work spindle. It was not judged before and is not judged now; it is
filed as #1190.

**Cost to change later.** One function (`work_speed`) and its pins in
`lathe_workholding_live`.
