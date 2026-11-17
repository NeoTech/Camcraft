## #2238 - the frame-cost rule: a still frame computes nothing again

The maintainer's words (2026-11-16): "caching to not have to redraw things
should prob. be a strategy thru out the application", "the performance
needs to gget down under 10ms to be useable", and "isolate each cost and
cache them".

**What was decided.** One rule for each derived value that a frame shows
(a region, a fill mesh, a chain, a note, a label list):

1. **Compute at the change, not at the frame.** A frame where no input
   changed computes no derived geometry again. It reads a cached answer.
2. **The key is the input.** Take the first key in this list that fits:
   - the input value itself, compared for equality (#2235
     `operation::pocket_memo`);
   - the allocation of an `Arc` that a change replaces (#2235
     `model::features_of_shared`, `model::resolve_chain`);
   - a revision counter that the one writer of the input bumps in the
     same function that writes it (for example a selection revision that
     only `SetOperationGeometry` and its undo move).
   A key on `doc_version` alone is not permitted for new caches. It is
   bumped by hand at about 70 sites, and one missed bump shows a stale
   answer.
3. **The writer invalidates.** No reader clears a cache. A new key is a
   miss; an old entry is dropped by a size cap or when its `Weak` dies.
4. **Store it in the layer that owns the input.** Derived document
   geometry is cached in `app` (it serves the GUI, MCP and the CLI). A
   value that only the screen uses (a triangulated fill, a projected
   label list) is cached in a `ui` module next to the code that makes it
   (`frame_memo`, `boundary_draft`), never inside a widget or a page.
5. **Each cache owes two tests.** (a) A still frame makes no new compute:
   assert the cache's counter, never a time. (b) A changed input gives
   the new answer, equal to a fresh compute.
6. **Safety line.** The verify check, the collision checks, the stock
   simulation verdict and the post gate read the document and compute
   their own answer. They never read a display cache. A display cache
   can be wrong for one frame; a gate cannot.
7. **Measure with the probe.** `frame_cost::span` names a piece of the
   frame (`commands/`, `scene/`, `ui/`). The test
   `frame_cost_probe::still_frames_on_fig8_name_their_costs` prints the
   cost of each piece in five states. A new cost over about 1 ms in a
   still frame gets an issue.

8. **Simulation keyframes are a cache under the same rule** (#2181,
   #2193, #2194). A keyframe (a stock state at a program position, or a
   field brick of it) is keyed on its inputs: the stock definition, the
   tool bodies, the toolpath values up to that position, and the position
   itself. A brick that no move up to that position touched is shared
   with the earlier keyframe, not copied (#2194). The writer of a
   toolpath, a stock or a tool drops the keyframes from the first changed
   move onward; the keyframes before that move stay valid. Frames shown
   while the removal is calculated (#2193) read the newest finished
   keyframe and say that the result is in progress. The verify check, the
   collision and gouge checks and the post gate never read a keyframe: a
   gate that needs the stock state calculates it from the document, so a
   stale or partial keyframe can never pass a program.

**How the caches of #2235 fit.** `features_of_shared` and
`resolve_chain` use an allocation key (item 2, second form).
`pocket_memo` uses an equality key (item 2, first form). The
`frame_memo::scene_scope` memo is different: it lives for one scene
collection only and makes one region build per frame, not zero. It
removed the double work inside a frame; a cache by rule 1 is still owed
for the region outline and fill.

**Why.** The probe found that the still-frame cost is in derived values
that the frame makes again from unchanged inputs; the measured table is
in the PR and in the #2238 comment. A rule with a fixed key order lets
each sub-issue add one cache without a new design.

**Dropped.**
- A frame-rate cap or a skipped frame. It hides the cost and does not
  remove it; the input lag stays.
- One global cache keyed on `doc_version`. Rule 2 says why.
- Moving the hot paths to caches in this PR. Each cost is a sub-issue,
  so that each cache has its own pinning tests and review.

**Cost to change later.** The spans are no-ops unless a probe records
(one thread-local flag read). To remove the probe, delete
`crates/ui/src/frame_cost.rs` and its `span` and `time` calls.
