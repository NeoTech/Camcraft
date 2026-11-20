## #2476 - Carry a WCS XY flip with a Z rotation

**What was decided**

- Keep the existing pass order: WCS flip, WCS rotation, then any
  machine-axis inversion. Bake mirrors the existing setup/table
  coordinates. Control rotates those mirrored coordinates using the yaw
  expressed in the mirrored frame. The existing passes carry arc
  centres, starts and stock bounds with the moves.
- Reverse the controller's G68 / ROT angle when exactly one of X and Y
  is flipped. Keep its sign when both are flipped. Keep that composition
  in the shared Rust passes so every rotation-capable dialect receives
  the same transformed stream.
- Allow a Z-yaw WCS through the app's flip validation. Keep named refusals
  for other rotations and nonfinite placements: "only a finite Z rotation
  is supported with flipped axes". Keep refusals for flipped Z, probing
  and the operation types the mirror does not carry. Keep the existing Control
  capability checks and its refusal with inverted machine axes. A non-XY
  arc in a rotated WCS remains refused by the rotation pass.
- Keep the existing travel and setup-sheet behavior. Travel judges the
  mirrored table-frame path after the controller rotation, independently
  of the output mode. The sheet states the setup's original yaw and its
  flipped axes.

**Why**

A reflection and a rotation do not commute. Write the XY mirror as `F`,
its determinant as `d`, and the original yaw as `a`. The flip pass states
`a1 = d * a`. The rotation pass then produces
`R(-a1) F (p - O) = F R(-a) (p - O)`. The controller applies `R(a1)`
in its mirrored frame, so the physical path is
`F R(a1) R(-a1) F (p - O) = p - O`. One flipped axis has `d = -1`;
flipping both XY axes has `d = 1` and preserves the angle sign. The
existing mirror also carries the arc-direction and cutter-compensation
side changes.

**Considered and dropped**

- Reorder rotation before reflection. A mutation that restored the old
  pass order still passed the Control pin: conjugating the yaw in the
  flip pass makes that order equivalent. The reorder adds no behavior,
  so it was dropped.
- Keep refusing every combined flip and yaw. The existing rotation and
  mirror passes can carry the supported milling cases with the
  conjugated yaw.
- Add separate mirror-angle rules to each post script. A shared stream
  transform gives each dialect the same geometry and avoids duplicated
  rules.
- Add dependencies or another placement representation. The existing
  WCS flags, yaw parameter and record transforms express the operation.

**Cost to change later**

The angle parity and composed physical path are part of the posting contract.
Adding another controller rotation convention must preserve the traced
physical path. Extending flipped Z, probing, tilted work or inverted
machine axes in Control mode still needs its own proof and named
capability handling; this change does not remove those refusals.
