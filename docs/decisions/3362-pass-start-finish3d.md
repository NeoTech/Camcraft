## #3362 - "Pass starts at" on 3D Finish

**What was decided.**

- 3D Finish gets the shared field `pass_start: machining2d::PassStart`
  (#3316) in `OpParams::Finish3D`. There is no new type. The project key
  is the shared `pass_start` (`PassStartDto`) with the same serde
  attributes as 3D Pocket: `auto` is never written, a file without the
  key reads Auto, and there is no version step. MCP uses the generic
  `edit_operation` (no kind-specific code). The page shows the shared
  row `Pass starts at` on the Passes tab.
- The engine takes the value as an argument, not as a field of
  `Finish3DParams`: `finishing3d::contour_from(facets, p, start)` and
  `finishing3d::flat_from(facets, p, start)`. `contour` and `flat` keep
  their signatures and call the new doors with `Auto`. This is the
  `direction_angle` pattern of #3426 (`parallel_with_angle`).
- Only Contour and Flat read the value (`finishing3d::reads_pass_start`).
  They cut nested offset loops of one region (`shadow_pass`), so there
  is a real inside and outside. Auto and Outside cut the outermost
  offset pass first (today's order). Inside cuts the innermost pass
  first. `shadow_pass` collects the runs of each offset pass and, for
  Inside, reverses the sequence of the passes. Inside one pass the loop
  order is not changed. No loop is added or removed.
- Outside is its own forced value, not an alias of Auto in code: the
  reorder reads `start == Inside`, so a later change of Auto's order in
  `shadow_pass` would have to keep Outside outer-first by a new arm.
  Today Outside and Auto give the same moves; the stamp differs (the
  digest writes the forced value), so an op set to Outside generates
  again one time and gets the same path.
- Project calls the same `shadow_pass` with `Some(0)` (one silhouette
  set) and always passes `Auto`.
- No completeness guard or fallback is needed: the change is a pure
  reorder of the loops that `shadow_pass` makes today. Each run goes
  through the same `emit_runs` link law as before (hop to retract or
  clearance, `diagonal_ok` for `ramp`, `straight_ok` for the gap link),
  so each entry and each link between loops is certified by the
  existing laws. Correction (#3627): the plain hop was not certified
  before #3627. With a retract plane below the end of the current
  run, it fed the tool down into the part (the verifier held the
  post). Since #3627 its plane is never below the run end, and its
  move across is over every facet, cleared by `straight_ok`, or at
  clearance. The 2D `level_ring_plan` is not used: its input is a
  pocket band with walls, and the finishing loops are ground loops of a
  shadow region.
- The digest writes the value only off Auto and only for Contour and
  Flat (`digest::write_pass_start`), so every other op keeps its stamp.
- All other patterns do not read the value. The row stays visible and
  enabled. The line under it names the pattern: "Not used with
  <pattern>: this pattern has no inside or outside ring order. Only
  Contour and Flat use it". A saved non-Auto value changes no move and
  is not an error. Reasons by pattern:
  - Waterline, Morph and the ring tails of Parallel, Ramp, Scallop,
    ParallelMulti, SteepShallow and Blend: the rings at one Z are
    disjoint components (CW holes are dropped), so they do not nest.
  - Project: one silhouette set and disjoint drape loops.
  - Radial, Spiral, MorphSpiral, Pencil, Corner and Flow: chords, one
    curve, crease chains or flow lines. These are not rings.
- Geodesic is treated as "not used" in this change. Its rings start at
  the open edge and go out by surface distance, so a start side exists,
  but the map from Inside and Outside to it is not clear. A follow-up
  issue ("Pass starts at: Geodesic start side") records it.

**Why.** The coordinator's brief for #3362 named these choices. The
argument form keeps every `Finish3DParams` literal (motion tests, the
golden builders, multi-axis callers) unchanged and keeps the hunk out of
`finishing3d/params.rs`, which the open #3550 also changes.

**What was dropped.**

- A `pass_start` field on `Finish3DParams`: it touches each struct
  literal of the type in many test files for one value that two
  generators read.
- An Inside or Outside order for Waterline rings by area: the rings do
  not nest, so "inside" has no meaning there.
- A toolpath note for a forced value on a pattern that does not read it:
  the page line says it, and the path is the same.
- A Geodesic mapping (follow-up issue).
