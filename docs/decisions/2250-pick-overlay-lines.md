## #2250 - the pick overlay lines are built once per input

**What was decided.** `interact::selection_feature_lines` reads the
violet pick lines from a memo in `crates/ui/src/pick_line_memo.rs`. One
miss runs the old build (`feature_lines`) one time. A hit returns the
shared `Arc<[[f32; 6]]>`. The memo holds at most 8 inputs, most recent
last.

The key is the input itself (rule 2 of
`docs/decisions/2238-frame-cost-rule.md`), never `doc_version`:

- the shown pick list, by value;
- the pick ink (the theme can change it);
- the vertex cross arm, only when a vertex is picked;
- for each picked model: the display-off flag, the entity, the
  allocation of its `Arc<Features>`, and the pose the viewport draws it
  in (`setup_align::view_pose`, so a change of the active setup or a
  move is a new key).

These are all the reads of the build. A hit is the line set that a fresh
build gives.

**Safety line (rule 6).** The lines are display only. No generator,
check or post reads them.

**Why.** On fig8, state c of the #2238 probe (2D Adaptive on one welded
face), the piece `scene/overlay.selection_features` cost 1.48 ms on each
still frame in release. A welded-face pick regrouped its patch rings on
each frame.

**Dropped.**
- A key on `doc_version`. Rule 2 forbids it.
- A cache in the scene scope only (`frame_memo::scene_scope`). It lives
  for one collect, and the cost is per frame.
- A change of `feature_lines` to the patch memo of #2247. The memo here
  makes it run only on a changed input, so the gain is small.
- The `ui/viewport.pick_ink` piece (0.96 ms in state c). It is a
  different piece; it stays in the #2238 table.

**Cost to change later.** Small. A new read in `feature_lines` must be
added to `Key`; the pin `pick_lines_build_once_per_input` compares each
answer with a fresh build.
