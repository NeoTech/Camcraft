## #1594 - Tormach mill programs end at the top of Z, no trailing G30

**Decided:** `tormach.cps` ends a program `M9` (when coolant ran), `M5`,
`G53 G0 Z<safe>` (Z0, or the machine's stated safe Z, #495), the `EndUnits`
word if any (#596), `M30`. The G30 that used to follow the final retract is
gone, and no rotary return (`G0 A0`) is added. The G30 before each tool
change is unchanged. The end retract and the tool-change retract are now
two functions (`liftZ` for both, `retract` adds the tool-change G30).

**Why:** the user's PathPilot Hub run (770MX sim, 2026-11-13, stored G30 at
machine Z -1.5611) tried four endings. `G53 G0 Z0` then `G30`, and `G30`
alone, both drove the tool down below the part top at the end (a bare G30
moves every axis to the stored G30 position, LinuxCNC params 5181-5189).
`G53 G0 Z0` alone stayed at the top of travel; the user: "C is the only
right one". Adding `G0 A0` was tried and "Definitely do not want".

**Fix, not breaking:** the posted output changes only in its last lines,
and the change corrects wrong behaviour a user hit (a P3 bug), so the
commit is `fix(post)`. No project-file field, parameter or command changes.

**Considered and dropped:**
- The issue's first request (keep G30, drop the lift) - the Hub run showed
  it slams down whenever the stored G30 is low.
- A post option for the ending - the user picked one ending; an option
  would keep the unsafe one reachable.
- Dropping the tool-change G30 too - out of the issue's scope (PathPilot's
  tool-change convention); the same low-G30 risk there is noted for a
  follow-up.
- Changing `tormach-plasma.cps` / `tormach-lathe.cps` - they end with G30
  by their own conventions (a plasma table and a lathe's X/Z safe spot) and
  the user's test covered the mill only; listed as a follow-up.

**Cost to change later:** one line in `onClose` plus the 33 Tormach mill NC
goldens' last lines.
