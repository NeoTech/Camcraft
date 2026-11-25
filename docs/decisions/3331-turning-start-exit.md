## #3331 - Turning start and exit points, slice 1 (Facing, Profile Rough, Profile Finish)

**What was decided**

- A turning pick is a model point `[x, y, z]` that the planner reads as
  (station, radius) about the fitted turning axis. This is the lathe (Z, X)
  plane of #3324, question 1. The pick must state its Z. A pick without a Z
  is refused at the command door, by the name of the kind.
- The registry has a new capability, `PickCapability::Meridian`. Facing,
  Profile Rough and Profile Finish have it. It takes a start and an exit.
- Threading says `NotTaken` with its own reason: "the thread start law sets
  the start and the exit of a thread". The G76 start judge is not changed.
- The eight other turning kinds keep the "owed" reason. They are the next
  slices of #3331.
- The pick is laid on the finished path of the kernel
  (`camcraft_motion::turning::with_picks`). The kernels are not changed.
  - Start: one rapid goes from the opening point of the path to the pick.
    The first feed of the path then starts at the pick and ends where it
    ended before. The moves after it are the same.
  - Exit: one feed goes from the end of the last feed to the pick. One
    rapid goes from the pick to the old end point of the path.
  - An arc pass is not changed. The pick feeds to the start of the arc in a
    straight line.
- A pick is refused by name, and never moved, when one of these is true:
  - it is not in air (inside the stock radius and behind the free end, each
    grown by the nose radius), or it is on the chuck side;
  - the rapid to the start, or from the exit, crosses that stock envelope;
  - the feed between the pick and the cut comes nearer to the part than
    nose radius + stock to leave - tolerance;
  - the exit feed runs along the surface that the last pass cut (it must
    lift about 6 degrees or more);
  - the path has canned cycles or control compensation.
- The operation page shows a Start point group and an Exit point group for
  the three kinds, on the Geometry tab. The rows are typed (Z and X). There
  is no viewport pick for a turning point in this slice.
- MCP: `edit_operation` already takes `picks` with a z. Its schema text and
  `list_operation_types` (`start_exit.plane`) now name the lathe plane.

**Why**

- The issue asks that no shipped turning golden changes without a pick.
  `with_picks` returns the path unchanged when there is no pick, before it
  reads a number. A unit test pins that.
- A post-pass on the finished path keeps each kernel's no-gouge
  construction. The new moves have their own clearance laws, and the
  existing turning judges (axial rapids, bar feeds, holder) still see the
  whole path.
- The stock envelope is the box of the part and the setup stock. It does
  not follow the stock that earlier operations removed. This is the safe
  side: a rapid is never permitted through a place where stock was.

**Considered and dropped**

- A start parameter in each kernel (start Z and start diameter of the
  roughing ladder). It changes the ladder of each kernel and its goldens.
  The issue says a pick sets where the first pass starts, and the post-pass
  gives that with no kernel change.
- All eleven kinds in one PR. The issue asks for slices. The blade kinds
  (grooves, Part) leave a slot and need their own exit law.
- A viewport pick. The shared viewport pick snaps to walls (2D Contour). A
  turning point is in air, so a snap to the part is the wrong point.
- The pick moved to the nearest legal point. The design of #3324 says that
  a pick is refused, never moved.

**Limits, and the cost of a change**

- The exit feed is checked against the part and its stock to leave. It is
  not checked against stock that the operation did not cut. A feed through
  such stock cuts it at the feed rate of the last pass.
- The radius is read about the fitted axis. For a part whose axis is not
  on the model origin, the typed X is not the radius. The refusal texts
  say the radius that the planner read.
- A later slice that needs a different law for a kind adds a field to
  `PickLaw`. The file format does not change: the picks use the
  operation-level `picks` key of #3339.
