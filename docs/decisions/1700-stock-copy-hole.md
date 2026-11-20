## #1700 - A stock copied from an incomplete model keeps naming its hole

**Decided:** a `FromModel` stock copy carries its source's hole
(`schema::MeshOpen`) itself, beside the fidelity-unknown sentinel it
already carried (#1056, #1610). Every path that lands the sentinel on a
copy lands the hole with it: `SetStock` and its undo, a setup clone
(`CreateSetup` with a datum), an undone `DeleteSetup`, `UpdateModel`'s
refresh of the copies and its undo, and the project loader. The refusal
(`model_fidelity::op_problem`) reads the hole off the entity that carries
the sentinel, so a copy's refusal still names its source and that hole
once the source is deleted. `mesh_open::hole_behind` is
`model_fidelity::chord_behind`'s twin, and `mesh_open::note_for` gives the
words for a hole already in hand.

**Why:** the copy's refusal read the hole from the source model by name
(`mesh_open::unknown_note`). With the source deleted that lookup found
nothing, and the refusal fell back to "saved before CamCraft recorded
mesh fidelity (project v58 or older)", which is the wrong reason for that
mesh. The post still refused it; only the words were wrong. The copy
already held the sentinel for exactly this case (#1056: "keeps it when
the model is deleted or renamed"), so the hole now rides the same way.

**File format:** the saver already wrote `mesh_open` for any entity that
has one. The loader now also accepts it on a `FromModel` stock marked
`fidelity_unknown`, and a hole without the mark stays corrupt. Commit
`35b3422cf` (#1610) shipped in `v0.21.0` through `v0.24.0`, where
`validate.rs` refuses `mesh_open` on any entity that is not a model
marked `fidelity_unknown` as corrupt. A project file written with a hole
on a `FromModel` stock row therefore does not open in `v0.21.0` to
`v0.24.0`, making this an incompatible project-file change (marked with
`fix(fidelity)!:` and a `BREAKING CHANGE:` footer).

**Review follow-up (PR #2024):**
- **Released `v0.21.0`–`v0.24.0` files:** those builds wrote
  `fidelity_unknown: true` on the stock copy and `mesh_open` only on the
  source model. `model_fidelity::adopt_copies` now heals such a copy on
  load from its live source (so deleting the source later keeps the hole
  and re-saving repairs the file), and `model_fidelity::coarse_source`
  falls back to the source's hole if an unhealed copy has none. Pinned
  by `a_v021_to_v024_copy_without_its_own_hole_keeps_naming_its_sources_hole`
  in `crates/app/tests/stock_copy_hole.rs`.
- **Project version:** kept unbumped on this branch (`67`, unchanged
  from its `upstream/main` merge base) rather than bumping `67` to `68`
  (which would conflict with `upstream/main`'s `v68`–`v71` ladder) or
  pulling `upstream/main` and bumping past `v71`. `upstream/main` is
  already at `v71` (above `v0.21.0`–`v0.24.0`'s `v61`–`v69`), so merged
  saves already trigger `UnsupportedVersion` in those releases, while any
  pre-#1700 `v67`–`v71` build fails closed as corrupt rather than dropping
  the hole. Cost to change later: one `PROJECT_VERSION` bump and
  no-op migration rung if a version refusal is wanted against unreleased
  pre-#1700 `v71` builds.

**Considered and dropped:**
- Keeping a tombstone of the deleted source's hole (by name) in the
  world, which would have given a second source of truth.
- Looking up a copy of the deleted source by name at refusal time. That
  has more than one possible answer when two setups each copy the model,
  and it is wrong after a rename.

**Cost to change later:** the hole is one optional component on the copy,
written beside the sentinel at the six sites listed above. Pinned by
`crates/app/tests/stock_copy_hole.rs`.
