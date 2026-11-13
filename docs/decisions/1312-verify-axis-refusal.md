## #1312 - verify reads a move's tool axis through one door

**Decided.** `verify` passes the path through a new door
(`crates/sim/src/verify/axis_door.rs`) before any law runs. A stated
axis with a non-finite component, a zero length, or a length that
overflows is refused by the existing `Warning::NonFinite { at }` (its
text now reads "non-finite coordinate or tool axis"). A finite axis whose
`|a|²` stands more than 1e-12 off 1 is normalised once; every law after
the door reads the normalised copy. A refused move keeps its axis as
stated: the door adds the refusal and takes no law's verdict away (the
radial rapid law already convicts a pose that is no direction, and
`radial_rapids::a_rapid_on_a_pose_that_is_no_direction_convicts_itself`
pins that it is never read as upright).

**Why.** The issue's two reproductions: `[1e-12, 0, inf]` read as tilted
and convicted nothing; a 7x-scaled axis lost its `ShankHit`. Normalising
at the door fixes every law that reads `a[2]` or `a[0..2]` as a unit
vector (reach, sweep, posed steel, rapids) at once, instead of patching
each reader. Already-unit axes (within 1e-12) pass bit for bit, through
a borrowed `Cow`, so no existing verdict or golden moves and generator
paths pay no copy.

**Dropped.**
- A new `Warning` variant (`BadAxis`): it would touch the app's match arms
  (`warned_moves`, the MCP job's warning names) for no gain. `NonFinite`
  is already the post-blocking refusal for an unreadable move.
- Refusing a non-unit axis instead of normalising it: a scaled axis
  still names a direction, and `Move::with_axis` / the builder already
  normalise. Refusing would make verify stricter than the CL layer for
  no safety gain.
- Dropping a refused axis to upright (None) in the copy the laws read,
  so the reach law would convict the shank as upright. Tried first: it
  turned the radial rapid law's own conviction of a zero axis into an
  upright reading (red `radial_rapids` test), which is the reading that
  law forbids. The NonFinite refusal already blocks the post.

**Cost to change.** Local to `verify_bodies`. The other public law entry
points (`bands_vs_fixtures`, `rapids_vs_fixtures_spindle`, which the post
gate calls directly) do not pass through the door. They are left as they
are because nothing found can feed them a bad axis (see the issue's
reachability comment).
