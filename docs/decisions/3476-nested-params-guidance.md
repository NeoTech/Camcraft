## #3476 - Give the corrected flat edit_operation call

**Decided:** Keep the existing flat parameter patch interface. When an edit
refuses a single wrapper for the operation's actual kind, append the exact
flat call if that call passes the existing patch and edit-command checks.
The suggestion preserves the requested name and all inner patch values as
JSON. The refused call does not execute a command or change undo history.

**Why:** Issue #3476 permits this diagnostic-only option. It lets an agent
retry parameters copied from `get_job` without changing accepted input shapes
or machining semantics. The normal flat edit path and its validation stay
unchanged. Construct the candidate command only to check the suggestion;
never execute it during refusal.

**Considered and dropped:** Accepting the kind wrapper would make reads and
writes use one shape, as the issue prefers. That changes the accepted input
contract. This change instead gives explicit guidance and preserves the
existing refusal. Do not suggest a call for a mismatched kind, additional
outer keys, unknown inner fields, malformed values or a failed command
check. Removing such data would produce a misleading or unusable call.

**Evidence:** The regression reads parameters from `get_job`, changes
`pocket.stepdown`, and checks the refusal, corrected JSON, retry and history.
The unchanged implementation failed at the required corrected-call assertion
before this fix. Additional cases cover escaped display names, another kind,
nested fields, null removal, unknown fields and invalid wrappers or values.
The final checks must run on the committed implementation before publication.

**Cost to change later:** Nested acceptance can be added as a separate input
contract change. It needs explicit matching-kind and unknown-field rules and
new acceptance tests. No project migration or dependency change is needed
for this diagnostic.
