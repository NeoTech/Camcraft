## #2088 - 3D Pocket cuts a blind floor shallower than one stepdown

**Decided:**

1. `machining3d::rest::rough_levels` augments `cut_levels_under` for
   `run_plain` and `run_rest` (`pocket3d`, `pocket3d_prev`,
   `pocket3d_offset`, `pocket3d_offset_prev`, and their `*_with_rest*`
   twins) with cuttable floor heights strictly between `p.top_z` and the
   first ladder plane below `p.top_z`.
2. When deeper levels already cut (`!first_center.is_empty()`), only
   up-facing horizontal floor heights (`flat_floor_heights_between`,
   using `zmax` so tiny facet tilt never drops a plane below a vertex)
   that expose cuttable tool-center area not covered at `first` (eroded
   by `1e-3` to reject micron slice jitter) are added.
3. When `first_center` is empty, horizontal floors are collected first;
   if none yields a cuttable center, `vertex_heights_between` adds at
   most the single deepest cuttable vertex plane (for a sloped or
   curved shallow pocket), or preserves an open flat plane whose
   center is empty so `ladder` reports `"too narrow"` by name instead
   of `"the model fills the pocket"`.
4. `pocket3d_air_split` (rotary wrap roughing) keeps `cut_levels_under`
   directly so empty height-window refusals in `rotary_offset` and
   `rotary_adaptive` goldens remain unchanged.

**Why:**

Under #2004 (PR #2013), `cut_levels` starts at `top_z - stepdown`. On
a blind hole shallower than one stepdown (for example depth 3 mm at
stepdown 4 mm with `bottom_z` below the floor), no ladder plane sits at
or above the floor. Isolated pockets failed with `GenError::Empty`, and
pockets whose footprint also covered deeper stock around the model
skipped the shallow blind hole. Checking `(first, top_z)` cuts the
floor in one level with cut depth at most `stepdown`.

**Dropped:**

- **Fallback only when `path.moves.is_empty()` in `ladder`:** misses a
  shallow blind hole whenever open stock around the part or a deeper
  pocket keeps `first` non-empty.
- **Snapping floors below `first`:** breaks the coarse roughing
  staircase contract pinned by `a_surface_between_levels_leaves_a_staircase_not_a_lie`
  and `the_rest_pass_closes_the_staircase_it_inherited`.
- **Using all vertex heights when deeper cuts exist:** would insert
  spurious levels on sloped flanks, chamfers, and domes.
- **Changing `pocket3d_air_split`:** rotary wrap windows that fall
  above the part skin are tested to refuse as empty (`rotary_offset`).

**Cost to change later:** one helper `rough_levels` in
`crates/motion/src/machining3d/rest.rs` and two call sites in
`crates/motion/src/machining3d/entry.rs`.
