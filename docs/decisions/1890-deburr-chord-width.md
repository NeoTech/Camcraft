## #1890 - 3D Deburr: chord width, Constant Depth and multi-pass breaks

**Decided.**

- `DeburrParams.width` and `OpParams::Deburr.width` are the chamfer chord
  `w = 2 s sin(theta / 2)`. The setback `s` is the cone's radius at the
  edge's level, which was the old meaning of `width`. The tip's drop is
  `s / tan(half-angle)`. `Chamfer2D.width` does not change.
- The project file goes from v74 to v75. The issue text said v61 to v62,
  but main was already at v74. A v74 Deburr `width` is multiplied by
  `sqrt(2)` at load, which is the chord of its setback on a 90-degree edge.
  The cut depth on a 90-degree edge does not change.
- The new settings are in one value, `finishing3d::DeburrBreak`: `mode`
  (Constant Width / Constant Depth), `depth`, `profile` (Chamfer / Round)
  and `passes`. `DeburrParams` and `OpParams::Deburr` each get one field,
  `sizing`. The file keys are flat: `break_mode`, `break_depth`,
  `break_profile` and `passes`. They are always written. When a key is
  absent, the value is one Constant Width chamfer pass.
- Constant Depth uses `s = d / cos(theta / 2)`.
- The wedge angle of a crease is `180 - (angle between the face
  normals)`. A rim has one face, so it is sized as a 90-degree edge (the
  legacy ratio).
- Range limits: Constant Width sizes wedges from 30 degrees up, so
  `s <= 1.93 w`. Constant Depth sizes wedges up to 150 degrees, so
  `s <= 3.86 d`. The ride skips an edge outside the range, and a note
  gives the count. Without these limits a knife edge (Constant Width) or
  a nearly flat seam (Constant Depth) needs an unlimited setback, so the
  vertical cone would cut very deep.
- An edge outside the range is not an unreachable edge. `Unreachable
  edges: Refuse` does not refuse for it. The edge is skipped with its
  own note.
- Runs chain only edges with the same drop (grouped to 1 nm). Thus each
  run has one depth and one declared `EdgeBreak`. On a curved part this
  can give more runs and more plunges. The old single-depth record is
  kept.
- Passes: pass `k` of `n` cuts the setback `s * k / n`, from the outer
  pass to the final pass. All passes of a run are done before the next
  run starts. Each pass retracts to clearance and plunges again. Only the
  final break is declared. The earlier passes stay above it.
- Round is refused by name in the kernel. Each Deburr tool is a sharp
  cone (the generator accepts only a chamfer mill), so Round always
  refuses until ball-tipped cutters are supported. It is not refused by
  the parameter sanity check, so the value can be saved.
- The stock-sim record bound (`declared_width`) for Deburr is the largest
  setback that the size law can give (`DeburrBreak::max_setback`). The
  flute-length check in generate uses the same value, so it is
  conservative.

**Dropped.**

- A step-width parameter for the passes. A pass count gives exact equal
  steps and leaves no remainder pass.
- One drop for each point along a run, with the run's deepest drop
  declared. That record would allow cuts deeper than the shallow part of
  the run, which makes the sim check less strict.
- A wedge angle clamped silently to the range. That would cut a break of
  the wrong size with no message.

**Cost to change later.** The range limits are constants in
`crates/motion/src/finishing3d/deburr_size.rs`. If ball-tipped cutters
come, the Round refusal in `DeburrParams::validate` must become a check
of the tool.

**Review corrections for PR #3220.** These replace the earlier sizing and
version decisions above. Size each cut from the cone contacts with both
faces. The symmetric wedge setback alone does not give the actual cone
radius on a general face orientation. Refuse a real face without a valid
cone contact, including an underside that a vertical cone cannot reach.
An open rim uses an orthogonal virtual second face. Do not use the old
symmetric drop when the real contact calculation fails: that would state
the wrong cut size. Tests measure the actual contacts independently.

Use project v76. Main already uses v75 for machine controller data.
Migrate Deburr widths from v75 and earlier, and preserve the controller
fields. Check every pass against the tip window before emitting moves.
Normalize the accepted numeric skin vertically to clearance before travel.
