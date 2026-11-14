# #771 - Two orientation dropdowns, one section everywhere

## What was decided

The five scattered alignment sections collapsed into ONE section with
TWO dropdowns - "Plane orientation" (Axes default, Points, Face,
Manual, Cylinder angle) and "Orientation point" (model box, stock
box, manual vertex) - mounted identically on the Setups overview and
each setup's Settings page. The existing row functions draw verbatim
as the expanded content; the completing work is the cylinder angle
(one curved-wall pick names the axis from the wall's facet normals;
it lands on a chosen world axis, turned by an angle about it) and the
per-setup WCS origin placement keeping the rotation.

## Why

The user's law: two dropdowns, the matching settings expanding below
each, existing functions reused, the missing possibilities completed
- and the section everywhere a setup's orientation is edited (the
all-setups view and each single setup's page), looking the same. The
cylinder reading had to be the wall's own geometry rather than a
two-centroid trick, because a plain bore or cylinder welds into ONE
wall whose centre of mass carries no direction: every facet normal
of a cylinder wall is perpendicular to its axis, so the first
normal crossed with the most perpendicular one names the axis
exactly, and every other normal verifies it - a mixed patch is
refused by name, never silently misread (the prime directive).

## Considered and dropped

- **Two wall picks naming the axis by their centroids' run**: only
  works for stepped bores (two separate walls); a plain cylinder is
  one welded wall, so both picks would read the same centre and
  refuse. Dropped for the facet-normal reading, which needs one pick
  and any wall.
- **A least-squares / PCA cylinder fit**: the axis as the normals'
  null direction is exact for a tessellated wall and cheap; a full
  fit buys nothing here and adds failure modes.
- **A new origin command**: `SetWcsOrigin` gained a `for_setup`
  constructor instead (the active-setup behaviour unchanged); the
  rotation is kept because the frame comes from the plane side - the
  two dropdowns are the 3-2-1 setup split.
- **Marking rows behind a collapsed dropdown as "shown"**: the #664
  law already covers it - a method whose row is not drawn is a row
  off screen; the draft disarms exactly as if the page was left.

## What it would cost to change later

The cylinder reading's contract (primary normals, verification
tolerance `NORMAL_TOL`) is pinned by `tests/orient_dropdowns.rs`
(landings on X/Y/Z, the angle swing, the refusals); the section's
mount points and the dropdown semantics by the rewritten structural
twin and `switch_plane_method` in the disarm suites. The temp-slot
law (one id read OUTSIDE the horizontal - a child Ui names a
different slot) is the one that bit: any future dropdown in this code
base must read its id once, outside the layout closure.
