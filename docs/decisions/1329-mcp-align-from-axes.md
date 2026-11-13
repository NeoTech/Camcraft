## #1329 - MCP: align the part from two axis picks (full 3D frame)

**What was decided.** `set_part_axes` keeps its name and arguments and now
always builds `AlignPartToSetup::align_from_axes`, exactly as the Setups
page's "Part axes" Apply does (`CommandKind::AlignPartFromAxes`). With one
axis given, that command falls back to the in-plane turn (#557), so the
one-axis behaviour and its `turn_deg` reply are unchanged. With both axes
given, the picks' 3D directions define the full frame (#769): the reply
says `frame: "full_3d"` and `turn_deg` is null (a 3D frame has no single
in-plane angle); a left-handed pair is honoured literally, not refused.

**Why.** The issue asks for GUI parity: an agent could not reach the full
3D alignment, and a left-handed pair the GUI lands was refused over MCP.
Reusing the one tool keeps the agent surface small and the call shape the
same; a separate tool would have duplicated the arguments and left two
ways to say the same thing.

**Considered and dropped.** A new `align_part_from_axes` tool beside the
in-plane one (more surface, and the in-plane-only path with both axes
would still disagree with the GUI). Keeping `turn_deg` for both axes by
computing the in-plane angle (it refuses a left-handed pair and means
nothing for an out-of-plane frame).

**Cost to change later.** Low: the reply's `frame` field is additive; an
agent that relied on the old in-plane-only reading of two axes (or on the
left-handed refusal) now gets the GUI's full 3D landing instead.
