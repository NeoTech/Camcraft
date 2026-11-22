## #1473 - A picked start point under control or wear compensation (item 4)

**Decided.** A closed 2D Contour now takes a user-set start point when the
compensation is Control or Wear. Before, the pick was refused for both
modes. The ring starts at the pick's projection, as it does in computer
compensation. A new check, `contour_comp_pick::picked_seam`, proves the
compensation seam law at the pick:

- the first cut after the engage and the last cut before the cancel are
  straight moves;
- the two cuts run on one line through the seam, so the offset that the
  control builds on the engage block is the one it withdraws on the
  cancel block;
- under Control only: the programmed part contour also runs straight at
  the seam (its nearest point is not a part vertex, and its edge is
  parallel to the first cut).

A pick that fails one of these is refused by name ("start point (x, y):
control compensation needs the seam on a straight run - ..."). The seam
is never moved to the automatic mid-run seam.

**Why.** The automatic seam meets this law by construction (the midpoint
of the longest edge). A pick can land on a corner arc or a vertex. The
existing `push_span` square check passes a seam on a chord of an outside
corner arc, where the nearest part point is the part vertex. There the
control would engage square to the first part edge, not at the proved
centre point. So the pick needs the stronger check, and the refusal must
name the pick.

**Dropped.**
- Keep the blanket refusal: it blocks a safe and common case (start on a
  long straight wall).
- Move the pick to the nearest straight run: a pick is a wish and is never
  moved silently (#1448).
- Add the part-vertex check to every seam in `push_span`: correct, but it
  changes the automatic path's checks with no defect to fix there.

**Still refused.** A picked exit point under compensation (the cancel
block must leave from the ring's seam), open chains (item 5), tilted
contours (item 6) and other operations (item 7).

**Cost to change.** Low. The law is one function, called only for a
picked ring.
