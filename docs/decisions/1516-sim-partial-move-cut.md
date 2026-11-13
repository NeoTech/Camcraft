## #1516 - Playback cuts the stock along the move in flight

**Decided.** During playback the stock picture is the stock cut by every
completed move plus the sweep of the move under the playhead from its
start to the tool's drawn position. The partial sweep is made on a copy
of the completed-move stock (`Stock::with_partial`), recomputed whenever
the playhead's fraction changes; the kept stock, its checkpoints and
every verdict are cut by whole moves exactly as before. The fraction is
the timeline's own (`Script::travelled`, the same `leg_frac` that places
the drawn tool), and the sub-move's end is `sim::point_along`, so the
material ends where the tool is drawn, on lines and arcs alike.

**Why.** The issue asks for display only. Cutting a copy keeps the stock
after a completed move bit-identical to today's (pinned:
`seek_to(k + 1, 0.0)` equals `seek(k + 1)` sample for sample), scrubbing
back inside a move needs no checkpoint restore, and the verdict and post
paths never see the partial stock.

**Frame cost.** Only when the playhead's fraction changes: one `Stock`
clone (the cut layers; the mesh layer is shared) and one sub-move sweep,
on the playback worker, then the existing view clone and pack. A paused
playhead costs nothing new.

**Considered and dropped.**
- Incremental sub-sweeps onto the kept stock (cut 0..f1, then f1..f2):
  cheaper per frame, but the stock at the end of the move would be the
  max of several sub-sweeps rather than the one move's sweep - equal in
  exact arithmetic, not provably bit-equal, and scrubbing back would need
  a restore. The issue's "final stock must be identical" law wins.
- Splitting long moves into several timeline segments: changes the
  script, the move index mapping and the move counts other suites pin.

**Rotary and turning ops keep whole-move display.** Their slices are
chorded about a fitted axis from the pose before them; a straight
sub-move would show a cut the machine does not make. Their move in
flight stays uncut until it completes, as before #1516. A follow-up can
add a partial rotary/lathe slice if it is wanted.

**Patch uploads.** The bricks the partial cut touched are sent with the
frame's patch, and re-sent once on the next frame (the "ghost" set), so
an eye holding a patched layer never keeps a partial cut that has moved
on.

**Cost to change later.** Local to `stock_play_head.rs`,
`stock_sim/partial.rs` and the worker's request; nothing persisted.
