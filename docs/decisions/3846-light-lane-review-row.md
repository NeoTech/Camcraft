## #3846 - Keep a fixed light-lane row in each review

**Decided:** Add a fixed table row to step 7 of `PR-reviewer.md`.
Each review records the light-lane result, the current head, and the evidence.
A review with no completed run keeps the row and gives `Not run` with a reason.
The existing xtask documentation tests pin the row and its reporting instructions.

**Why:** Step 5 already requires the light lane on both review paths.
Step 7 mentions its result, but that prose did not prevent the reported omissions.
A fixed row makes a missing result visible without a new gate or workflow.
A red light run stays a note, not a blocker by itself.
`Not run` is never a pass for a required step.

**Considered and dropped:** Do not add a merge-time checker or change the gate policy.
Do not change the full heavy lanes, runner routing, or `PROCESS_VERSION`.
The merging session owns the process-version bump.

**Cost to change later:** Change the report row and its xtask test together.
No runtime code, project format, or posted output changes.
