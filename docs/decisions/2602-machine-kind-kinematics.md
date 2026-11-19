## #2602 - Refuse a machine edit that adds turning capability

**What was decided:** `edit_machine` refuses a change from a non-turning
machine to `lathe` or `mill-turn`. The refusal names the machine and tells
the caller to load a turning machine file. Existing turning machines can
still be edited.

**Why:** Changing the kind does not add a work spindle. The flattened
`Machine` data does not retain the spindle's role. Its axis inventory
cannot prove that a milling spindle can turn the work. A kind edit must
not let the same mill identity authorize turning code.

**Considered and dropped:** Requiring the declared kind to equal the kind
inferred from the axes can reject valid Y-axis lathes. Keeping the edit and
adding a warning still permits turning code under a mill's identity. The
refusal prevents that output before the edit enters command history.

**Cost to change later:** Add explicit work-spindle kinematics and validate
the complete machine conversion. Then permit a kind change only when the
definition proves the required turning capability.
