## #1347 - Re-applied two-axis part picks refuse as "already held"

**Decided:** `pick::align_from_axes` hands back the setup's held
alignment unchanged when the turn the picks call for is below a quarter of
the landing tolerance (`LAND_TOL / 4` = 2.5e-10 rad). `AlignPartToSetup`'s
existing `change_of` then sees no change and refuses "setup '...' already
holds the part that way" - in the Setups page's Apply check and in MCP
`set_part_axes` alike, since both go through `align_from_axes`. The
landing proof still runs on the returned alignment.

**Why:** the rotation is the thing the picks decide; when it is the
identity the alignment must be the held one bit for bit. Re-composing an
identity turn about the part centre left float noise in the origin
(-7.499999999999999 against -7.5), which `change_of`'s exact compare took
for a change and stored as a no-op undo step. A turn under 2.5e-10 rad moves
a point 1 m from the pivot by under 0.25 nm, and is already inside what
the landing proof accepts as "on the axis" (1e-9), so no alignment a user
could tell apart is ever swallowed.

**Considered and dropped:** a tolerant compare in `change_of` (it would
apply to every alignment door - presets, typed angles, locate - and a
positional tolerance there could let a real small move through as "already
held", which the issue forbids); snapping the composed origin to a grid
(changes what a real turn stores); reusing `setup_align::is_identity_rot`
(its 1e-12 bound on w is about 2.8e-6 rad, too loose for this door).

**Cost to change:** one constant and one branch in `crates/app/src/pick/axis.rs`.
