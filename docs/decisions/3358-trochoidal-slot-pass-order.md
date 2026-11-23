## #3358 - Trochoidal Slot advances from the entry bore

**What was decided.** The shared `Pass starts at` row shows Auto and
explains the fixed sequence. The MCP `add_operation` and `edit_operation`
tools return that reason when a Trochoidal Slot patch specifies `pass_start`.
The project parameters retain their existing shape.

**Why.** At each depth, the helix opens a bore at the start of the
centreline. For a wide band, the bore opens out to the start of the first
loop. The main loop centres advance along the centreline at equal spacing.
The final arcs advance to the end of the slot. A Microlift return follows
the arcs it just cut and descends at their start before the next loop.
The planner measures engagement and checks lifted returns in this sequence
against the stock removed by the preceding moves.

**Considered and dropped.** Sorting or reversing the arcs would change
the leading cut and the clearance available to later cuts and returns.
The bore opening is part of entry. Inside and Outside would need a
separate path strategy for those choices.

**Cost to change later.** A different sequence needs its own entry,
links, engagement measurement and lifted-return checks. It also needs
tests for the new paths and a byte comparison for the Auto path.
