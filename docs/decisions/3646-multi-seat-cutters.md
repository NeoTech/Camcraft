## #3646 - Multi-seat indexable face mills and end mills (tools slice)

**What was decided.**
- This PR is the `camcraft-tools` slice of #3646. It adds
  `milling_cutter.rs` (with `milling_cutter_error.rs`,
  `milling_cutter_assembly.rs` and `milling_cutter_tests.rs`). There is no new crate edge, no ECS and no
  file I/O. `ComponentKind` does not change: the cutter facts are a
  separate record (`MillingBody`) next to the #3642 tool item, as the
  #3643 geometry is. #3644 decides where the document stores it.
- `MillingBody` holds the catalog facts of the cutter body: `DC`, the
  cutting edge angle `KAPR`, `APMX`, `center_cutting` and an optional
  seat count. A seat count needs one seat port for all seats, or one
  port for each seat. Without a count, the seat ports are counted.
  Every seat must take the one shared insert (#3642 `check_insert_fit`,
  the same named refusals).
- `DC` is the diameter where the main cutting edges, carried past the
  corner, meet the tip plane. On a round-insert cutter it is the largest
  diameter that the round edges sweep (the outer diameter of the
  bull-nose model).
- The law: the derived tool is never smaller than the real cutter.
  - At `KAPR` 90 the cutting diameter is `DC` and the corner is the
    insert's nose radius (0 for a sharp corner or a wiper facet). The
    model is exact.
  - Below 90 degrees the edge runs outward as it rises. A `DC` cylinder
    is smaller than the cutter there, so a gouge could pass. The cutting
    diameter is therefore the edge's reach at the top of the cutting
    depth: `DC + 2 * flute_len * cot(KAPR)`. The nose radius is kept only
    while it is no larger than the depth and the edge's run-out over the
    depth (then the bull corner on the wider diameter holds the real
    corner); a larger nose is refused by name. The toolpath keeps the
    cutter farther from a wall than the tip needs, and leaves stock. It
    never gouges.
  - A round insert: the corner is half the size code. The code cuts the
    diameter to whole mm (an inch 12.7 mm insert is code 12), so the
    model corner is never rounder than the insert. A smaller corner on
    the same diameter is the larger shape.
  - `flute_len` is `APMX`, no deeper than the shortest edge can rise:
    the size code (whole mm, cut down) times `sin(KAPR)`, or half the
    size on a round insert.
  - The rest of the insert above the cutting depth is held by the
    tool's shank: `shank_d` is the farthest the inserts reach, and the
    shank runs up to the highest point of the inserts. The height bound
    is the longest chord of the insert (from its inscribed circle and
    its smallest corner, or twice the size code plus 1 mm when the code
    states no circle). The radius bound is the top of the main edge
    (longest true edge from the inscribed circle) when the next edge
    turns inward, else the tip radius plus the longest chord. The
    clearance behind the edges keeps the insert inside the circle that
    its edges sweep (that is what the clearance is for).
- In an assembly (`MillingCutter::assembled`, `milling_cutter_assembly.rs`)
  the cutter takes the assembly nodes and resolves them itself (#3643).
  Three rules close the three classes that the #4192 review found:
  - **One source.** The cutter keeps the body and the insert that it
    was derived from. The assembly must mount exactly these two
    components (full equality, not only the diameter). Another insert
    (for example a sharp `SCMT120400` under a cutter derived from
    `SCMT120408`) or another body is refused by name (`Mismatch`).
    Dropped: derive again from the mounted insert inside `assembled`.
    That hides a changed insert from the caller, who holds the old
    cutter.
  - **Every seat.** Each seat port of the body is placed as the
    mounted insert is (the port frame, the mount turn, the insert's
    cutting point). Its cutting point must be on `DC` and in the tip
    plane (0.001 mm), and its axes, read in its own radial frame, must
    be the mounted seat's (1e-6). Thus each seat is the mounted seat
    turned about the spindle axis, and its insert sweeps the same
    envelope. A seat port with no frame is refused. On a body with one
    seat port and a seat count, the other seats are copies of that
    port turned about the axis by definition, so the check of the one
    port covers them. Dropped: a separate envelope for each seat. The
    tool record has one cutter shape, so the seats must agree.
  - **The real profile.** A body band inside the cutting depth must
    stand inside the tool's profile at the band's lowest point: the
    corner arc `r - rc + sqrt(rc^2 - (rc - z)^2)` below `rc`, else the
    cutting radius. The profile widens from the tip up, so the lowest
    point is the narrowest. This covers a bull-nose and a round-insert
    cutter. The band part above the cutting depth is enclosed over the
    tool (`HolderProfile::enclose`) as before.
- The tool record takes the running data (speed, feed, materials,
  cutting rows, catalog facts, hand, holder stand-in) from a base tool
  that the caller gives. The fields of other kinds are reset to their
  inert values. The tool law (`why_not_sane`) has the last word.

**Why.** The issue asks for derived values that feed the gouge and
collision laws (Prime Directive). A literal `diameter = DC` is smaller
than a cutter with a lead angle, so the slice takes the enclosing
diameter. The envelope pin (`the_tool_holds_the_real_cutter`) samples
the real meridian outline (the edges, the nose arc, the whole insert,
a round insert of its metric and its inch size) and asserts that each
point is inside the tool.

**Considered and dropped (left for later work).**
- The app part (`crates/app/src/tool_assembly.rs`): the document
  record, the edit of a shared insert that marks the milling operations
  `Stale`, and the plunge refusal and collision tests on the operation.
  Left for after #3644, which builds the document table and its
  commands now.
- `tool_material` and cutting rows inherited from the insert. Dropped:
  the #3642 insert component carries no material and no cutting rows.
  The caller gives them in the base tool until a component stores them.
- A tapered model (a cone frustum) for a cutter below 90 degrees.
  Dropped: an operation reads a milling kind (end, bull or ball mill);
  a chamfer-mill kind would change which operations take the tool.
- A check of `KAPR` against the seat's style letter. Dropped: the
  ISO 5608 style letter is a turning-holder convention; a milling body
  states `KAPR` itself.

**Cost to change later.** Low. Nothing outside the new files reads the
resolver yet. A changed rule changes the pins in
`milling_cutter_tests.rs`.
