## #3643 - Tool assemblies: gauge length, cutting parameters and the judged body (spindle slice)

**What was decided.**
- This PR is the spindle slice of #3643. It is pure domain code in
  `camcraft-tools`, in five new files: `frame.rs` (a checked rigid
  frame), `component_geometry.rs` (the ports, the cutting point and the
  body of one component), `tool_assembly.rs` (the stack and the
  resolver), `tool_assembly_body.rs` (cutting parameters and the judged
  body) and `tool_assembly_error.rs` (the refusals). There is no new
  crate edge, no ECS and no file I/O.
- The geometry is a separate record (`ComponentGeometry`) next to the
  #3642 `ToolComponent`. The component stays the catalog fact (codes,
  seats, the solid tool). The geometry says where its ports, cutting
  point and body sit. #3644 and #3645 decide if the two are stored
  together.
- Frame convention: a component's frame has its origin on the machine
  side (MCS) and +Z toward the machine, as the tool frame of the
  collision laws. A child's MCS sits exactly on its parent's port frame
  (the axes coincide), as the existing STEP mate in
  `crates/app/src/tool_body_assembly.rs` does. The root's MCS is the
  identity, and the result is moved so that the cutting tip is the
  origin.
- Gauge length is the height of the root's MCS above the cutting tip.
- `projection_mm` is the stickout from the parent's port to the
  cutting tip. It is the same length as `HolderRef::projection` (nose
  face above the tip). Only a solid tool on a plain or Weldon bore (a
  collet or a side-lock holder) takes it. It must be more than 0, must
  leave shank in the bore, and must not push the shank past a stated
  bore stop. Without it, the shank sits fully home on the bore stop
  that the parent's geometry states (`ComponentGeometry::with_stop`,
  the depth from the port frame to the stop). A solid tool's MCS is at
  the back end of its shank, so "home" moves that MCS the stop depth up
  the port's Z. A shank mount with no stickout and no stated stop is
  refused by name. The nose face is never a home position: there the
  whole tool hangs out of the bore, the gauge length is too long by the
  insertion depth, and the holder body is modelled too far from the
  tip, so a holder collision could pass (#4119 review). The built-in
  library holders (`holder_library`) state no bore depth or clamping
  length, so `ComponentGeometry::holder` states no stop, and a solid
  tool in such a holder needs a stated stickout. A shank on a taper
  coupling still sits on the port: the coupling sets its position.
- `z_rot_deg` turns the child about the mating Z axis. It moves the
  frames (and so an off-axis seat), not a revolved body.
- The judged body is the shape the collision laws read today. One body
  is exactly its spun profile's `holder_profile()`, as
  `app::tool_body::build_library` makes it for a library holder.
  Several bodies are united band by band (the larger radius wins where
  two bodies overlap, for example a collet body inside the chuck nose).
  A span that no body covers takes the smaller radius of its
  neighbours, so the bands stay contiguous. Both rules are pinned
  (`overlapping_bodies_keep_the_larger_radius`,
  `a_gap_between_bodies_takes_the_smaller_neighbour`). Each body is checked as a spun profile
  at its place in the tool frame, so a body below the tip or past the
  size caps is refused by name. A body that reaches into a solid tool's
  flutes is refused (the same `HolderProfile::enclose` test as
  `app::tool_body::fits`).
- Indexable cutter: the cutting diameter is twice the distance of the
  insert's cutting point from the axis. The seat frame is that point.
  The corner radius is the insert's nose radius, half the diameter of a
  round insert, and 0 for a wiper facet. A sharp corner (0) makes the
  model larger at the corner than the real chamfered or radiused tool.
  This is the safe side for a gouge check.
- A spindle assembly has one root, one active cutting item, and one
  child on each port. A port may be left unnamed only when the parent
  has one port. Fit refusals reuse the #3642 checks (`check_fit`).
- `ComponentGeometry::holder` makes a built-in library holder a
  component: MCS on the gauge line, one port on the nose face, and the
  entry's own outline.

**Why.** The issue asks for derived values that feed the gouge and
collision laws. A separate shape would need its own proof. The
equivalence pins
(`a_collet_chuck_assembly_is_judged_as_the_library_holder`,
`a_weldon_holder_with_the_shank_home_is_judged_as_the_library_holder`)
show that a plain tool and holder give the same gauge length and the
same bands as today's `HolderRef`. The frame algebra is textbook rigid
body math; the coordinate-system roles are the public ISO 13399
concepts. No standard text is copied.

**Considered and dropped (left for later issues).**
- The lathe side (`LatheClearance`, turning orientation 1-8, the CRP
  of a turning insert, `MAX_TIP_GAP`). Dropped from this slice: the
  turning frame and the lathe body law are a second half of the same
  size. Left on #3643.
- STEP, STL and OBJ mesh bodies, and `MCS_INSERT` as
  `CsysRole::Mounting` in `step_csys.rs`. Dropped from this slice: they
  are app code and need the ECS body record. Left on #3643 and #3644.
- A full `Tool` record for an indexable cutter (kind, flutes, axial
  depth). Dropped: #3646 owns multi-seat cutters and their `Tool`
  fields. This slice gives the diameter and the corner radius only.
- A projection on a taper or an ER seat. Dropped: there the position is
  set by the coupling.

**Cost to change later.** Low. Nothing outside the new files reads the
resolver yet. A changed rule changes the pins in
`tool_assembly_tests.rs`.
