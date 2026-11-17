## #2247 - the welded-face grouping is made once per feature set

**What was decided.** `model::patches` and `model::patch_rings` (app,
`model/patch_memo.rs`) memoize `feature::patches` and
`feature::patch_rings`. The key is the allocation of the `Arc<Features>`
(rule 2 of `docs/decisions/2238-frame-cost-rule.md`, second form), the
same key as `model::resolve_chain`. A landing makes a new allocation, so
it is a new key. The pick chips (`State::geo_chips`) and
`pick::face_origin` (which the chips and `pick::align_321` call) read the
memo. `face_origin` also finds the strips of a patch with a binary search
in the sorted `Patch::faces`, not with a linear `contains` for each face.

**Why.** The chips and the 3-2-1 check grouped all the faces of the model
again on each frame (6.7 ms of a still frame in release on fig8). The
grouping is a pure function of the feature set.

**Dropped.**
- A cache of the whole chip list keyed by the picks, the features, the
  view pose and the unit. The key is larger and the label code stays in
  the UI. The memo of the shared step removes the cost with a smaller
  key.
- A change of the other `feature::patches` callers (commands, MCP,
  heights). They do not run on each frame. They can move to the memo
  later, with no change to their answers.

**Cost to change later.** Small. The memo is a thread-local list of at
most 32 entries; a dead feature set is pruned.
