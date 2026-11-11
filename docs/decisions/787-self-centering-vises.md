## #787 - Built-in 5-axis self-centering vises, centred on the stock and checked in tilted setups

**What was decided.**

- **A self-centering vise is a style of #788's built-in vise, not a new
  fixture kind.** `ViseParams` gains `style: ViseStyle` (`Standard` or
  `SelfCentering { min_opening, lip_height, lip_depth }`). Everything
  #788 built is reused unchanged: `InsertVise` / `SetVise` over
  `InsertFixture` / `UpdateModel`, the `FixtureDef::vise` marker, the
  per-vise opening drafts, the vise row, and the MCP tools. Only the law,
  the mesh, the fit and "open to stock" branch on the style
  (`vise::self_centering`). Dropped: a separate `FixtureDef` field with its
  own commands, DTO, MCP tools and UI row. It doubles the code, and two
  markers on one fixture could disagree.
- **Its own frame.** The origin is midway between the jaw faces, at the
  floor, centred across the jaws; the jaw faces sit at y = +-opening/2.
  "Fit to the stock" puts the origin at the stock's x/y centre on its
  floor and closes the jaws to the stock's y extent. That's where a
  self-centering vise puts its jaws.
- **Geometry.** The side profile is a body block the full overall length
  under the floor, two jaw blocks `jaw_body_depth` deep up to jaw height,
  and an optional dovetail lip. It's extruded across the overall width,
  so the jaws are drawn as wide as the body (conservative, as in #788).
  Repeated and collinear corners are dropped before ear clipping, so a
  vise closed to 0 mm (the 75/80/125 mm rows clamp from 0) becomes one
  merged jaw block, and jaws at the body's ends run flush with it. Both
  stay a single closed shell.
- **The dovetail lip is drawn solid, flush with the clamping face.** It's
  a step `lip_depth` deep and `lip_height` above the jaw top, with its
  face in the jaw's face plane. A real dovetail leans over the part above
  an undercut; filling the undercut and keeping the face vertical never
  shows less steel than the real jaw. Both lip figures are zero (no lip)
  by default; the user sets them to their jaw set. Dropped: modelling the
  dovetail angle. It needs the part's tenon geometry, and it would carve
  air out of the envelope.
- **No screw housing.** On a self-centering vise the screw runs inside
  the body. Its housing figures must be 0; a non-zero one is refused by
  name, so an MCP override can't quietly add steel the user never sees.
- **The clamping range has a floor.** `min_opening` (the 52 mm row clamps
  from 3.5 mm) is refused against: an opening outside min..max, or a fit
  or "open to stock" on stock narrower or wider than the range, is named
  with the range.
- **"Open to stock" refuses stock off the vise's centre.** Both jaws
  close on the centre. Drawing the jaws at the far side of an off-centre
  stock would put the near jaw's steel short of where the real jaw stops,
  which under-reports steel. Within 1e-6 mm of the centre (float dust on
  a fitted pose) it closes the jaws; otherwise it says to Fit first.
- **Tilted setups use the existing posed laws.** No new collision code.
  The vise is a meshed fixture, so `fixture_struck::op_hits` (the post
  gate's per-op law: box broad phase, then the #339 slice 3 mesh
  refinement of the posed stack) judges it. It's pinned in the posed
  frame by a tool leaning 30 deg toward a jaw, tip low beside the stock:
  struck, while the same tip upright clears the jaw by 3 mm. The same
  lean from the centre clears. End to end, a 3+2 face indexed B 10 posts
  clean over stock clamped in the vise, and raising the jaws into it
  blocks the post.
- **Project file.** Optional `self_centering` key inside the `vise` key
  (`min_opening`, `lip_height`, `lip_depth`), with no version bump. A
  build without #787 reads such a vise as a standard vise with zero
  housing, which breaks its law, so the build keeps the saved facets as
  plain steel (logged), never a wrongly regenerated vise.
- **MCP.** `add_builtin_vise` lists the four rows in its `size` enum and
  describes the centred frame. `dimensions` takes `min_opening`,
  `lip_height` and `lip_depth` (refused by name on a standard vise). The
  vise view reports `style`. No follow-up issue is needed.
- **UI.** The dialog shows a second size row, "5-axis self-centering".
  Picking one swaps the housing fields for Min opening and the two
  dovetail lip fields, and the pose fields read "Centre X/Y".

**The size table** (typical values in our own rounded, generic
proportions; no vendor's model). All in mm. Jaw height is above the floor
the part sits on; base height is the body, table to floor.

| Size | Jaw width | Jaw height | Jaw thick | Jaw body | Clamping | Length | Width | Base height |
|---|---|---|---|---|---|---|---|---|
| 52 mm self-centering | 52 | 10 | 8 | 15 | 3.5-38.5 | 75 | 52 | 28 |
| 75 mm self-centering | 75 | 12 | 12 | 25 | 0-40 | 100 | 75 | 44 |
| 80 mm self-centering | 80 | 15 | 12 | 25 | 0-127 | 182 | 80 | 50 |
| 125 mm self-centering | 125 | 20 | 15 | 25 | 0-208 | 260 | 125 | 69 |

**Sources** (public product spec listings, read for the ranges only):

- 52 mm: a compact self-centering mini vise listed at 51.5 mm jaw width,
  3.5-38.5 mm clamping range, 52 x 75 x 38 mm overall (5thaxis.com,
  V5275). The row takes 75 mm length and 38 mm overall height (28 + 10).
- 75 mm: a compact self-centering vise listed at 75 mm jaw width, 0-40.3
  mm clamping range with jaws forward, 75 x 100 x 56 mm nominal
  (5thaxis.com, V75100X). The row's height is 44 + 12 = 56.
- 80 mm: a self-centering vise family at 80 mm jaw width with 120-200 mm
  grip-to-grip openings, e.g. a 160 mm assembly at 182 x 80 x 65 mm
  (mate.com, DynoGrip-52 M5MVC-160080). The row keeps the issue's 0-127
  mm range on that 182 mm body (127 + 2 x 25 = 177 fits), 65 mm high.
- 125-130 mm: 5-axis vises listed at 0-205 mm clamping range, 210 x 125 x
  89 mm (lang-technik.de, Makro-Grip 125). The issue's 0-208 mm "on long
  bodies" is kept, on a 260 mm body: the law needs the jaws, drawn as
  solid blocks, to ride on the body at the max opening (208 + 2 x 25 =
  258). The overall height is 89 mm (69 + 20).
- Jaw heights above the floor (10-20 mm) and jaw body depths are our own
  conservative choices; listings rarely state them. The user edits any
  figure in the dialog or over MCP.

**Considered and dropped.**

- A jaw drawn at the true jaw width beside a narrower body: two touching
  shells are not one closed solid (#788's reasoning). The jaws are drawn
  as wide as the body, which on these narrow vises is the jaw width
  anyway (width = jaw width on every row).
- A full-length screw channel or an open space under the stock between
  the jaws: the body below the floor is drawn solid. A tool never goes
  under the stock's floor, and solid is the conservative reading.
- Sharing `vise::mesh::solid`'s extrusion by turning it into a public
  prism helper: that would edit #788's file, and #789 is being built on
  the same base. The 20-line prism lives in `self_centering.rs` beside
  its profile instead. It can fold into one helper once both stacks have
  merged.

**Cost to change later.** The profile is one function
(`self_centering::profile`). A tenon-aware dovetail, a stop under the
part or a narrower body below the jaws all stay within the same style
and commands, as long as the result is one closed shell. Another
self-centering figure is one field in `SelfCentering`, the DTO and the
dialog's style rows.
