## #4214 - The `import_model` answer for an update gives `changes`, in MCP only

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

## #4214 - No tool description changes in this PR

**Decided.** The descriptions of `import_model`, `list_features` and `get_operation_geometry` stay as they are. The new fields are optional output only.

**Why.** `mcp_doc_drift` pins `docs/agents/mcp-tools.md` to the live tool table, and the file is made again with `cargo run ... mcp reference`. Builds were held when this was written. The doc lines are owed: one sentence each for `changes` and `source_name`, made with the reference command before the PR is ready.

**Cost to change later.** Two description strings and the regenerated reference file.

## #4214 - SolidCraft names show as `source_name` or `source_names`, absent when unknown

**Decided.** When a model holds the source face record of a `.sc` import (C13 PR 1), each `list_features` face row and patch row, and each face or patch pick that `picks_view` shows, gets one of:
- `source_name`: a string, when the feature lies on one SolidCraft face;
- `source_names`: a sorted array, when the feature spans several SolidCraft faces that CamCraft welds into one (coplanar neighbours, or a flat side, a fillet and a flat side that meet tangentially);
- and `source_unverified: true` when SolidCraft has not verified one of those names.

A row gets none of these keys when the model has no record, when the record does not tile the model's facets, or when the pick is no longer in the model. The keys are absent, not `null`.

**Why.**
- The C13 plan's agent side feature-detects on `source_name`, and most faces lie on one SolidCraft face. But the step-0 baseline found a patch over three names (`F2:side:b`, the fillet and `F2:side:c`). One string with a separator would make the agent parse names that can hold any character.
- Absent keys keep every row of an STL, STEP or OBJ model the same as before. Existing tests compare pick rows exactly (`mcp_pick.rs`).
- `source_unverified`: C13 PR 2 follows only verified names. The agent sees the same flag that decides a follow. `former` names are not shown (an open question in the status file).

**Considered and dropped.**
- Always `source_names` as an array: one shape, but the plan and the agent side test `source_name`.
- `null` when there is no record: it adds three keys to every row of every model.
- Leave out the features whose names are not verified: that hides a name that SolidCraft does give.

**Cost to change later.** One function (`put` in `mcp/source_names.rs`) writes all three keys.

## #4214 - The names use features built from the model's own facets

**Decided.** `mcp/source_names.rs` builds the features for the names with `feature::features_of_facets` on the model's facets, once per call. It does not use `features_of_shared` or `feature_graph_shared`.

**Why.** The record indexes facets in the model's facet order. Those caches fall through to a cache keyed by the content of the facets, which ignores their order (`feature::digest_of`). A hit from another mesh with the same content in another facet order gives other facet indexes, and so other names. The feature ids are order-free, so the ids in the fresh features are the ids of the listed rows. The extra work is one feature build for each `list_features` or picks view of a model that has a record; models without a record pay nothing.

**Considered and dropped.**
- Use the cached features and check each facet against the face plane. Two separate coplanar faces would pass that check with the wrong facets.
- A new cache keyed by the facet `Arc`. It is a second cache to keep correct, for a read that an agent makes a few times per job.

**Cost to change later.** Two lines in `Named::of`, if the features caches become order-exact.

## #4214 - The name logic is in the geometry crate; the MCP view is in the app

**Decided.** `camcraft_geometry::geometry::source_names` holds the pure part: from a record and a mesh's features, the name set of a face id or a patch. It returns no names when the record does not tile the facets, when an index passes the name list, or when a name is empty. The app's `mcp/source_names.rs` finds the record, builds the features and writes the keys. The hooks are three lines in `mcp/features.rs` (after the compact step, so compact rows keep the names) and two lines in `mcp/pick.rs::picks_view`.

**Why.** AGENTS.md "`app` holds the document": the name set works only on geometry values. C13 PR 2 needs the same name set for its follow, so it can use this function too. `picks_view` is the one place where the operation geometry, the selection, the area and "Do not touch" views and `pick_in_box` show picks, so they all get the names from one hook.

**Considered and dropped.** A method on `SourceFaces` in PR 1's `source_faces.rs`: that file belongs to PR 1. If PR 1 or PR 2 wants the function there, it moves with no change to the MCP side.

**Cost to change later.** One new geometry file (`pub mod` line in `geometry/mod.rs`) and its unit tests.
