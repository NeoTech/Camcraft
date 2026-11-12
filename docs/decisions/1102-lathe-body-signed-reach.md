## #1102 - Lathe holder body read on the holder's own side of the axis

**What was decided.** The holder law (#369) now reads a lathe body along
the holder's own +x, not along the path's signed-meridian +x. The holder's
side is a per-path property (`operation::turning_holder_side::holder_side`):
the side of the axis the path reaches farthest on. The other side may hold
only a past-centre overrun of at most `PAST_CENTRE` = 2.5 mm (a face pass
cleaning the centre pip runs past by up to a nose radius; 2.4 mm is the
largest common ISO turning nose) while the far side runs beyond it. A path
on both sides beyond that, or a path within 2.5 mm of the axis on both
sides, is refused by name ("the path stands on both sides of the spindle
axis ..."). The side is never guessed. On the holder's side (`+1`) the
reading is `Reach::radii(X)`, unchanged. On the other side (`-1`) it is
`Reach::radii(-X)`. The drawn body (`body_axes`, #1038) flips its radial
by the same side, so the drawn holder is still the judged one.

**Which signed X the generators produce.** Every turning generator places
its tips with `Frame::point(rho, station)` at `rho >= 0`, and a face pass
runs to `rho = 0` at most. `SignedMeridian` takes its + side from the
path's first off-axis point. So a generated path has every tip at X >= 0
of its own frame, and the law reads it as before (byte-identical: the
`+1` branch passes `x` untouched). Today no turning op can carry a tip
at X < 0. That is why this stays P2 (a latent defect plus a guard) and
was not re-rated P0. A tip at X < 0 arises only when a path's first point
sits on the other side from where it works, for example a rear tool
starting a hair past centre. Read along the frame's +x, that body would
be mirrored through the axis: a striking lip judged clear.

**Why per path, not per tip.** A rigid holder does not flip as its tip
crosses the axis. Reading the side from each tip's own sign would mirror
the body during a past-centre overrun and misread it by 2|X|.

**Considered and dropped.**
- Changing `SignedMeridian::of` to take its + side from the farthest
  point. That moves the sign convention under the rapids, feeds and
  credit laws too, and their open PRs would have to be re-checked. Here
  the fix stays inside the reach reading.
- Refusing every X < 0. That would refuse a face pass that runs just past
  centre, which is legitimate.
- Taking the tool's nose radius as the overrun bound. A fixed bound above
  every common nose is simpler and can only refuse more, never read a
  body on the wrong side.

**UI/MCP wording.** Main carries no "+x points away from the spindle
axis" text. That wording is in PR #1043 (lathe body editor and MCP), and
with this law it is now true as written: +x is away from the axis on the
holder's side.

**Cost to change later.** The constant and the side rule live in one
small module (`turning_holder_side.rs`), pinned by its unit tests.
