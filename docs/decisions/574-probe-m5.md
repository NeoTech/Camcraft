## #574 - a probe section states M5 after its tool change

**Decided:** every mill post (fanuc, haas, tormach, sinumerik, heidenhain,
mazak) writes its own spindle-stop word - `M5`, `MOTOFF` on Mazak - in a
probe section's preamble, after the tool change and length offset and
before coolant and the first move. A probe section is one whose stream
states a spindle speed of 0 (`spindleWait && !(sRpm > 0)`): tools refuse
rpm 0, so only a probe reaches it. It is written even without a tool
change (a same-tool continuation), the case the issue names.

**Why:** the spindle being stopped at a probe was a property of the control
(PathPilot's `M6` stops it), not of the program. The stop word is
idempotent and costs one block.

**Considered and dropped:** `S0` (only sets a speed; some controls refuse
it); keying on "no SpindleSpeed PARAM" (a legacy `.cl` with no PARAMs would
then gain an `M5` - the stated-0 test leaves those programs unchanged);
folding the stop into #523's `sRpm > 0` gate as an `else` (that gate is a
separate PR, #526; a separate `probeStop` flag computed at section open
composes with it in either merge order).

**Cost to change:** one line per post plus the probe goldens.
