## #3627 - the plain hop of the 3D finish runs: never below the run end, across only where clear

**What was decided.** The plain hop between two runs of `emit_runs`
(`crates/motion/src/finishing3d/march.rs`) uses the plane
`max(retract, next run start, current run end)`. The move across is made at
that plane only if the plane is at or above the highest facet of the model,
or if the exact sweep certifier `gaplink::straight_ok` accepts the
horizontal move. If not, the hop goes to the clearance plane.

**Why.** On `main`, the hop used `max(retract, next run start)`. With a
retract plane below the end of the current run (a cavity with Contour: the
top passes come first, the floor passes next), the tool fed down at the
run end into the part and then made a rapid through the part. The test
`a_retract_below_the_high_run_never_takes_the_tool_into_the_part` measured
it: a feed from z 10 to z 7 at (7.35, -7.35), then a rapid at z 7 through
the block. The verifier found a 3 mm envelope gouge and a rapid in stock.
No edit rule refuses a retract plane below the top of the part (the
height resolver and `sane.rs` check only that the value is finite).

**Considered and dropped.**
- Only `hop.max(z0).max(cur.z)`: it stops the move down, but a rapid at
  the run end height can still cross higher material between the two
  runs (a ridge). The move across needs its own check.
- A sampled probe like `diagonal_ok`: it has a probe cap, so long hops
  would change to clearance in the usual case and the goldens would move.
  The exact sweep has no cap, and the "above every facet" test skips it
  in the usual case.
- A refusal of a retract plane below the top of the part at edit time: a
  retract plane inside a cavity is a legal and useful setup (hops between
  floor passes stay low). The generator now makes it safe.

**The usual case is unchanged.** When the retract plane (or the clearance
plane) is at or above the model top, the hop is move for move the old hop:
`cur.z` and `z0` are never above the model top, and no sweep runs.

**Cost to change later.** Local to one branch of `emit_runs`. The other
link kinds (ramp diagonal, straight gap link) and the rings' hop are not
changed.
