## #93 - the drawn cutter, neck and shank (spun profile, slice 2)

**Decided:** a milling tool's own shape is a drawn spun half-outline
stored beside the tool's numbers, never instead of them. It is a new
component (`schema::ToolCutter`) set by its own undoable command
(`SetToolCutter`, MCP `set_tool_cutter_profile` / `get_tool_cutter_profile`),
saved as an additive `cutter` key on the tool record (no project-version
step: absent = the outline the numbers draw, which is every older file).
The editor's Profile tab gains Draw holder / Draw cutter; the holder view
stays the default and unchanged.

**The numbers stay the truth.** `cutter_profile::check` refuses by name an
outline that does not start at the tip, does not reach exactly the stated
diameter within the flute length (never wider, never short of it), does
not end at flute + shank length, is wider than the stated shank above the
flute top (the shank the stack laws judge), folds back over itself, or
fails any spun-profile law (negative / non-finite radius, open ends, too
few points, ...). Agreement slack is 0.001 mm. No generator or judging
law reads the drawn cutter, so a drawn cutter cannot move an operation's
stamp (pinned).

**A number edit that would break a drawn cutter is refused** (`ChangeTool`:
"the drawn cutter profile no longer fits ... reset the cutter profile to
Parametric first"), the same way a body that no longer fits is refused.
Considered and dropped: silently dropping the drawn cutter on the edit
(loses the user's drawing without a word), or keeping it and showing a
stale picture (the drawing would disagree with the numbers the laws read).
Cost to change later: one branch in `ChangeTool::new`.

**Separate component, not a field on `Tool`.** `Tool` is built by struct
literal in many places and its record feeds every generator; a sibling
component keeps the drawn data out of every path a toolpath reads and
needs no change to those constructors. DeleteTool / DeleteEntity snapshot
it so undo restores it.

**Turning tools** are refused (`CutterError::Lathe`) and never see the
Draw cutter view - the owner's "keep Lathe and Milling tools in separate
editors".

**Owed (follow-ups):** feeding a drawn neck to the simulation and reach
laws (a necked tool reaches deeper than its stated shank cylinder) is a
law change and is filed separately; the drawn holder profile (slice 1)
still has no MCP tool of its own.
