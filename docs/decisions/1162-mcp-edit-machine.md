## #1162 - MCP edit_machine (merge patch over the mounted machine)

**Decided.** `edit_machine {patch, setup?}` applies an RFC 7396 merge patch
to the stored `Machine` (the project-file form the Machine window also
edits) and mounts it through `LoadMachine` (document) or `SetSetupMachine`
(a setup's own machine): the same undoable commands Apply uses, one undo
step. `get_job` gains `machine {path, definition}` for read-back.

**Why.** The window edits a `Machine` copy, not `.mch` JSON, so the patch
shape is the stored shape; one door, one undo step, no second validator.

**Refusals by name.** New `Edm::refusal()` (post crate) names the first
broken rule as `edm.<key>: <rule>` (same laws as `Edm::is_sane`). Unknown
keys are found by re-serializing the merged machine and refusing any patch
key that did not survive (`unknown key 'edm.orbit_radius'`). A no-op patch
and a setup without its own machine are refused too.

**Dropped.** (a) `deny_unknown_fields` on `Machine`: would break project
files carrying newer keys. (b) A `.mch`-grammar patch: needs a
Machine-to-file writer that does not exist. (c) `edm.codes`: the issue names
it, but this tree has no such field (generator codes are separate
sub-issues); nothing invented. (d) Editing the camcraft-mcp skill: a process
file; the PR notes the tool for the maintainer.

**Cost to change.** Low; the tool is one file.
