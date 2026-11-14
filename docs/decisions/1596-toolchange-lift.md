## #1596 - Tormach mill lifts Z after each tool change; the plasma and lathe G30s stay

**Decided:** `tormach.cps` writes `G53 G0 Z<safe>` (Z0, or the machine's
stated safe Z, #495, through `liftZ`) right after each tool change's
`G43 H..`, before the section's spindle start and its first X/Y or A move.
The tool-change sequence is now `G53 G0 Z0`, `G30`, `T.. M6`, `G43 H..`,
`G53 G0 Z0`. The G30 before the change stays (PathPilot's convention: the
operator's preset tool-change spot). The lift is written after a
non-G30 retract as well (a machine stating its own safe Z), since M6 may
move Z on the control's side. The 4th-axis index keeps its own lift before
`G0 A..`, so an indexed section has two `G53 G0 Z` lines between `G43` and
`G0 A`; the second is redundant and harmless.

**Why:** M3's review of PR #1600 (pullrequestreview-5900043149) found that
every 3-axis Tormach tool change went `G30`, `T M6`, `G43`, `S M3`, then
`G0 X Y` with no Z lift. With a stored G30 below the part top (the user's
PathPilot Hub sim had machine Z -1.5611), the tool rapids sideways at that
height into the part or the vise - the same cause as #1594. Lifting after
the change makes the result independent of where G30 stores Z. Rated P1.

**The other two bare G30s of the issue - kept:**
- `tormach-plasma.cps` (preamble and `onClose`): on PathPilot plasma the
  G30 height IS the traverse height. Rapids carry no Z, M205 returns the
  torch to the G30 height after each cut, and Tormach's plasma manual
  (UM10720 §6.2.1/§6.2.2) starts and ends programs on G30. The closing G30
  moves the torch at the height it has traversed at all program, so a low
  G30 is no new hazard there; a lift in machine Z would contradict the
  control's own height management.
- `tormach-lathe.cps` `onClose`: G30 is the lathe's operator-preset safe
  spot in X/Z, the same one each tool change goes to; there is no "straight
  up" on a lathe to lift to first. The related risk - the first rapid after
  a tool change runs diagonally from the G30 spot to the start point, and
  the closing G30 diagonally back, so a G30 stored inside the stock's
  envelope could cross the stock - is a different fix (an X-then-Z
  approach order) and is filed as #1604 (P3).

**Considered and dropped:**
- Dropping the tool-change G30 - PathPilot's documented convention, and
  the lift after the change makes the stored G30 Z harmless anyway.
- Suppressing the 4th-axis duplicate lift with a "Z is up" flag - more
  state for no safety gain; the extra line moves nothing.
- Lifting only when the retract wrote a G30 - M6 may move Z on a changer
  machine, so the unconditional lift is the simpler safe rule.

**Cost to change later:** one line in `startPreamble`; the Tormach mill
goldens carry one added line per tool change.
