## #2276 - 2D Offset, Trace and Engrave in a setup that holds the part turned

**Decided:** remove the aligned-setup refusal for three more kinds: 2D
Offset, Trace and Engrave. No new transform is added. The other kinds
keep the refusal until a test proves that they read the setup frame.

**Why:** the three kinds take their region from `job::region_for_job`, the
same door that Face, 2D Pocket and 2D Adaptive use. That door reads the
part material (`OpFrame::material_of`, `setup_align::facets_in`), the
heights (`setup_align::part_aabb_in`), the whole-model pocket and the
picks (`selection_geometry_in` with `pose_in_setup`) in the setup frame.
The new tests (`crates/app/tests/setup_align_2d_kinds.rs`) turn an L
prism 90 degrees about X and then 90 degrees about Z. Each kind cuts and
posts the same program as the same part with that turn baked into its
mesh. The part as modelled cuts a different path, so a read of the model
frame fails the test.

**Kept refused, with the reason:**
- Slot: `slot_pick::slot_bound_of` reads the picks through
  `model::transform_of` (the model frame).
- Drill, Bore, Thread: the hole list is params data that the panel
  recognizes from the model. The frame of that recognition is not proved.
- Deburr: its reads are not checked in this PR.
- Probe: `probe_pick` reads the picks through `model::transform_of`.
- Circular, Trochoidal Slot, 2D Chamfer: their regions come from
  `region_for_job` too, but no test proves it yet (2D Chamfer needs a
  picked edge and a chamfer mill in the fixture). Circular and
  Trochoidal Slot have authored coordinates that the model does not change,
  so the "model frame cuts another path" proof needs a different shape.
- The 5-axis and rotary kinds: `selection::drive_chains` reads the model
  frame, and a turned part under a moving tool axis is open design.
- Lathe, plasma, wire and sinker kinds: the non-mill machine refusal
  stays.

**Dropped:** remove the whole kind list and keep only the tilt and
machine refusals. That would post the kinds above without proof, and
Slot and Probe read the model frame: they would cut in the wrong place.

**Cost to change later:** low. The kind list is one `matches!` in
`setup_align::verified_kind`; each new kind needs one call of the test's
`equivalent` helper.

## #2276 (second slice) - hole kinds, Circular, Slot and Trochoidal Slot

**Decided:** remove the aligned-setup refusal for six more kinds: Drill,
Bore, Thread, Circular, Trochoidal Slot and Slot. Slot picks now read the
setup frame: `slot_pick::slot_bound_of`, `confine_slot` and
`slot_from_picks` take the op's setup and place each pick with
`setup_align::pose_in` (not `model::transform_of`). In an unaligned setup
`pose_in` gives the model pose, so the old path does not change.

**Why:**
- Drill, Bore, Thread: hole recognition (`operation::detect_holes`) reads
  the material of the op's setup (`setup_align::facets_in`). The tests in
  `crates/app/tests/setup_align_hole_kinds.rs` turn a plate with two
  holes about X and Z. In the turned setup, recognition finds the holes
  and each kind cuts and posts the same program as the baked part. The
  part as modelled has no upright holes and is refused. A hole list from
  the turned setup, then the alignment set back to As modelled, does not
  drill the part as modelled silently: generation refuses or holds the
  post.
- Circular, Trochoidal Slot, typed Slot: the region and the heights come
  from the setup frame (`region_for_job`, `part_aabb_in`). The tests in
  `setup_align_2d_kinds.rs` show the same program as the baked part, and
  a different path for the part as modelled.
- Picked Slot: `turned_picked_slot_fills_the_turned_face` picks the step
  face of the turned L (flat only in the setup frame). The turned and
  baked documents seed and cut the same slot.

**Kept refused, with the reason:**
- 2D Chamfer and Deburr: no proof test yet (each needs a chamfer-mill
  fixture and a picked edge).
- Probe: `probe_pick` reads the picks through `model::transform_of`.
- The 5-axis and rotary kinds, and the non-mill machines: as above.

**Dropped:** enable Slot only without picks. That splits one kind in the
refusal law. The frame fix of the picks is smaller and has its own test.
