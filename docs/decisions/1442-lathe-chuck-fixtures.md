## #1442 - Lathe chucks as generated built-in accessories

**What was decided.** The 3-jaw, 4-jaw and collet chucks are three new
built-in accessory kinds (`three_jaw_chuck`, `four_jaw_chuck`,
`collet_chuck`), placed by the existing undoable `InsertAccessory` /
resized by `SetAccessory`, so their steel is read by the same fixture
laws (the lathe holder law and the post gate) as every other accessory.
- **Frame:** +z is the spindle axis, z = 0 the chuck's work face; the body
  stands below it, jaws stand on the face (z 0..jaw height), the first jaw
  on +x. A collet chuck's nose runs z -nose length..0 with the body below.
  A chuck placed at a pose is therefore "face at the given point".
- **Shells:** body (an annulus with the through bore) and each jaw are
  separate closed shells, so the mesh law reads them as solids without a
  boolean union.
- **Round drawn conservatively:** the body's polygon (64 sides) has its
  flats on the true circle, so the drawn steel encloses the real body (a
  collision is never missed by the facetting); the bore is drawn on its
  circle (slightly less air - also the safe side).
- **Fit to the stock:** a new placement `OnStock` (MCP `fit_to_stock`)
  closes the jaws (or collet) on round stock standing on a vertical axis,
  on the stock's axis at its underside (a collet the nose length deeper).
  Square or tilted stock is refused by name.
- **Not refused on mill machines:** a chuck on a rotary axis of a mill
  (4th-axis work) is real workholding, so no machine-kind check.
- **Chucks stay `fixed = false`:** they turn with the part.
- **Generic default figures** from general machining knowledge (a 160 mm
  3-jaw, 200 mm 4-jaw, 100 mm collet chuck), no vendor catalogue.
- **Where the code lives:** the chuck figures, their check and their
  shells are in `camcraft-sim` (`chuck.rs`, domain values only); app keeps
  the accessory wiring, defaults and the stock fit. No new crate edge.

**Considered and dropped.** A revolved profile mesh (needs a union with
the jaws, and a revolve with flats inside the circle under-reads steel);
a separate `InsertChuck` command (duplicates the accessory path, its MCP
tool and undo); refusing chucks on mill setups (blocks 4th-axis use).

**Cost to change later.** Inside grip, stepped jaws, independent 4-jaw
offsets and the chuck's rpm limit are additive fields with defaults;
the frame is fixed by the pinned envelope tests and would be a project-file
change.
