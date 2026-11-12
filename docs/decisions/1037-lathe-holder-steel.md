## #1037 - The lathe holder body judged against workholding, not only part and stock

**Where it stood.** A turning tool's lathe body (#369) was judged on
every move against the revolved part and the standing stock. The
setup's fixtures were read only by the fixture laws, which sweep the
tool record's holder cylinder - 0 x 0 for a turning insert, since
`tool_body::judged` leaves a lathe-frame body's record unchanged. So an
offset holder running into a chuck jaw passed with no finding, while
the op had dropped its "holder not judged" note because a body was
present.

**What was decided.**

1. **A new law beside the holder law, same frame, same moves.**
   `operation/turning_holder_steel.rs` reads each move's meridian
   pieces (the holder law's own `pieces_of`: chords, a thread's helix
   at its radius, an arc's chords) against every fixture box with each
   of the body's clearance spans (`LatheClearance::spans`). A strike is
   a `FixtureHit { fixture_part: Holder }` on that move, which blocks
   the post and is drawn on the struck fixture like any fixture hit.
   Dropped: a new `Warning` variant (`HolderSteelHit`) - it would be a
   new file-format word in every cached op and every match site for
   nothing a user reads differently; "holder at move N strikes a
   fixture" already says it.
2. **Turning steel is revolved; standing steel stays a box in the
   machine frame.** The fixtures' existing split (M13.8 (2): default
   turns with the part, `SetFixtureFixed` stands) decides the reading.
   A chuck jaw is read as its revolved envelope: its stations, out to
   its farthest corner, in to the nearest reach of its corners' box in
   the frame (nearer than the jaw, never farther). A tailstock or
   steady rest is a box in (station, radial, across-plane) coordinates,
   the radial being the plane the kernels write the moves in - the
   plane the post's X is taken in. The band keeps only |y|, so both
   sides of the plane are read. Both readings are conservative; both
   are exact for a box on a spindle along a world axis.
3. **Solved, not sampled.** For a straight tip motion every condition
   ("band stations overlap the box", "band radii reach into the
   envelope") is an interval of the piece's parameter, so a strike is
   found exactly, however short. The holder law samples its part and
   stock skins at a pitch; box steel needs no such compromise.
4. **Contact is contact.** Boxes grow by the gouge tolerance, as the
   fixture laws grow theirs (`point_inside`), rather than shrink as the
   holder law shrinks the part's skin.
5. **The same law at the post gate.** `fixture_struck::op_hits` (the
   gate's live fixture walk and the struck-fixture view) runs it too,
   reading the op's part as generation does and the steel as it stands
   now - a jaw moved after generation is caught at the post, as every
   fixture law is. A part it cannot read is "unjudged" by name.
6. **Steel no law can read is named.** A tagged fixture with no
   readable geometry yields no box and every box law passes over it.
   A bodied turning op now carries a note naming such fixtures
   (`STEEL_UNREAD_NOTE`). Dropped: a note when no workholding is
   modelled at all - every lathe document without a modelled chuck
   would carry it, and the milling fixture laws do not note absent
   steel either; what is modelled is the user's statement of the room.
   Dropped too: noting the machine table or spindle housing - the
   table is a box and read like any standing steel; the spindle
   housing rides a milling spindle, not a lathe holder.

**What it would cost to change later.** Reading a fixture's mesh
instead of its box touches only `read` (the mesh law's acquittal
pattern, #339). A depth on the finding would need the new variant
dropped in (1).

**Overlap.** #904 (reduced-fidelity laws for turning/rotary/multiaxis)
was in flight on main at the same time; this change adds a module and
one call after the holder law in `generate.rs`, and touches
`turning_holder.rs` only to make `pieces_of` visible to its sibling.
