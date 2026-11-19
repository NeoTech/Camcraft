## #2201 - the path overlay and the region are cached across frames

**Decided.** The overlay lines of each drawn toolpath are built once per
input and kept in `crate::path_overlay::PathLineCache` on `State`. The key
is the path's content signature (`Toolpath::path_sig`), its move count,
its warnings, the palette, the viewport ground, the dim and the chevron
arm. The chevron arm is kept under each model's facet allocation and pose,
so `model::solid_world_aabb` runs only when a model changes. The region of
a shown operation (`frame_memo::region_of`) is kept across frames under
the operation value and its `operation::op_stamp`. The op page and the
browser read the path through `model::with_toolpath` and do not clone it.

**Why.** The measurements on the windowed release build showed that the
CPU, not the GPU, is the cost. With an 8100-move path, `world::collect`
took 5-6.5 ms per frame (the path lines about 3 ms), and a selected or
previewed operation built its region two times per still frame. The GPU
encode was about 0.3 ms. So #2525, #2526 and #2607 (GPU draw work) are not
linked to this fix.

**Dropped.**
- A key on `doc_version`: crates/ui/AGENTS.md "Frame cost" forbids it, and
  the counter is bumped by hand, so a missed bump shows a stale picture.
- A `History` revision counter: it does not exist, and a new one has the
  same weakness.
- Measurement on the maintainer's project `testdata/models/fig8.camproj`:
  the worker brief forbids to read it. The numbers come from synthetic
  fixtures of the same size (about 8100 moves).

**Cost to change.** The two caches are local to `crates/ui`. A new display
input of the path must be added to the `Look` key, or the lines show the
old look until the path changes.
