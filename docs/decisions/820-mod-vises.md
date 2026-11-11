## #820 - Mod vises: a fixed and an adjustable jaw block on the plate grid

**What was decided.**

- **Two jaw-block fixtures, not a style of the screw vise** (the
  maker's own instructions, read locally per the issue's design note: a
  plate-mounted modular vise is two separate blocks bolted to the
  plate). Each block is an ordinary meshed fixture wearing
  `FixtureDef::mod_jaw` (`modvise::JawParams`) and a place on its plate
  (`FixtureDef::plate.mount`, #819), so the fixture laws, the riders rule
  (moving the plate carries both, deleting it lets go of both), undo and
  the project file need no new code paths. Dropped: one fixture holding
  both blocks - the blocks bolt to different holes and a plate edit
  re-seats each at its own named hole.
- **The block frame:** origin at the ANCHOR bolt (the hole it is placed
  on) on the block's underside; local +Y toward the jaw it clamps
  against; the clamping face (insert front) at `face_offset + slide`;
  the body centred across on the bolt pattern's mid-span. A pair stands
  face to face, the adjustable block turned 180 degrees.
- **The solid is a conservative envelope:** the side profile (body plus
  the insert slab) extruded across the wider of body and jaw. Bolt and
  pin holes are data (the mounting footprint, mm in the block frame),
  not drawn. Hard and soft inserts are the same slab, recorded for the
  setup sheet.
- **The slide lives in the block's parameters (its mesh), not its
  pose.** The adjustable block's frame stays on its hole, so a plate edit
  (`SetPlate`) re-seats it at the named hole with its slide intact, and
  the place on the plate is still a plain grid location.
- **The pair fit (`modvise::fit_pair`, in the plate's frame):** the
  fixed block stands on the named hole turned a quarter turn (0/90/180/
  270 - anything else is refused: the pair must run along a grid row or
  column). The adjustable block goes on the same grid line, shifted
  across to the grid line nearest to lining the two bodies up, at the
  hole NEAREST the stock that still clears it (a non-negative gap), and
  slides by that gap so its face touches the stock's far side (pinned to
  1e-6 at all four turns). A gap over the slot travel is refused by name
  with the hole, the gap and the travel. The fixed block does not move:
  a gap between it and the stock is reported (MCP `fixed_gap`), not
  closed - the same as #802's opening-from-stock.
- **Refusals by name:** stock in the fixed jaw, stock beside the jaws
  (no overlap with either jaw's width), stock below the plate top or
  above the jaws, no hole past the stock before the grid ends, a bolt
  that lands between holes or off the grid, pins on a plate with no
  dowel bores (or off them), a block hanging off the plate (the #819
  seat law), a plate with no grid, no stock in the plate's setup, two
  blocks of one side, and every nonsense figure (non-finite or
  non-positive sizes, a fixed block with slot travel, a slide past the
  travel, no anchor bolt at [0, 0], a bolt or pin outside the body at
  any slide - a slotted body must still cover its bolts at full travel).
- **One command (`PlaceModVisePair`), one undo step,** composing two
  `InsertFixture`s seated by #823's own seat rule (`seat`: the hole, the
  turn, standing on the plate top, the on-the-plate law). The issue
  brief said "place the fixed block via PlaceOnPlate": `PlaceOnPlate`
  moves an existing fixture, so a fresh pair uses the same seat code at
  insert time instead - same rule, one step.
- **Presets** (generic round figures of our own, no vendor model or
  dimension): "1 in grid 3/8-16" (fits the built-in 12 in and 770MX
  plates), "2 in grid 1/2-13", "50 mm grid M12" (fits the 400 x 300 mm
  plate). Each adjustable block's slots travel one grid pitch, so any
  stock that fits between the jaws is reached from the nearest clear
  hole. No pins on the presets (bolt-located; pins are checked against a
  plate's dowel lattice when a user adds them). The vendor STEPs were
  opened locally only to check the proportions are the same order of
  size; none is committed and none of their figures is used.
- **Project file: one optional `mod_jaw` key on the fixture marker, no
  version bump.** Read leniently the #819 way (raw JSON, parsed on
  apply, a key that does not read or breaks the block law is dropped and
  logged - the steel and its place stay). When #830's shared
  `fixture_keys` reader lands, this key moves onto it.
- **MCP:** `add_mod_vise_pair` (plate, hole, preset, yaw, insert,
  per-block overrides including bolts/pins); it reports both holes, the
  slide, the opening and the fixed gap. The `camcraft-mcp` skill table is
  a process file, left to the next process sync (as #802/#823 did).
- **UI:** a plate's row on the Fixtures page gets "Add mod vise pair…"
  (preset, the fixed jaw's row and column, which way its jaw faces, soft
  jaws, Place pair); each jaw block's row says its side, jaws and slide.

**Split off:** the dual-station block, and editing/re-fitting a placed
pair (`SetModJaw`), are #832.

**Cost to change later.** The fit is one function over `PlateDef`; a
turned-in-model grid (#819's cost note) or non-quarter turns would add a
lattice walk along an arbitrary direction there.
