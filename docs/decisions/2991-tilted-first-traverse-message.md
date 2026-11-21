## #2991 - The first-traverse refusal of a 3+2 operation names the heights that the user set

**What was decided.**
- `Warning::FirstTraverseLow` gets the field `tilted: bool`, with
  `#[serde(default)]`. A cache saved before this change loads, and its
  verdict reads as upright, with the text that it had.
- The law (`camcraft_sim::verify::first_traverse_low`) does not change. It
  gives the two heights in the frame of the path and `tilted: false`.
- The operation layer (`operation::first_traverse::judge`) puts a 3+2
  verdict into the frame in which the user sets the heights. Generate
  shifts each height by `h - origin.z` into the frame
  (`operation/generate.rs`), so `in_user_frame` adds `origin.z` to the two
  heights. A frame that is not upright (`TiltedFrame::is_upright`) sets
  `tilted`. In the guide example (`Indexed Face1`, B 20 deg, origin Z 20),
  the verdict now gives the clearance height 30 (the typed value) and the
  corner 31.114, not 10 and 11.114.
- The text of a tilted verdict says that the two heights are measured along
  the tilted tool axis, and it names the raise (`top - height`, the same in
  each frame). It has no `Z` value and no `X/Y`: "the first rapid (move 0)
  comes from an unknown position at the clearance height 30.000 mm, under
  the stock top at 31.114 mm (both measured along the tilted tool axis) -
  raise the clearance height by at least 1.114 mm".
- The text of an upright verdict does not change.
- The posed law (#3084, #3100) reads a 3+2 plane along the frame's axis
  from its origin, so its verdict is local too. It takes the same shift
  and `tilted` before it replaces the lower first-traverse verdicts. The
  tops then compare in one frame, and one verdict names the highest top.
  `FirstTraverseUnderTable` stays as it is (#3138).
- The troubleshooting page gets an entry for the new text. The indexed
  guide page says what the warning gives after a regenerate.

**Why.**
- The refusal was correct, but it named the local height (10), and the user
  typed 30 (the issue). A Fixed height is typed in the frame of the setup,
  and `h - origin.z` gives the local value. Thus `local + origin.z` is the
  typed value. For a height that refers to the model or the stock, the
  resolved value is `z_local + origin.z` (#2389, `fold_height`), and the
  same shift gives the resolved height. The raise is correct in the two
  cases.
- The conversion is in `app`, which knows the frame. The sim law stays one
  function in one frame.

**Considered and dropped.**
- Keep the local heights in the warning and add `origin.z` as a second
  field, for the text to add: two new fields in place of one, and the
  stored heights do not match the text.
- Change the upright text too (no `Z`, a raise): the upright text is
  correct, and many tests and the troubleshooting heading quote it. That
  is out of the scope of this issue.
- A change of the op stamp salt (v4 to v5), so that each saved cache reads
  Stale and is generated again with the new text: each stamp moves, and
  the pinned stamps of `deburr_cache_stamp` and `wire_stamp` and the stamps
  of the example projects change for a text change. The verdict and the
  held post are the same in an old cache. Only its text is the old text
  until the operation is generated again.
- Regenerate `examples/guide-multiaxis.camproj`: the cache diff (stamps,
  rounding) is large for a text change. The guide page says "after a
  regenerate".

**Cost to change later.** Low. One field with a default, one map in
`operation/first_traverse.rs`, and one text arm in `verify/types.rs`.
