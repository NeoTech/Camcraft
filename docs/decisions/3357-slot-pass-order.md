## #3357 - Slot has one continuous pass at each depth

**What was decided.** The Slot Passes tab shows the shared `Pass starts
at` row, fixed at Auto with the Slot's reason visible. The MCP
`add_operation` and `edit_operation` tools give the same reason if a
Slot patch specifies `pass_start`. The Slot project parameters keep
their existing shape.

**Why.** At tool width, the cutter follows the centerline. In a wider
slot, the cutter enters once per depth, follows the connected rows and
then cuts the wall lap from the end of the last row. Inside and Outside
describe an order for separate passes. The connected Slot path has no
such order.

**Considered and dropped.** A wider slot could start at a different row
and link to the remaining rows. This would change where the tool enters
uncut stock, how it reaches later rows and how it joins the wall lap.
That needs a new path strategy, with its own checks for engagement,
links and remaining material. A hidden field or an ignored MCP input
would leave the person or agent without the reason.

**Cost to change later.** The row can use the shared ordering parameter
once Slot has a planner that certifies the new paths. The planner needs
its own tests for the two entry directions, the links, the wall lap and
the Auto path.
