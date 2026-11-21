## #1443 - V-Carve: depth from the inscribed circle, one tool, a verify mode of its own

**What was decided**

1. The tool positions come from the inscribed circle at each point of the
   outline, not from a Voronoi diagram of the outline. For each sampled
   boundary point the kernel finds the largest circle that is tangent
   there and stays inside the region (the tangent-circle shrink of Ma,
   Bae and Choi 2012, in closed form against line segments). The centre
   is a point of the medial axis (Blum 1967; Choi, Choi and Moon 1997).
   The walk goes one time around each loop, so each branch of the axis
   is ridden from its two walls.
2. The cone leaves a wall allowance of 0.01 mm (`ALLOW`, two times the
   0.005 mm chord tolerance). Each straight feed move is measured against
   the outline before it is written. A move that fails becomes a retract
   and a new plunge.
3. A branch into a corner that turns less than 20 degrees is not ridden
   (the separation angle of Foskey, Lin and Manocha 2003). A tessellated
   circle is then one dip and not a star of short spokes.
4. The maximum depth is `Top Z - Bottom Z`. The start depth is `Top Z`.
   The cone height (`diameter / 2 / tan(angle / 2)`) also limits the
   depth, and the path carries a note when it does.
5. Flat clearing uses the V-bit itself: offset rings at each level,
   inside the wall pass (Held 1991). It is off in the draft.
6. The V-bit is the existing `Taper` tool kind (the chamfer mill). The
   included angle is the tool's `tip_angle`. The operation has no angle
   number of its own.
7. The simulation gets `Mode::Carve`. It has the stand-downs of
   `Mode::Engrave`, and the crossing law is off. The second ride of a
   branch lies on the first within the chord tolerance, so the two rides
   cross at one depth by design. With the crossing law on, the letter L
   is refused (mutation checked).
8. The depth law is `DepthLaw::Mark`, as for Engrave and Trace. A Top Z
   typed under the real surface is a gouge and blocks the post.
9. The MCP surface is the generic `add_operation` tool with the kind
   `V-Carve`. `list_operation_types` lists the kind and its fields. No
   new MCP tool was necessary.
10. The project file gets a new `VCarve` arm, appended, with no version
    step. The kind tag is 44, appended.

11. The branch is not stacked on PR #3058 (Waterjet Profile). That PR
    also appends kind tag 44 and `ALL[44]`. The two rows are appends to
    the same table, not one shared logic. The PR that merges second
    takes tag 45 and the next `ALL` row when it refreshes.

**Why**

- The inscribed-circle walk needs no new dependency and no robust
  segment Voronoi code. Its safety law is local and easy to test: each
  position is the centre of a circle inside the region.
- The allowance and the per-move check make the Prime Directive hold
  without trust in the sampling: a bad link is not cut.
- One tool keeps the first version small. The cone already clears a
  floor correctly, only slowly.

- A stack on #3058 would make the V-carve wait for an unrelated
  feature that has `needs-rework`. A renumber of one appended tag at
  merge time costs less.

**Considered and dropped**

- A segment Voronoi diagram (exact medial axis, each branch ridden one
  time). Dropped for this PR: it needs a robust Voronoi kernel or a new
  dependency. Cost to change later: the walk in `vcarve_walk.rs` is
  replaced; `VCarveParams` and the app side stay.
- A second, flat tool for the floor clearing (the issue names it as an
  option). Dropped: an operation holds one tool in the current model.
  A 2D Pocket with a flat tool on the same region does this work today.
- A refusal for "a tip flat larger than the narrowest feature". Dropped:
  the `Tool` record has no tip-flat field for a taper tool, so the cone
  is sharp in the model and there is nothing to compare. This needs a
  tool-model change first.
- A literature search with Perplexity in this session. The session has
  no web access. The sources are the published papers that the module
  heads name. No code and no output of another program was read.

**What it would cost to change later**

- `Mode::Carve` is one enum arm with four match sites in
  `crates/sim/src/verify/`. A Voronoi walk that rides each branch one
  time can put the crossing law back on.
- The `VCarve` arms in `OpParams` and `ParamsDto` are stored in project
  files. A removed or renamed field is a breaking change.
