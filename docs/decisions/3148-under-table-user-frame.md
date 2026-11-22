## #3148 - The under-table refusal names the heights the user set

**Decided:** `FirstTraverseUnderTable` gives its plane and its limit in the
frame the user sets the clearance height in. `in_user_frame`
(`crates/app/src/operation/first_traverse.rs`) adds the frame's `origin.z`
to `height_mm` and `limit_mm`, as #2991 does for `FirstTraverseLow`. The
text gives the two values in mm "measured along the tilted tool axis", and
no longer as Z values. The refusal decision (`table_read`, the #3081 law)
does not change: the shift is applied after the verdict.

**Why:** a user who types the old (local) limit gets a plane that is too
low (origin Z > 0) or the refusal again (origin Z < 0). A height along a
tilted tool axis is not a machine Z.

**Dropped:**
- A `tilted` field, as `FirstTraverseLow` has. The verdict comes only from
  a tool axis that points down (a head tilt past 90 degrees), so it is
  always tilted. A new field would change the cache format for no reading.
- A stamp bump that makes old caches stale. A cache saved before this
  change keeps its local numbers until the operation generates again. The
  verdict and the post block stay the same, so only the old numbers show.

**Cost to change later:** one match arm and one format string.
