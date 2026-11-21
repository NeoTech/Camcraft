## #2830 - Bore gets a Helical strategy

**What was decided.**
- `BoreStrategy` (`camcraft_motion::machining2d`, new file
  `bore_helix.rs`) is `Rings` (the level ladder, the default and every
  earlier bore) or `Helical { spring_pass }`. `BoreParams` and
  `OpParams::Bore` get the field `strategy`.
- A helical hole: a rapid over the helix start at the clearance height, a
  rapid down to the plunge height over the material (the Lead-in
  `plunge_height`, over the stock top when the stock stands higher), one
  continuous helix on the wall radius (`rc = R - r - leave`, the ring
  ladder's last ring) to the floor, one flat floor circle, a second one
  with `spring_pass`, a feed to the centre at the floor, a rapid up to
  clearance. A through hole's helix ends at the floor less the
  break-through. The helix is arcs of at most half a turn, so each posts
  as one G2/G3 with Z, as the thread mill does.
- The pitch is `stepdown`. The page names the field "Helix pitch" when
  the strategy is Helical. The helix feed is the Lead-in ramp feed when
  set, or else the cutting feed. Climb is CCW, as for the rings.
- Two refusals, in `BoreParams::validate`, before any move:
  - The core door: one helix clears the floor only while
    `rc <= bore_floor_reach` (the flat bottom of a blind hole, the corner
    reach at the floor plane of a through hole). The widest hole is
    `d + 2 leave + 2 reach`: twice the diameter for a flat end mill. The
    message gives the core and the widest hole.
  - The slope door: a helix steeper than `MAX_HELIX_ANGLE_DEG` (15 deg)
    is refused, with the largest pitch that fits.
- The ring ladder's ring-budget door does not apply to a helical hole.
  The mouth door (`rc > 0`) and the counterbore-fold door still apply.
- Detect holes (`bore_reach::keep_borable`) uses the helical core door
  for a helical Bore, and names each hole that it skips.
- Project file: `helical` and `spring_pass` on the `bore` record,
  `#[serde(default)]`, written only when true. No version step.
- Stamp: the params digest writes the tag `0xB8` and the spring flag only
  for a helical Bore. A ring ladder keeps the digest that it had.
- MCP: `list_operation_types` lists `helical` and `spring_pass` for
  `bore` (they are absent from the serialized defaults). An
  `edit_operation` params patch sets them like any other field.
- UI: a Strategy combo and a Spring pass check on the Passes tab, the
  pitch label on the Heights tab, and a Lead-in hint that the level
  links do not apply.

**Why.**
- The issue: a light mill bores counterbores with an end mill by helical
  interpolation. The ring ladder dives at the centre at each level, which
  is the move most likely to chatter or break the tool.
- `rc` is the ring ladder's wall circle, so the finished wall is the same
  in the two strategies, and so are the checks that read it.
- A refusal for a wide hole, not an inner helix followed by rings: the
  issue permits the two. The refusal is small and clear, and the Rings
  strategy already cuts wide holes. A core left by a helix would be a
  loose slug in a through hole.
- The 15 deg limit: a helix in a hole only a little wider than the tool
  has a short turn. At 1 mm per turn, a Ø6 end mill in a Ø6.5 hole
  descends at 32.5 deg, and the bottom face meets the material almost as
  in a plunge. No tool record has a ramp limit to read. 15 deg passes
  the issue's case (3.6 deg) with a wide margin and refuses the near
  plunges.
- Pitch = stepdown: the stepdown is already the axial bite of the
  operation. No new number goes into the file.
- No version step: an old reader drops the two keys and posts the ring
  ladder, which that reader verifies as before. It is a different
  program, not an unsafe one. The Thread kind's `external` and `pitch`
  went in the same way.

**Considered and dropped.**
- An inner helix, then rings out at the floor, for a wide hole: more
  code and a second engagement law for one case the Rings strategy
  covers.
- A separate Helical bore operation kind: a second hole-list kind with
  the same heights, detection and checks.
- A separate `helix_pitch` field, or the Lead-in `helix_pitch` /
  `helix_angle`: a second number for what the stepdown already says,
  and the Lead-in entry fields are for an entry, not for the cut.
- Cap the pitch to the slope limit without a word: the user typed the
  pitch. The refusal gives the value to type.
- A project version step: see "Why".

**Cost to change later.** Low. One enum, one kernel file, two DTO flags
with defaults, one digest tag. An inner-helix plan for wide holes can
replace the core refusal without a file change.
