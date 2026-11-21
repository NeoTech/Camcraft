## #2944 - the lean of a wall comes from the frame, not from the sections

**Decided**

The lean exemption of #2936 (`leans` in
`crates/motion/src/machining2d/chamfer_ride.rs`) now uses the move that the
operation frame gives to an upright wall. It does not read the move from
the two sections.

- `wall_lean(frame, depth)` gives that move. With no frame or an upright
  frame, the move is zero. When the tool axis is in the world XY plane, it
  gives `None`, and no loop is exempt.
- `ride_loops` takes the move as a new argument. The app caller
  (`material_source.rs`) computes it from the operation frame and the
  chamfer depth.
- `partly_bevelled` now cuts each top segment at the foot of each wall
  vertex that falls inside it. Then it judges each piece as before.

**Why**

The part of the issue has a 1 mm bevel on the -X side and a 1 mm undercut
lip on the +X side. Its top section is its wall section moved 1 mm in X.
The old test took the move from the area centroids, so it found a lean and
skipped the #2887 refusal. An upright frame leans no wall, so the move must
be zero. Now this part is not exempt.

The second change is necessary too. The top segment of each Y side runs
from X-29 to X31. Its X31 end is off the wall, so the old judgement read the
whole segment as bevelled. No segment was sharp, and the check did not
refuse. With the cut at X30, the piece from X-29 to X30 is sharp (59 mm)
and the piece from X30 to X31 is the lip. The #2887 refusal now holds.

The box under a 10 degree tilt (#2936) still generates
(`framed_stock_chamfer_live_refusal_keeps_hint_without_local_move_number`).
A lean with the wrong sign makes that test fail, so the sign of
`wall_lean` is checked end to end.

**Considered and dropped**

- Keep the centroid move and only compare it with the frame move. The frame
  move alone is sufficient, and the centroid code is then not necessary.
- A tolerance of `MATCH_SLACK` (0.05 mm) on the move. `LEAN_SLACK`
  (1e-4 mm) is stricter, and the sections of an upright box under a tilt
  agree with the frame move far inside it.

**Not decided here**

A part with a real draft wall in a tilted frame is not exempt. It is read
as before #2936. That is the safe reading.

**Cost to change later**

Small. One helper, one extra argument, one caller.
