## #1547 - Ignore unused stored Stepover in load strategies

**Decided:** Apply `!strategy.steps_by_load()` to the positive Stepover check in `Pocket3DParams::validate`.
The diameter check already uses this condition.
Adaptive and Offset rings accept finite stored zero and negative Stepover values.
Their generated moves stay identical to those for a valid Stepover of 3 mm.
Keep the finite parameter check and the optimal load limits.
Zigzag, Waterline and Auto still require a positive Stepover within the tool diameter.

**Why:** Adaptive and Offset rings space passes by optimal load.
They do not read Stepover, and their Passes page hides that field.
An unused stored value must not prevent generation or name a field the user cannot see.
This change needs no new dependency or toolpath algorithm.

**Considered and dropped:** Resetting the stored value would change project data without a machining need.
Asking the user to switch strategies would expose the field but retain an unnecessary refusal.
The tests reuse small fixtures and one depth pass to keep generation below 60 seconds.
The existing zero Stepover refusal assertion now tests Zigzag.
The new generation test covers acceptance under both load strategies.

**Cost to change later:** A strategy that starts to read Stepover must update its validation and its Passes page together.
The generation and refusal tests must then cover that new rule.
