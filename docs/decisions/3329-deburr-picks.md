## #3329 - Start and exit points: the Deburr slice

**What was decided.** The Deburr (`OpType::Deburr`) takes a start point
and an exit point. It has a new capability, `PickCapability::Chain`: a
point on an edge chain. The picks order the chains before any motion
(`finishing3d::deburr_seam`, `finishing3d::deburr_seamed`):

- Start: the chain nearest the start pick is cut first. An open chain
  starts at its end nearest the pick: it runs the other way when that
  is its far end. A closed chain is re-seated: it starts and ends at
  its point nearest the pick.
- Exit: the chain nearest the exit pick is cut last. An open chain ends
  at its end nearest the pick. A closed chain is re-seated at its point
  nearest the pick, so its lap ends there.
- Start and exit on one closed chain: the chain starts at the start
  point, cuts its whole lap, and runs on along its own edges to the exit
  point. The run-on re-traces cut edges at the same depth.
- The other chains keep the automatic order between the first and the
  last chain.
- A pick with a Z is measured in 3D. A pick without a Z is measured in
  XY.
- The cut geometry does not change: the same edges at the same depth,
  only the order and the direction. A re-seat splits one segment at the
  foot point on its own line. `Toolpath::breaks` stays the automatic
  runs, because those are the edges that the ride cuts.
- No pick is the automatic ride, byte for byte (`ChainSeam::order`
  returns `None`).

**Refused by name, before any motion, never moved:**

- a pick farther than 1.5 tool diameters from every chain (the reach of
  the contour and the 5-axis slices);
- an exit pick on an open chain nearest the end where the start pick
  starts it ("ends at its other end");
- a start pick and an exit pick on one chain while the Deburr has other
  chains ("pick one edge chain", PR #4168 review). That chain must be
  cut first and last, so the operation would end on another chain: a
  silently moved exit;
- a plunge or retract column at a chain end that a pick moved, which
  comes within the tool radius of the part.

**The column law.** At each chain end that a pick moved (the new first
point and the new last point of a reversed or re-seated chain), the
vertical column from one tool radius above the edge up to the clearance
plane must keep a full tool radius from every facet. The check is exact:
`stock_sim::dist_seg_tri` from the column's segment to each facet, no
sampling. Both moved ends of a reversed chain are checked, and the
refusal names the pick that moved the end.

- Why from one radius above the edge: below that the cone works across
  the break, which the ride's contact law sizes and verify's
  Mode::Envelope judges. A column that starts lower always touches the
  two faces of the edge.
- Why a full tool radius above: the descending chamfer mill sweeps its
  own body at that position and the column above it, up to the
  clearance plane. The radius is conservative above the height where
  the cone reaches its full diameter. A false refusal is a refusal, not
  a gouge.
- Why only at moved ends: the automatic ride does not check its columns
  today. A check at each end would refuse automatic paths that pass
  today, and the no-pick path must stay byte for byte.

**What was dropped.**

- Use `PickCapability::Ends`: it names "one end of an open chain or a
  slot". A Deburr chain is often closed, and a closed chain takes a pick
  at any point. A new capability says what the pick means.
- An exit on a closed chain as a lap and a run-on from the automatic
  seam (the 5-axis contour's law): a re-seat cuts the same edges with no
  re-trace. The run-on is used only when the start and the exit are on
  the same closed chain.
- A ball (capsule) column check from the edge point itself: it touches
  the two faces of each edge and refuses each pick.
- Start and exit on one chain with other chains: keep both endpoints.
  The picked chain would be cut from the start foot to a split point,
  the other chains between, and the rest of the picked chain last, to
  the exit foot. That adds a third moved column (the re-entry plunge at
  the split), a split rule for open chains, and a new order law. The
  refusal is simple and safe; the user picks the exit on another chain
  or clears one point. A later issue can add the split if users ask.

**Not in this slice.** The 3D Pocket and the 3D Finish (the other kinds
of #3329). The UI has no Deburr start / exit rows yet: the picks are set
through MCP `edit_operation` and the project file, as for the 5-axis
slices. The "Not taken" line of the Deburr goes away, because the kind
now takes both picks.
