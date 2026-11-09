## #531 - EDM condition table as user data

**Decided:** `edm.conditions[]` in the `.mch`: `{ id, name, gap, wire_diameter | undersize, role: rough|skim|finish, notes }`.
- `id` is the control's own selector, required and unique. A numeric id reads as its word.
- Lengths scale with the file's unit. Gap and undersize may be 0; a wire diameter must be positive.
- A garbled entry refuses the file, naming its id, and `is_sane` restates the law for project files.
- `Edm::condition(machine, id, need)` returns the condition with every needed field stated, or the refusal naming the machine, the id and the missing field. It is the door the first EDM op (#532) calls at command creation, so a missing gap or wire diameter is refused before any generator runs.
- No conditions ship (#290 D4). Tests use an explicitly synthetic table.

**Why:** absent is not zero, and generator data is vendor material. CamCraft needs only the geometry (gap, wire diameter, undersize) to place a path; power and flushing stay the control's, selected by `id`.

**Considered and dropped:**
- A separate conditions file per machine: the `.mch` is already the machine's statement, and the project file carries it whole.
- Modelling power, on-time or flushing: that is spark-process physics, out of scope for #290.

**Cost to change later:** low. The section is additive.
