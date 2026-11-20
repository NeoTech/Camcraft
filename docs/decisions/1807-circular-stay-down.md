## #1807 - Keep Circular ramp transitions at depth

**Decided:** For an explicit Circular Ramp entry, limit the effective
`approach.plunge_height` to 0.1 mm. Keep a lower configured value. Use
`entry_top(material_top)` to place this gap above the remaining material.
The hole top alone does not establish the remaining stock height.

Replace the default Clearance level link with the existing certified
KeepDown link when the effective transition is also Ramp and
`approach.ramp_to_depth` is false. Keep an explicit Retract link and the
links for other transition types. Keep the ring order, the lifts between
rings, the final retract, the entry refusals, and the ramp angle law.

**Why:** A closed ring ends at its next ramp start. The existing link
proof can certify the connection on the floor just cut. This removes
the unnecessary clearance trip without adding a new safety proof. A
small initial gap reduces the ramp travel through air. The remaining
material height keeps the rapid above stock carried over the hole top.

**Dropped:** Keep the clearance trip and feed its final descent segment.
That option retains the extra travel. Change the shared ladder code.
That option would change operations outside Circular. Use the hole top
as the stock top. That option can put a rapid into uncut stock.

**Cost to change later:** Change the local Circular parameter adaptation
and update its motion pins, application safety test, and Ramp golden.
