## #1505 - one pick style, one edge rule

**Decided:**
- A plain or Ctrl click in Edge mode takes exactly the edge (run) under the
  cursor (`feature_click::ask`: `EdgeUnit::Run`); Shift takes the tangent
  chain, which `edge_pick::take_loop` then closes around the face loop. Before,
  every click took the tangent chain, so a click gave one edge, two edges or a
  whole rounded outline by geometry alone: that was the "sporadic" picking.
  Ctrl+click on an edge a stored loop/chain already covers now removes that
  pick instead of stacking an overlapping one.
- Selected picks and the hover are screen-space strokes over the model with
  a dark halo (`pick_ink`): selected 3 px violet (the existing Pick ink),
  hover 5 px amber (the existing Hover ink). Both inks keep their dark/light
  ground variants. The hover here is the viewport hover; the Geometry row
  hover (#1507) is main's `row_hover_segments` pass, kept as merged.

**Why:** the GPU line pass draws 1 px lines that depth-test against the shaded
model, so a selected edge vanished into it; one thick haloed stroke reads on
both grounds. Main's row hover (371db9c7c) stays the one row-hover path.

**Considered and dropped:** thickening the GPU lines (wgpu lines have no
width); a Shift-hover preview of the loop (hover is computed without
modifiers, kept simple); marking the row when its entity is hovered in the
viewport (the optional reverse direction of #1507, left as a follow-up).
