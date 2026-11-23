## #3011 - CL toolpath transforms: translate, XY turn, A/B turn, vertical mirror

**What was decided.** `crates/cl/src/cl/transform.rs` (re-exported as
`app::cl::transform`) adds `translate`, `transform_xy`,
`rotate_about_axis` and `mirror_plane`. All four use one map,
`p -> M p + t` with `M` orthogonal. An arc keeps its plane only when `M`
maps the plane normal exactly onto plus or minus itself. Its turning sense
flips when `det(M)` times that sign is negative. This one rule gives
G2 <-> G3 under a mirror, keeps G2/G3 under a turn, and finds the XZ / YZ
arcs that a transform cannot keep. A reflection swaps `CompSide`
(G41 <-> G42). Quarter turns use exact sine and cosine, so a 90 degree
map stays on the lattice bit for bit.

- Moves, arc centres and cycles snap to the 1 um lattice. Edge-break
  polylines do not snap: a snap would move a framed cap's frames off
  orthonormal, and `EdgeBreak::is_sane` would then refuse the cap.
- `transform_xy` refuses an XZ / YZ arc at every angle except a half or
  a whole turn (`VerticalArc`). It does not flatten it: a flatten changes
  the move count, and the index records (cycles, compensated spans, torch
  spans) would then point at the wrong moves.
- `rotate_about_axis` flattens every arc at `chord_tol` when the turn is
  not the identity. A rotated vertical-plane arc can turn past a Z
  extremum, and the builder's Z-monotone law for XZ / YZ arcs would then
  be false. With a flatten, the index records cannot follow. So a canned
  cycle and a plasma statement are refused, and a compensated span is
  dropped with a note, as `TiltedFrame::world_path` does. A legacy
  vertical chamfer cone is dropped with a note, as in a tilted frame. The
  map is the same rotation that a `TiltedFrame` with those columns
  applies; the axis goes through `pose::normalize_axis`.
- `mirror_plane` refuses a horizontal plane (`HorizontalMirrorPlane`, the
  reason that `post::wcs_flip` refuses a flipped Z) and a tilted plane
  (`TiltedMirrorPlane`). It refuses tilted moves and plasma statements.
  An XZ / YZ arc mirrors only across a plane whose normal is X or Y.
- `MirrorMode::PreserveClimb(PassFamily)` mirrors, then walks each pass
  backwards. The caller names the path family, because the cl layer
  cannot read the operation from the moves. Contour, profile and
  chamfer reverse. Helical ramp, adaptive, multi-pass roughing and other
  are refused by name. The moves must also read as passes: an entry
  column down, a planar cut at one Z, an exit column up to the same top
  with the same rapid-to-feed height, and rapids on that one clearance
  plane between the passes. The entry column moves to the old cut end
  and the exit column to the old cut start. A compensated span inside a
  cut reverses with it and keeps its original side. Every other shape (a
  ramp or helical entry, a level change without a retract, a cut before
  the first pass or after the last, a link off the clearance plane) is
  refused by name.
- Every transform refuses thread cycles, probe touches and C-axis moves,
  and a non-finite input or result.
- Review #3308: rotary arc transforms retain the bounded 1,024-chord
  sampler. They sample at half the requested tolerance, then check the
  curve error bound for the actual count and the endpoint rounding error.
  `ArcChordTolerance` refuses a result that cannot meet `chord_tol`.
  Circular arcs use their sagitta. Radius ramps use the sampler's second
  derivative bound. This keeps the work bounded and includes coordinate
  rounding in the requested tolerance. Increasing the sampler budget was
  dropped because a named refusal meets the contract without more work.
- Review #3308, collapsed arcs: a rotary chord with equal rounded
  endpoints is dropped only when its tool axis matches the last emitted
  pose. A changed pose is retained as a straight move at the standing
  tip, with its feed and move kind. A distance tolerance cannot permit
  removal of a required rotation. A named refusal was considered and
  dropped because the existing no-motion law permits this pose change.
  Changing this choice later is local to the rotary chord filter and
  its regression test; the public API does not change.
- Review #3308, fine boring: a `CycleKind::FineBore` cycle is refused
  (`FineBoreShift`) under each map whose matrix is not the identity: an
  XY turn and a mirror. Its shift rapid at the hole bottom points in the
  direction that the oriented spindle stop fixes on the machine. A turned
  or mirrored shift drives the stopped tip into the finished wall. A
  translate keeps it. A rotary turn refuses every cycle already. A remap
  of the shift to the new direction was dropped: the stop orientation is
  a machine setting, not a path value.
- Review #3308, `PreserveClimb` leads: the reversed pass plunges where
  the old pass left its cut. The cl layer cannot tell a lead from the
  contour. So the law asks for symmetry: the first cut move and the last
  cut move have one shape (straight or arc) and one length, within 2 um.
  A pass with a lead-in and no lead-out (or a shorter lead-out) is
  refused by name. This also refuses some safe paths, for example a
  closed contour with no leads whose first and last segments differ.
  `GeometricMirror` stays available for them. A tag on each lead move was
  dropped: it changes the `Move` record and every generator.
- A geometric mirror of a thread-milling helix changes the hand of the
  thread (a right-hand thread becomes a left-hand thread).
  `PreserveClimb` refuses a helix, so only `GeometricMirror` gives this
  result. The caller must know it.
- Heights outside the path (clearance and retract planes of the
  operation) do not move with a Z translate. Only the moves, the cycle
  heights and the records in the path move.

**Why.** The issue asks for a pure module shared by patterns, mirror
folders and nesting (#2166). One matrix rule is easier to prove than four
special cases. Refusal is the Prime Directive answer where a record's
meaning does not follow a rigid map.

**Dropped.**
- Flatten of vertical arcs in `transform_xy`: it would need an index
  remap of every index record. A refusal is safe and named.
- Keeping vertical-plane arcs under an A/B turn when the plane is kept:
  the Z-monotone law would need a re-split through `PathBuilder`.
- Reading the path family from the moves: no safe test separates a
  contour from a single-level pocket. The caller states it.
- Moving probe touches: a probe writes a work offset, and a moved touch
  changes what it measures.

**Cost to change.** Local to `crates/cl/src/cl/transform*.rs`. No caller
exists yet. A later caller that needs vertical arcs under an XY turn can
add a flatten with an index remap behind the same function.
