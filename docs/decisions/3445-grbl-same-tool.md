## #3445 - grbl-family posts keep a tool that the program already loaded

**What was decided.** `grblhal.cps`, `grbl.cps` and `tinyg.cps` remember the
tool that the program last loaded (`loadedTool`), as `tormach.cps` does
since #1626. When the next section asks for the same tool, the post writes
no tool change:

- grblHAL: no `T<n> M6`, and no optional `M1` before it.
- TinyG: no `M5` / `M9` / `T<n> M6` / `M0` block, and no frame restate after it.
- Grbl 1.1: no `TOOL CHANGE - LOAD` comment and no `M0` stop.

The tool comment stays. A program stop or an optional stop after an
operation (`AfterOperation`) sets the loaded tool to unknown, because the
operator can take the tool out at a stop. The tool at program start is
unknown, so the first section always changes (or names) its tool.

**Why.** On a manual-change router, each `M6` (grblHAL, TinyG) or `M0`
(Grbl) halts the program. A repeated tool made the operator resume three
extra times on the q07 router job, and on grblHAL it can start a tool
length probe each time.

**What was kept, and why.**

- The Z lift before the section (`G53 G0 Z<safe>` or the work-offset
  clearance) stays. The issue lists it as unwanted, but it costs only
  time, and it keeps the next section's first traverse safe when the
  next ClearancePlane is not proven above the previous section's
  geometry. Tormach also keeps its lift.
- The section-end `M5` / `M9` and the next `S.. M3` / coolant word stay.
  The issue asks for no `M5`. A deferred `M5` touches the spindle-on cut
  rule (#1357), the probe rule and the after-operation stops, which need
  a stopped spindle. A spindle stop and start does not halt a grbl
  program. Tormach also keeps them.
- `X/Y` stay unknown after a kept tool, so the first traverse still comes
  after a Z-only rapid to the section's clearance (#1616).

**Dropped.** Skipping the lift and the spindle words too (the full ask of
the issue). It can come later as a separate change with its own rule for
when Z and the spindle may stay; it is a change to two lines in each post.
