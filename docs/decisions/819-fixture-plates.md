## #819 - Fixture plates: a parametric grid plate, fixtures placed at grid locations

**What was decided.**

- **A plate is an ordinary fixture with a grid on its marker.** A
  built-in plate is generated geometry placed through the fixture
  library's insert path (`InsertPlate` over `InsertFixture`, the #788
  pattern). Its grid rides `FixtureDef::plate` (`plate::PlateLink`: the
  plate's own grid, and/or where the fixture stands on another plate),
  so delete, undo and re-binding carry it with no extra code. There is
  no new collision code: the plate is judged by the existing mesh laws
  (pinned through `posting::verification_gate`).
- **The collision mesh is a solid slab** (12 triangles, holes drawn
  solid): at least the real steel, never less. Holes, dowels and
  counterbores are kept as data for placement and markers.
- **The grid convention** (documented in the module, the dialog, MCP):
  rows are letters along the plate frame's +Y from A (A..Z, AA, AB...,
  spreadsheet-style), columns are numbers along +X from 1, A1 is the
  hole nearest the frame's origin corner. A built-in plate's frame has
  its origin at the plate corner on the TOP face, the slab below z = 0.
  Dowel holes are a second lattice, named the same way. Dropped:
  numbering rows from the far side (the machinist's view from the
  front): the frame-relative rule is the same on every plate and every
  turn of it.
- **Hole count rule:** from the first-hole offset, the lattice runs on
  while a hole keeps at least that offset to the far edge (symmetric
  margins). A plate with uneven margins is entered by its own offset and
  size.
- **Placing on a plate (`PlaceOnPlate`)** puts the fixture's model frame
  origin over the hole (or on the first of a dowel pair, +X toward the
  second), turned by a yaw about the plate's Z, and **standing on the
  plate**: the fixture's lowest point on the top face. This works for
  every fixture whatever its model origin (a vise's frame is its floor,
  with the base below; an imported STEP's origin is anywhere).
  Dropped: putting the model origin itself on the top face - a vise's
  base would sink into the plate.
- **Refusals by name:** an unknown location ("no location M1 (rows A-L,
  columns 1-12)"), a gap of an imported plate, a staggered plate's off
  places, a dowel pair on a plate without dowels, a fixture whose
  footprint (its placed mesh's XY extent) hangs off the plate edge (side
  and amount named), a fixture on itself or on a plate that rides on it,
  a non-finite turn, and every nonsense plate figure (non-finite or
  non-positive, a first hole over the edge or past half the plate, a
  thread as wide as the pitch, dowels running into tapped holes, a
  counterbore deeper than the plate, more than 20000 holes).
- **Moving the plate moves its fixtures, in its one command.**
  `MoveFixture` carries every fixture placed on the moved plate,
  transitively (a subplate carries its own), by the plate's rigid move.
  `SetPlate` (editing a built-in plate's figures) re-seats each fixture at
  its named location on the new grid, refused by name if the location is
  gone or the fixture would hang off. Moving a placed fixture by hand
  takes it off its grid location (the marker change is in the same undo
  step). Deleting a plate (Fixtures page or tree), untagging it, or
  re-importing it without its grid leaves its fixtures where they stand
  AND takes them off it in the same undo step (PR #823 review): the
  place names its plate by model name, and a later plate given the
  freed name must never carry fixtures that were never placed on it.
  A file whose place names a plate it does not hold loads with that
  place dropped and logged. Keying the place by a stable id instead of
  the name was considered and dropped: names are already the identity
  every command and the save format use (#812 lenient load), and
  clearing on the way out plus the load-time check close the hole
  without a new id scheme or a format change.
- **Grid from an imported plate (`SetPlateGrid::detect`)** landed in this
  PR, not split off. It runs the Drill operation's hole recognition
  (`drilling::recognize`, the rule behind MCP `detect_holes`) on the
  model's own facets, groups holes by diameter (5 um), and fits the most
  numerous group to an axis-aligned lattice in the model frame (pitch
  from the smallest gap, every hole within 0.05 mm of its node, at least
  half the nodes filled, gaps recorded by name); the next group that is
  a lattice becomes the dowels. A checkerboard (every hole on one row +
  column parity) is a **staggered** grid: names from the full lattice,
  the off places refused. Holes off any pitch, or no Z holes at all, are
  refused by name. A grid turned in its model frame is refused (not a
  lattice on X/Y); turning the model is the user's fix. One turn fits
  a lattice anyway: a square grid turned exactly 45 degrees is, hole for
  hole, a checkerboard - the same set a staggered plate has. The holes
  alone can't tell them apart, so a staggered read also needs the
  plate's own edges to run along X and Y (the vertices at its least X
  span at least half its Y extent, and the same for Y); a turned plate
  meets its least X at a corner and is refused as turned (PR #823
  review). The refusals of a fit (an off-pitch hole, a lattice too big
  or too empty) say the likely cause: the holes don't line up with the
  model's X/Y axes - a turned plate, or irregular holes. A loaded or
  derived grid must keep its nearest two holes more than a hole apart
  and its pitch above twice the 0.05 mm snap tolerance.
- **The sanity check on a vendor plate STEP** (local only, never
  committed): recognition found the plate's tapped holes as one size and
  detection read them as a staggered lattice with a handful of gaps
  (where other features sit); the odd-sized bores did not form a
  rectangular lattice and were left out. That staggered case is why the
  stagger rule exists.
- **Presets** (our own round figures, no vendor model or dimension):
  12 x 12 in (0.75 in thick, 3/8-16 on 1 in, A1 at 0.5 in); 300 x 300 mm
  (20 mm, M8 on 25 mm, 10 mm dowels on 50 mm between the tapped holes);
  400 x 300 mm (30 mm, M12 on 50 mm, counterbored 18 x 12 mm); and
  "770MX table 28 x 15 in", sized from the shipped
  `tormach-770mx.mch`'s table box (its X/Y travel spans; pinned against
  the file). "Lay it on the machine table" centres a plate on the setup
  machine's table box, on its top.
- **Project file: two optional keys on the fixture marker, `plate` and
  `on_plate`, no version bump.** Both are read leniently (the #812
  lesson): kept as raw JSON in the DTO, parsed on apply, and a key that
  does not parse or breaks the plate law is dropped and logged - the
  fixture and its saved steel stay - never a failed load. A built-in
  plate's stored grid must match its stored figures.
- **MCP:** `add_fixture_plate`, `set_fixture_plate`, `place_on_plate`,
  `detect_plate_grid`, `get_plate_grid`; each edit one History step. The
  `camcraft-mcp` skill's tool table is a process file and is left to the
  next process sync (as #802 did).
- **UI:** "Add fixture plate…" on the Fixtures node menu and page (the
  fixture library's modal); a plate's row shows its grid and what it
  carries; any fixture row gets "Place on plate…" (plate, row letter,
  column number or a dowel pair, turn); an imported fixture gets
  "Detect plate grid". The viewport marks every hole (crosses; dowels as
  plus signs) and labels row A and column 1.

**Considered and dropped.**

- A separate component for the plate data: it would need its own
  snapshot in every delete/undo path the marker already covers.
- Inserting a vise directly at a grid location: place it, then "Place on
  plate" - two undo steps, each one command, and every fixture kind
  (vise, accessory, STEP) takes the same path.
- Snapping a hand-dragged fixture to the nearest hole: placement is
  explicit by name, which is what reproduces a real setup.

**Cost to change later.** The grid is one struct (`plate::Grid`); a
turned-in-model lattice would add an angle there and in `detect`. The
rotary subplate (#821) and mod vises (#820) build on `PlaceOnPlate` and
the carried-fixtures rule.
