## #2201 - Frame caches for the toolpath overlay and the solid box

**Decided.** Two display caches, each keyed on its real inputs and never
on the document version alone (docs/decisions/2238-frame-cost-rule.md):

- `crates/ui/src/path_line_memo.rs` keeps the overlay lines of each
  drawn operation. The key is the path signature (`Toolpath::path_sig`)
  with the move count, the verify warnings by equality, the palette, the
  dim factor, the chevron arm and the viewport ground colour. Entries of
  operations that do not draw are dropped on each frame.
- `crates/ui/src/solid_aabb_memo.rs` keeps the world box of the solids.
  The key is, for each model: the entity, a `Weak` of its facet `Arc`
  (compared by `ptr_eq`) and its pose by value.

Per-frame readers in `crates/app/src/model/toolpath_read.rs`
(`with_toolpath`, `toolpath_warning_count`, `toolpath_nonempty`) read the
stored path in place. `status_of`, the browser tree and the operation
page no longer clone each path on each frame.

**Why.**
- The path signature is written by `Toolpath::new`, the one constructor,
  over each move field on the 1 um lattice. A change below 1 um is below
  the f32 display resolution of the overlay. A whole clone of the path as
  the key costs the clone that the cache removes.
- A `Weak` holds the allocation, so an address is not used again while
  the key holds it. A geometry write replaces the `Arc`, so a new list of
  the same triangles builds again (safe; seldom).
- `move_colour` fades a dimmed path toward the live theme ground. The
  ground is in the key. The tests pass the ground through `lines_on` and
  do not set the global theme, so parallel tests do not race.
- Display only: verify, the collision checks and the post never read
  these caches.

**Dropped.**
- A cache of `operation::status_of` for the browser and the status bar.
  Its input is `op_stamp`, which reads the whole prior chain. A correct
  key needs a writer revision for each input; that is a separate change.
  It is tracked on #2201.
- A key of `doc_version` alone: it misses theme, palette and view inputs.

**Cost to change later.** Low. Each cache is one module with its tests;
the call sites are three in `interact.rs` and one loop.
