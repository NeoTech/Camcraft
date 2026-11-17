## #1749 - a re-machining mask's edge is air, not a wall (3D Pocket ladder)

**What was decided.** The 3D roughing ladder erodes the level's UNMASKED
target (the footprint minus the part's islands) by `r + stock_to_leave`,
and only then applies the re-machining mask (M3.1 D) in `mask_center`:
it intersects the mask with `center.offset(r)` (the swept envelope
the unmasked centers can actually reach), opens that standing set by
one lattice cell (`MASK_RESIDUE` with `offset_sharp`) to drop sampling
residue, and keeps the centers whose disc meets that standing set
(`standing.offset(r)`). Before, the ladder intersected the target with
the mask first and eroded that, so the mask's edge - stock that an
earlier op had already cut - acted as a wall a full radius thick.

**Why.** The reopened #1749 case: the user's Setup 1 cuts an outside
profile first (2D Adaptive, containment outside), then the whole-model
3D Pocket on the notch. The profile clears the stock just past the part's
open edges, so the mask boundary sits on the silhouette. The #1772
open-side footprint then had no effect: the cutter centre stayed a radius
inside the mask's edge, and the corner where the notch's inner wall meets
the open edge kept the stock at full height. Setup 2's soft jaw (9) stood
in that ridge (10.83 mm deep).

**Lattice residue and swept reach.** The mask is a column lattice. A cell
that straddles a wall the earlier op cut flush still reads as standing.
Under the old erosion such a strip was never reached; under the new law
it would send the cutter round every wall again, and a duplicate op would
no longer refuse as redundant (`remachining::a_redundant_duplicate_refuses_rather_than_cutting_air`
failed). Likewise, intersecting the mask with `center.offset(r)` rather
than raw `target` excludes unreachable internal corners and any
`stock_to_leave` wall collar left by a prior identical roughing pass, so
a duplicate op with positive `stock_to_leave` also refuses cleanly. The
standing set is opened (eroded, then grown) by one lattice cell
(`MASK_RESIDUE`) with mitered offsets (`offset_sharp`) so rectilinear
grid corners stay exact without adding arc vertices. A real strip
narrower than two cells is ignored; the old law ignored strips up to a
full diameter, so this is no worse, and the carried-stock check still
reads whatever stands. When a level's unmasked `center` is non-empty and
the mask empties it (`masked_empty`), the op reports nothing left to cut
rather than "too narrow".

**Safety.** The new centre set is a subset of the unmasked ladder's centre
set, which the island erosion already proves clear of the part. The mask
only removes centres, so it cannot add a gouge, and the entry/dive
superset laws read a set that is still inside the unmasked one.

**Considered and dropped.**
- Drop the mask for 3D Pocket: simpler, but it brings back the
  air-cutting over every area an earlier op already cleared, and the
  "nothing left to cut" refusal of a redundant op.
- Change only the whole-model 3D Pocket arm in `app`: the defect is in the
  shared ladder, so the Multi-Axis Rough twin and the rest-rough variant's
  coarse leg have it too.
- The rest-pass (`rest`) intersection is left as it is. Its walls are the
  coarse leg's own uncut material, a separate law.

**Cost to change later.** One function (`machining3d::ladder`). A path
after an earlier op now cuts up to a radius past where it stopped before,
over ground that op cleared (air cutting, no material).
