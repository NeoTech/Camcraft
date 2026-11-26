## #3915 - The `import_model` answer for an update gives `changes`, in MCP only

**Decided.** When `import_model` updates a model that was there before the call, its answer gets the optional key `changes`:

```text
"changes": {
  "faces":   { "carried": <n>, "added": <n>, "missing": <n> },
  "patches": { "carried": <n>, "added": <n>, "missing": <n> },
  "chains":  { "carried": <n>, "added": <n>, "missing": <n> },
  "summary": "<n> faces, <n> edge chains changed"
}
```

The counts come from `revision::changes`, the diff that `UpdateModel::changes` gives the window after a re-import. `summary` is `FeatureChanges::describe`, the phrase the window shows. A first import has no `changes`. A refused import has no answer to add it to.

The hook is in `mcp/tools.rs::import_model` (four lines). The logic is in the new `mcp/reimport_changes.rs`. Before the import, it keeps each model's facets (`Arc`, no copy) and revision counter. After the import, it selects the model by the answer's own `model` name and adds `changes` only when that model's revision counter moved.

**Why.**
- The step-0 baseline (C13, 2026-11-26) found that the update answer holds only `facets` and the notes. #3814 asks for "counts of carried, added, and missing faces, patches, and chains". This is that shape, so #3814 can take it as it is.
- `script/api.rs::v_import` is the other place that could hold it. Two open C13 branches change that function (PR 1 adds `with_source_faces`, PR 2 adds the lost picks). An MCP-side hook keeps this PR out of that code.
- The answer's `model` selects the snapshot, so the file-stem rule of `ImportModel::graded` is not copied. If that rule changes, nothing here goes wrong.
- Counts only, no id lists: the baseline bracket lost 23 face ids and 10 chain ids in one change. An agent that wants the ids calls `list_features`.

**Considered and dropped.**
- Compute it in `v_import` from `UpdateModel::changes()` before `execute`. It is two lines there, and `cam.import` scripts would get it too. But it edits the same lines as PR 1 and PR 2. It can move there when those land.
- Copy the stem rule to look up only the one model. That keeps a second copy of a naming law.
- Id lists of the added and missing ids. They are long, and `list_features` gives them.

**Cost to change later.** One new file and four lines in `import_model`. To move it into `v_import`, call `before` and `add` there instead. The pins are in `crates/app/tests/mcp/reimport_changes.rs`.

## #3915 - No tool description changes in this PR

**Decided.** The descriptions of `import_model`, `list_features` and `get_operation_geometry` stay as they are. The new fields are optional output only.

**Why.** `mcp_doc_drift` pins `docs/agents/mcp-tools.md` to the live tool table, and the file is made again with `cargo run ... mcp reference`. Builds were held when this was written. The doc lines are owed: one sentence each for `changes` and `source_name`, made with the reference command before the PR is ready.

**Cost to change later.** Two description strings and the regenerated reference file.
