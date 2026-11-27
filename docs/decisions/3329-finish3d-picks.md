## #3329 - Start and exit points: the 3D Finish slice

**What was decided.** The 3D Finish (`OpType::Finish3D`) takes a start
point and an exit point on its Parallel pattern. Its capability is
`PickCapability::Raster`, the same as the Face and the 5-Axis Parallel:
a pick chooses a corner of the zigzag raster. The order law is the
raster seam's (`finishing3d::raster_seam::order`, from #3330). The new
entry point is `finishing3d::finish_seamed`, the Parallel raster at any
pass direction with the picks.

- Start: of the four raster corners (each end of the first and of the
  last pass), the one nearest the start pick. The pass order is reversed,
  or each pass runs the other way, so that the cut starts there.
- Exit: the order whose last pass ends nearest the exit pick. With a
  start pick, the exit must be near the end that the start's order gives.
- The geometry of each pass does not change: the same samples, only the
  order and the direction. The links between passes are planned on the
  new order by the raster's own link laws (`march::emit_runs`).
- No pick is the automatic path, byte for byte: the arm calls the same
  generator as before when the seam is empty.

**Refused by name, never moved:**

- a pick on any pattern but Parallel. This is refused at the command door
  (`pick_refusal`: an `edit_operation` that sets the picks on another
  pattern, or that changes the pattern under the picks) and again in the
  generator, for a project file that skipped the door;
- a pick farther than 1.5 tool diameters from every corner, and an exit
  that the start's order cannot end at (the raster seam's refusals);
- a pick on a model with no shallow ground (no raster pass);
- a plunge column at the picked start, or a retract column at the picked
  exit, that comes within the ball radius of the part. The check is
  exact: `support::column_clear`, the distance from the column's segment
  to each facet, no sampling;
- an exit when the waterline rings cut after the raster: the rings end
  the cut, so the raster's last pass is not the exit;
- a picked corner that the area controls clip away.

**Both picks on one raster.** The Deburr slice (#4168) refuses by name
when the start and the exit pick one chain while other chains exist. The
3D Finish has one raster, and the start and exit are its two ends, so
the question is the same: an exit that the start's order cannot end at.
The rule is also the same: a refusal by name, never a moved exit. The
raster seam keeps only the orders whose start is near the start pick and
whose end is near the exit pick. If none is left, it refuses: "exit
point (x, y): a raster that starts at the start point ends at ... - pick
near that end, or clear the exit point". Both picks at one corner are
refused in this way. Keeping both picks by a re-trace (as a closed
Deburr chain does) would cut the raster twice, so it is not done.

**The one rule for a moved corner.** The last two refusals are one rule
(`finishing3d::raster_ends_kept`): the finished path is read again. Its
first cut must land on the raster's first tip when a start is picked,
and its last feed must end on the raster's last tip when an exit is
picked. The motion crate applies it after the rings; the application
applies it again after the area-control clip and the centre-cutting
guard. Any later stage that moves a picked corner is refused by this
rule, not only the two stages that we know today.

**Why the column check is at the picked ends only, and is also in the
raster.** Each raster sample already passed the same exact law
(`support::plunge_support` calls `column_clear` with the same limit), so
a whole raster cannot make this refusal fire. The seam checks it again
at the corners that a pick moves, as the Deburr and 5-axis slices do: a
later change to the raster's ground law cannot then move a plunge under
an overhang without a refusal. The refusal is pinned by an in-crate test
that gives the law the ends directly (`finish_seam_tests`).

**What was dropped.**

- A start or exit on the other patterns (Parallel Multi, Scallop,
  Steep and Shallow, Blend, Ramp, Radial, Spiral, Contour, Waterline and
  the rest). Each has its own pass order and needs its own seam law. The
  waterline ring seam (`ring_seam`) exists for the 5-Axis Contour, but
  the 3D Finish's Waterline pattern adds the land model, the area clip
  and the centre-cutting guard, which need their own pins. They are left
  for a follow-up; a pick on them is refused by name today.
- An exit with the waterline rings that cuts the rings first and the
  raster last. That changes the cut order of the whole operation (rings
  before the shallow passes), which is a larger behaviour change than a
  pick. The user can turn the rings off, or use a separate waterline
  operation.
- A pick read in the setup's frame. The picks are read in the frame of
  the facets that the generator gets, as for the 5-Axis Parallel; a WCS
  yaw turns the picks with the facets (`wcs_yaw`).

**Not in this slice.** The 3D Pocket (the last kind of #3329). The UI has
no 3D Finish start / exit rows yet: the picks are set through MCP
`edit_operation` and the project file, as for the 5-axis slices.
