## #3348 - Trace clearance from protected faces and edges

**Decided.** Keep the cutting path on the selected line. Use the existing rapid
routing for upright links. Refuse generation if the full tool still enters the
requested clearance or the clearance check fails. Report the protected item.
Use the same protection setting in the viewport, MCP and the project file.

**Why.** Moving an engaged cutter around a protected item would mark a line
that the user did not select. Even a move with two clear endpoints can bring
the holder or the shank against a protected face between those points. The
existing sweep checks the full tool throughout each move.

**Alternatives.** A warning would leave an unusable path in the cache. Clipping
the trace would silently omit part of the selected line.

**Cost to change.** A future option that permits gaps would need a way to show
the omitted spans and test safe retract and re-entry at each gap.
