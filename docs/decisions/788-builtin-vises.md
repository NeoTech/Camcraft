## #788 - Built-in milling vises: generated geometry through the fixture library's insert path

**What was decided.**

- **A built-in vise is generated geometry on an ordinary fixture.** The vise
  (`app::vise`) builds a mesh from ten parameters and is placed by
  `InsertVise`, which wraps the fixture library's `InsertFixture`. The result
  is a model wearing `FixtureDef`, judged by the existing mesh and box laws.
  There's no new collision code. The parameters ride the marker
  (`FixtureDef::vise`), so every command that already snapshots the marker
  (`DeleteFixture`, `DeleteEntity`, a `TagFixture` re-bind) carries them
  through undo with no extra code. A re-import (`ImportFixture`) over a
  vise drops them: the file's geometry no longer matches.
- **One conservative closed shell.** The solid is the vise's side profile
  (base, fixed jaw, moving jaw at the opening, and a screw and handle
  housing step at the front), extruded across the vise's **overall width**.
  The jaws are drawn as wide as the body. That's a conservative envelope:
  the laws see at least the real steel and never less. The cap is
  triangulated by ear clipping, so the shell is watertight by
  construction (pinned: every directed edge meets its reverse once, and
  the volume is outward and exact). Dropped: separate boxes for the jaws at
  the true jaw width. Touching or overlapping shells aren't one closed
  solid, and extra precision isn't worth a weaker mesh verdict.
- **The vise frame.** x runs across the jaws, centred. y runs along the
  screw: the fixed jaw's clamping face is at y = 0 with its body toward +y,
  and the moving jaw's face is at y = -opening. z = 0 is the floor the part
  sits on, with the base below it. Placing the origin puts the fixed jaw's
  face, centred, on the floor. That's the frame a machinist indicates.
- **Jaw opening from the stock.** "Fit to the stock" (the dialog's default,
  and `fit_to_stock` over MCP) places the vise at identity rotation with
  the fixed jaw on the stock's +Y side, centred in X, the floor under the
  stock, and the opening equal to the stock's Y extent. "Open to stock"
  (`SetVise::opening_from_stock`) re-opens a placed vise as it stands. It
  measures the stock box along the vise's own screw axis, so a turned vise
  works too. It's refused by name when the stock reaches into the fixed
  jaw, lies behind it, or is wider than the max opening.
- **Editing is one command.** `SetVise` regenerates the mesh through
  `UpdateModel` (the revision moves, so caches see new steel) and swaps the
  marker's parameters with it, in one undo unit. It covers the opening
  and any dimension.
- **Refusals by name.** Every dimension must be finite and greater than
  zero, the opening at most the max, the overall width at least the jaw
  width, and the overall length more than two jaws plus the max opening
  plus the housing. Each refusal names the dimension. On load, a vise key
  that breaks the law is dropped and logged, and the facets stay as plain
  steel.
- **Project file: an optional `vise` key on the fixture marker, no version
  bump.** It's absent on every imported fixture. An older build ignores the
  key and keeps the saved facets, so files stay readable both ways.
- **MCP:** `add_builtin_vise` (size, setup, fit_to_stock or
  position/rotation/opening, dimension overrides) and `set_vise` (opening
  or from_stock, dimension overrides). Each is one History step.
- **UI:** "Add built-in vise…" sits on the Fixtures node's menu and on the
  Fixtures page. The dialog shares the fixture library's modal. A placed
  vise's row shows Jaw opening, Set opening and Open to stock.

**The size table** (typical values in our own rounded, generic
proportions; no vendor's model). In the code, inches are converted at
25.4 mm.

| Size | Jaw width | Jaw height | Jaw thick | Max opening | Length | Width | Base height | Housing L x H |
|---|---|---|---|---|---|---|---|---|
| 3 in | 3 in | 1 5/16 in | 1 in | 4 3/4 in | 12 in | 4 in | 2 in | 1.5 x 0.75 in |
| 4 in | 4 in | 1 1/4 in | 1 1/4 in | 6 in | 14 in | 5.25 in | 2.25 in | 1.75 x 1 in |
| 5 in | 5 in | 1 1/2 in | 1 3/8 in | 7 in | 16 in | 6.25 in | 2.5 in | 2 x 1 in |
| 6 in | 6 in | 1 3/4 in | 1 1/2 in | 9 in | 18.5 in | 7.5 in | 2.875 in | 2.25 x 1.25 in |
| 8 in | 8 in | 2 in | 1 3/4 in | 8 1/4 in | 20 in | 9.5 in | 3.25 in | 2.5 x 1.5 in |
| 100 mm | 100 | 32 | 25 | 110 | 300 | 125 | 55 | 35 x 20 |
| 125 mm | 125 | 40 | 30 | 160 | 360 | 150 | 65 | 40 x 25 |
| 150 mm | 150 | 45 | 35 | 200 | 420 | 180 | 70 | 45 x 30 |
| 160 mm | 160 | 50 | 38 | 220 | 450 | 190 | 75 | 50 x 30 |
| 200 mm | 200 | 62 | 45 | 260 | 550 | 240 | 85 | 60 x 35 |

**Sources** (public catalog and retailer spec listings, read for the
ranges only):

- 6 in class: jaw height about 1.73-1.75 in, max opening 8-9 in, overall
  length 16.75-18.9 in, base width about 7.3-8.7 in, overall height about
  4.4-4.6 in (so a base about 2.9 in high). From the spec listings on
  travers.com (6 in vises, e.g. items 61-421-400 and 61-421-172) and
  shop.butlerbros.com (6 in stationary precision vises).
- Metric 125/160/200 class: smooth-jaw heights of about 39/49/62 mm, jaw
  plate thickness 12/16/20 mm, lengths about 360/425/550 mm. From the
  Kesel high-pressure vise brochures (kesel.com) and the RÖHM vise catalog
  (clamping ranges and opened lengths by jaw width).
- 3, 4 and 8 in: the issue's starting figures (1 5/16 in / 4 3/4-5 in;
  1 1/4 in / 6 in; 2 in / 8.25 in). They're kept as given, and the pinning
  test checks them. Sizes in between (5 in, 100 mm, 150 mm) are interpolated.

**Considered and dropped.**

- Shipping the vises as library file entries: they couldn't be
  regenerated at a new opening, and the user's library file would carry
  our meshes.
- A new collision path or a "vise" fixture kind: the issue asks for none,
  and a model with a marker already flows through every law, the project
  file and every fixture command.
- Fitting with a yaw (vise turned 90 degrees): "Fit" always places at
  identity. A turned vise is placed by pose and then "Open to stock",
  which does honour the turn.

**Cost to change later.** The geometry is one function (`vise::mesh`). A
more detailed vise (jaws at the true width, a handle) stays within the
same parameters and commands, as long as it remains one closed shell.
Adding a parameter means one field in `ViseParams`, `ViseDto` and the
dialog grid. Files without it would need a default.
