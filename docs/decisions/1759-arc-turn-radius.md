## #1759 - Arc turns and their radius on Face, 2D Contour and Slot

**What was decided**

All three kinds can round their corners safely. Each kind has the two
parameters of #1733: `blend_turns` and `blend_radius`.

- **2D Contour** had `blend_turns` already. It gets `blend_radius`. No
  radius is the tool radius, as before. The corners that can turn on an
  arc are the corners that turn toward the air. That law is the law of
  `fillet.rs` and it did not change: a corner that turns toward the part
  stays sharp and the notes name it. A convex outside corner is such a
  corner, so the issue's concern (material left on an outside corner)
  cannot occur.
- **Face** had `blend_turns` already. It gets `blend_radius`. No radius
  is the semicircle of half the row spacing, as before. A radius smaller
  than half the row spacing gives two quarter arcs with a straight step
  between them. The step stands one radius outside the sharp step, away
  from the footprint. A radius larger than half the row spacing gives the
  semicircle: the radius can become smaller, never larger.
- **Slot** gets `blend_turns` and `blend_radius`. Each row-end corner of
  the serpentine pass turns on one tangent arc (`fillet_corners`). No
  radius is the tool radius.

The parameter door (`sane.rs`) refuses a radius below 0.01 mm on the
three kinds. The generators refuse a radius larger than the tool radius
(`corner_arc_radius`). The project file writes the keys only when they
are set. The cache digest reads the radius only when the option is on.

**Why**

- Slot: the pass is in the cutter-center stadium, and the stadium is
  convex. A fillet is inside the triangle of its corner and its two
  tangent points. That triangle is inside the stadium. Thus no arc
  center goes outside the stadium and no arc cuts past the wall.
  `fillet_corners` proves each triangle again against the tessellated
  stadium and keeps a corner sharp if it cannot prove it.
- Slot: an arc moves the tool away from the row end, and the row end is
  on the round end of the slot. The arc leaves corner rest there. The
  note states the largest rest, as the pocket kinds do.
- Face: a turn that stands outside the overrun line cannot enter the
  footprint. The reach past the overrun line is the radius, and the
  radius is not larger than the tool radius. This is the law that
  `blend.rs` already has for the semicircle.
- Face: with a radius, rows spaced wider than the tool diameter can turn
  on arcs too. Without a radius they stay sharp, as before, because the
  semicircle would reach more than a tool radius past the overrun.

**What was considered and dropped**

- Face with "no radius means the tool radius" (the issue text). Dropped:
  each file that has `blend_turns` on a Face would change its path. No
  radius stays the semicircle. The Face page names it: "Arc radius =
  half the row spacing".
- Face with the turn inside the overrun line. Dropped for the reason in
  `blend.rs`: the row ends would stand closer to the footprint than the
  verifier's boundary law permits.
- Slot with the join on the round end of the stadium itself (an arc
  about the slot end). Dropped: it is tangent to the rows only when the
  rows are at the two walls, so it is not a general corner arc.
- Slot named as "cannot round safely" because of the corner rest.
  Dropped: the rest is not a gouge, it is stated, and the pocket kinds
  have the same trade.
- A refusal when the Face radius is larger than half the row spacing.
  Dropped: the last row of a face is usually closer than the stepover,
  so almost each face would be refused.

**What it would cost to change later**

- The file keys are new and optional. A change of their meaning needs a
  project-file migration for the files that set them.
- The Face meaning of "no radius" is in `blend.rs` (`plan_reversals`)
  and in one UI label. A change to the tool radius changes the path of
  each blended Face without a radius.
