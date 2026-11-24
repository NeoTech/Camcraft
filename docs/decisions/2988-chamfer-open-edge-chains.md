## #2988 - 2D Chamfer selected open edge chains

**Decided:** Treat a 2D Chamfer selection as raw closed loops plus joined
open runs. Machine each open run from air to the first selected end,
through only the selected chain, and from the last end back to air.
Emit one edge-break record for each selected run.

**Why:** This preserves the existing closed-loop generator and the
Contour selection grouping. It also gives an open chain a safe,
vertical entry and exit without closing it or adding credit for the
unselected part of a rim.

**Considered and dropped:** Reusing `chamfer` as if the open run were a
closed loop was dropped because it would add an unselected closing cut.
Entering on the selected endpoint was dropped because a diagonal move
after the cut could cross unselected material.

**Cost to change:** A different lead convention requires a motion
change and new endpoint stock-simulation tests. A new edge-break
representation requires compatible project-file and simulation
handling.
