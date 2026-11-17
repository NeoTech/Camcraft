## #2235 - a chain selection costs its geometry once, not on each frame

The maintainer's words (2026-11-16): "the performance needs to gget down
under 10ms to be useable", and "caching to not have to redraw things should
prob. be a strategy thru out the application to keep framerates stable and
enough compute lefter over for things that matter.. Precalculation and
cache is not something that will give us a penalty.. this is not game
engine".

**What was decided.** Three caches of derived values. None is a command
and none is in the project file. Each key is the input itself, so a stale
answer cannot occur:

1. `model::features_of_shared` (app): the features of one model, keyed by
   the allocation of its facet `Arc` (a `Weak` pointer compare). Each
   landing (`set_facets`) makes a new allocation, so it is a new key. A
   dead allocation is pruned. 32 entries.
2. `model::resolve_chain` (app, `model/chain_memo.rs`): the chain for one
   `ChainId`, keyed by the allocation of the `Arc<Features>` from item 1,
   with one map of ids for each. A miss is kept as a miss. All the call
   sites that took `feature::resolve_chain` now take this one. The change
   in about 15 files is a signature ripple, not new logic.
3. `operation::pocket_memo` (app): the pocket family's region on a part,
   keyed by the exact input (loops, part facets, ladder planes, unit)
   compared for equality. The region and the `Invalid` / `Empty` refusals
   are kept; other errors are computed again. 4 entries.

In the UI, `frame_memo::scene_scope` opens a memo for the time of one
`world::collect`. Inside it, `region_lines` and `shown_regions` get the
same region for an operation from one `region_of` call. The state is
borrowed immutably for that scope, and the memo is dropped at the scope's
end (also on an unwind).

**Why.** Measured headless in release on the fig8.step part with 8-10 chains
selected along its top edge: a still frame took 640-765 ms in the scene collect
and 425-510 ms in the UI step. The cause was that each frame:

- content-hashed all the facets in `features_of`;
- walked all the model's chains (about 5 ms) in each `resolve_chain` call,
  for each chain and each caller;
- sliced the part at each ladder plane (about 15 ms) for each of three
  `region_of` calls.

After the caches, a frame is about 6 ms headless. A still frame makes no
digest, no chain walk and no slice. The cost moves to the first frame
after a selection change or a landing, which the maintainer accepts.

**Dropped.**
- A key on the document version counter. It is bumped by hand at about 70
  sites, and one missed bump gives a stale overlay. An allocation key or
  an equality key cannot miss a change.
- A cache in the UI state, filled by the UI. That puts derived geometry
  into the UI layer, and each new caller must remember to use it. The app
  layer caches serve the GUI, MCP and the CLI.
- A change to the hover and highlighter paths. That is #2201.

**Cost to change later.** Each cache is one file with a counter that the
pinning tests read (`chain_walks`, `feature_digests`, `pocket_slices`,
`frame_memo::region_builds`). The pattern (key on the input or on its
allocation, never on a hand-bumped counter) can be used again for other
derived views.
