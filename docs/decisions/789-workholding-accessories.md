## #789 - Built-in workholding accessories: parallels, 1-2-3 and step blocks, strap clamps, soft jaws

**What was decided.**

- **The #788 vise pattern, unchanged.** Each accessory (`app::accessory`)
  is generated geometry on an ordinary fixture: `InsertAccessory` wraps
  the fixture library's `InsertFixture`, the parameters ride the marker
  (`FixtureDef::accessory`), and `SetAccessory` regenerates the mesh
  through `UpdateModel` and swaps the marker in one undo unit. There's no
  new collision code: the box laws, the mesh law and #489's routing judge
  an accessory like any meshed fixture. A resize keeps the kind (a strap
  clamp can't become a block). `TagFixture` keeps the parameters on a
  re-bind and `ImportFixture` drops them, as for a vise.
- **One enum, five kinds.** `Accessory` is `Parallels`, `Block`,
  `StepBlock`, `StrapClamp` or `SoftJaw`, each with a few dimensions in
  millimetres. The project file holds it as an optional `accessory` key
  on the fixture marker, tagged by `kind` (no version bump: an older build
  ignores the key and keeps the saved facets). On load a key that breaks
  the law is dropped and logged, and the facets stay as plain steel.
  Dropped: a separate DTO mirror per kind. The enum is already a plain
  data type, and the load check is what keeps a file honest.
- **Shells that touch, never overlap.** The mesh law's inside test is a
  ray-crossing parity, so two overlapping shells would read their shared
  volume as outside. Every accessory is one or more closed, outward
  shells with disjoint insides. A strap clamp's stud is drawn below the
  strap (table to clamp height) and above it (strap top to the stud
  top), never through it; the heel block stands flush under the strap's
  end. Pinned by the mesh law's own 2-cycle test, the exact outward
  volume and the bounds of every kind.
- **Conservative where it matters.** A 1-2-3 block's holes, the strap's
  slot and the stud's thread are drawn solid. The stud is a 16-sided
  polygon whose flats touch the stated diameter, so it encloses the true
  circle. "Stud above the strap" covers the nut and washer; set the stud
  diameter to the washer's to model those wider.
- **The frames.** z = 0 is every item's underside. Parallels: two bars
  along x, centred, the gap between their inner faces centred on y = 0.
  Block: centred. Soft jaw: centred in x, its clamping face at y = 0 and
  its body toward +y (the vise's fixed-jaw voice). Step block: steps
  rising toward +y from y = 0. Strap clamp: the toe tip at x = 0 on the
  table, the strap running toward +x, centred in y.
- **Parallels fit under the stock in a built-in vise.** Given a vise and
  "stock top over the jaws", the height is the jaw height plus that
  amount less the stock's height (all along the vise's own z). The pair
  stands on the vise floor against both jaws (spacing = opening less two
  thicknesses), and the vise drops so its floor is that height under the
  stock - in the same undo unit as the insert (a `MoveFixture`). The
  stock is the setup's, so the steel moves, not the stock. Refused by
  name when the stock already stands that high, or two parallels don't
  fit in the opening.
- **Soft jaws are a standalone block, not a vise option.** #802's vise
  draws each jaw as its whole body (plate plus `jaw_body_depth`, from the
  #802 review), merged into one extruded profile. A soft jaw on the vise
  would have to reshape that profile while #802 was still changing it.
  A soft-jaw block (width, height, thickness) placed at the jaw face lets
  the user size machined soft jaws today. The vise's own jaws stay drawn
  underneath, which errs on the safe side. A "soft jaws" vise option can
  come later inside `vise::mesh` with the same parameters.
- **Refusals by name.** Every dimension finite, over zero and at most
  5000 mm; a step block 1 to 50 steps; a strap clamp's heel block
  shorter than the strap and its stud on the strap between the toe and
  the heel block. Each names the accessory and the dimension.
- **#489's routing.** #489 (fixture-aware link routing) is not on main
  yet, so the pin uses the very law the router consults: a link laid at
  z 50 across the stud is struck (`fixture_struck::op_hits`, the post
  gate's per-op law), and the link the router would lay - up at its
  start, across at the clamp box's top plus its 5 mm gap, down - is
  clear. The same height over the strap beside the stud is clear too,
  because the mesh acquits what the box (topped by the stud) convicts.
- **Defaults** (common inch sizes, our own generic figures): parallels
  1 x 1/8 x 6 in, 4 in apart; block 1 x 2 x 3 in; step block 6 steps of
  1/2 in rise and 3/4 in run, 1 in wide; strap clamp 6 x 1 1/4 x 5/8 in
  strap, 1/2 in stud standing 1 1/2 in over the strap at 2 1/4 in from
  the toe, 1 1/4 in heel block, 1 in clamp height; soft jaw
  6 x 2 x 3/4 in; parallels fit 1/4 in of stock over the jaws.
- **MCP:** `add_accessory` (kind, setup, position/rotation or
  `under_stock_in_vise` + `above`, `steps`, dimension overrides) and
  `set_accessory`. Each is one History step.
- **UI:** "Add accessory…" on the Fixtures node's menu and the Fixtures
  page, sharing the fixture library's modal. Each placed accessory's row
  shows its figures from its own draft (a map by name, like #802's
  per-vise opening drafts) and "Apply sizes".

**Considered and dropped.**

- Strap clamps fitted to the stock automatically: where a clamp goes is
  the machinist's choice. A pose (toe tip on the table at the part's
  edge) and the clamp height say it in two numbers.
- Steel modelled with holes (1-2-3 block holes, the strap slot): less
  steel than the envelope for no gain in what a path may do.
- A stud running through the strap as one cylinder: overlapping shells
  break the inside test (above).

**Cost to change later.** Each kind's solid is one arm of
`accessory::mesh::solid`; a new kind is one enum variant, its
`default_of`, `dims_mut` and `solid` arms, and its label.
