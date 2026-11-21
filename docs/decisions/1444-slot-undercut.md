## #1444 - Slot Undercut: T-slot, keyseat, dovetail and lollipop passes that never move in Z under material

**What was decided**

1. **Four cutter kinds, no new tool fields.** `TSlot`, `Keyseat`, `Dovetail`
   and `Lollipop` join `ToolKind` (file words `t_slot`, `keyseat`,
   `dovetail`, `lollipop`; additive, no `PROJECT_VERSION` step). They read
   the existing fields: `diameter` is the head, `flute_len` the head's
   height, `shank_d` / `shank_len` the neck, `tip_angle` the dovetail's
   flank angle off the bottom face. One reading for every consumer:
   `camcraft_tools::undercut::Undercut` (head shape, neck radius, contact
   point).
2. **One operation kind, `Slot Undercut`** (`OpParams::SlotUndercut`, tag
   44, appended). The centre line is typed: `a`, `b` and a DXF-style
   `bulge` (0 = straight, else a circular arc). The undercut `width` is
   measured at the cutter's widest point, so the pass offset comes from
   the profile's contact radius, not from the tip.
3. **The Z law is a construction law and a verified law.** The generator
   changes Z at one XY station only (`run_in` behind `a` on the entry
   tangent) and checks that on the built path. The verifier has a new
   mode (`Mode::Undercut`) and law (g): any move that changes Z while
   material stands in or over the head's disc is `Warning::UndercutLift`
   (blocking); a neck or holder band with material above its bottom is a
   `ShankHit`. Both are re-read against the stock the program ahead left
   (`acquit_undercut_by_prior`), so a neck slot or an entry pocket that an
   earlier operation cut is air. With no such cut the conviction stands.
4. **An undercut clears no column.** In `Mode::Undercut` the verifier's
   rest model takes no cut from the path, and the column stock engines
   (program verdict, carried stock, rest grids) remove nothing for an
   undercut operation. A column holds one top per (x, y): lowering it to
   the head would remove the roof and report a false gouge, and would let
   a lift through the roof pass as a retract in its own kerf.
5. **The true profile where a solid can hold it.** The solid law judges
   head and neck against the part mesh with the real head: `Prof::Dove`
   (frustum) and `Prof::Ball` (whole ball) beside the existing disc. The
   field stock (`Stock::simulate_undercut`) removes the head and the neck
   as two convex solids (their union is the non-convex cutter), with a new
   exact frustum solid (`SdfTool::Dovetail`). The stock view uses it.
6. **Refusal instead of support where nothing is verified:** a flipped WCS
   and a rotated work offset refuse the kind; an aligned setup and a WCS
   yaw do not list it (refused by their own doors); no 3+2 tilt.
7. **A cylinder against a 'Do not touch' item** (merge of main, #2055).
   Main's avoid law grows each atom by the clearance
   (`swept::rho_expanded`). For `Prof::Dove` and `Prof::Ball` it reads the
   circumscribed cylinder: the head's largest radius over the atom's
   height. `avoid_poly` stretches an atom over a Z range, and a profile
   that is not non-decreasing would then report too small a radius above
   its bottom. The cylinder holds the head at each stretch, so the law
   can refuse a legal move near a protected item by at most the head's
   taper, and never passes a violation. The stretched atom has no `cap`
   allowance: a protected item is never cut. *Dropped:* the exact grown
   frustum and ball, which are correct only for an atom that is not
   stretched and need a second code path for the keep-out.
   Pinned by `crates/sim/tests/verify_avoid_undercut.rs`.

**Why**

- The issue's safety statement is about Z moves under material. A plain
  contour's laws read a vertical retract as "rides its own kerf", which is
  false under a roof. The law had to say so in the verifier, not only in
  the generator, because a path can be edited, pasted or produced by a
  later generator.
- Reusing `shank_d` as the neck keeps every existing reader of the tool
  record correct: `Tool::profile`, the drawn-cutter law and the verifier's
  shank band already describe "a cylinder head on a narrower shank". Only
  the dovetail and the ball needed new profile code.
- A union of convex solids keeps the field engine's convex sweep law
  (golden section on a convex distance) true for each piece. A general
  non-convex revolved profile would need a bracketed search (issue 896's
  research, slice 2) and was not needed for these four heads.

**Considered and dropped**

- *A generic `FormMill` kind with a drawn outline* (issue 896, slice 1).
  It needs the drawn outline to survive `cutter_fit::refit`, a digest of
  the flute outline and a non-convex sweep. That is issue 896's work. The
  four kinds here are parametric and need none of it. Issue 896 stays open
  for corner rounding, back chamfer and slitting saws.
- *A new `SimOp` field for the undercut.* `SimOp` has about 90 literal
  construction sites. The undercut rides `Planned::undercut` in the app
  and `Stock::simulate_undercut` in the sim crate instead.
- *New neck fields on `Tool`.* Not needed (see above); the file format and
  every tool dialog stay as they are.
- *Multiple depth levels.* One level per operation: the head cuts the
  whole undercut height when the cutter matches the slot. A taller
  undercut takes a second operation with another Bottom Z.
- *Round far ends.* The far end links offsets with a straight lateral
  feed. The cutter centre never leaves the band within the shift of the
  centre line, so the undercut is never wider than asked; a blind end is
  left square by up to the shift.
- *Picked geometry.* The centre line is typed (as the Slot's is). A pick
  law for undercut faces is follow-up work.
- *Cutting the neck with the column engine.* It would be correct only
  where the neck slot is narrower than the neck (a collision the law
  already refuses), so it adds nothing.

**Literature search (review follow-up, 2026-11-19)**

The original implementation preceded this search. This follow-up checks its
choices against published material; it does not change that history. The
search used the web search tool because no Perplexity tool was available.
Queries covered T-slot preparation and side entry, dovetail and lollipop
contact geometry, distance-field CSG, and tri-dexel machining simulation.
Only public Autodesk behaviour documentation, a machining handbook, and
published research support the findings below. No camlib implementation,
HSMWorks source, binary, output trace, or bundled asset was inspected.

Checked sources and the choices they support:

1. Walter, [Technical Compendium — Milling, 2024, section D2,
   page D 140, “Strategies for preparing a T-slot”](https://cdn.walter-tools.com/files/sitecollectiondocuments/downloads/global/manuals/en-gb/technical-compendium-milling-2024-en.pdf).
   The indexed section describes a prepared opening that extends below
   the shoulder to leave the cutter shank clear. This supports checking
   the neck against the opening as well as the head against the slot.
   Our additional allowance for each radial pass follows from the neck's
   lateral displacement. The handbook gives machining preparation
   advice, not a proof of collision freedom for an arbitrary entry.
2. Autodesk, [Sketch profile selection](https://help.autodesk.com/cloudhelp/ENU/Fusion-CAM/files/MFG-GEOMETRY-SELECTION-SKETCH-PROFILE.htm),
   and [Is it possible to create a keyway undercut with a Slot operation
   in Fusion?](https://www.autodesk.com/support/technical/article/caas/sfdcarticles/sfdcarticles/Is-it-possible-to-create-a-keyway-undercut-with-a-Slot-operation-in-Fusion.html)
   (2025-11-04). The first describes tapered walls cut with a dovetail
   cutter and clearance at both path ends. The second describes a gouge
   with an ordinary slot path and returning to the entry position.
   These support an explicit entry and exit clearance check. Our chosen
   construction is more restrictive: descend at one outside station,
   enter and leave laterally at depth, then retract at that station.
   This construction and the separate stock check are our design.
3. Autodesk, [Generate a Blend toolpath](https://help.autodesk.com/cloudhelp/ENU/Fusion-CAM/files/MFG-MULTI-AXIS-BLEND-STEPS.htm),
   and [Lollipop cutter fails to completely deburr undercut hole using
   'Flow' toolpath strategy in Fusion 360](https://help.autodesk.com/view/fusion360/ENU/?caas=caas/sfdcarticles/sfdcarticles/Lollipop-cutter-fails-to-completely-deburr-undercut-hole-using-Flow-toolpath-strategy-in-Fusion-360.html)
   (2023-10-30). The first distinguishes surface-based undercut spacing
   from tip-based spacing. The second identifies the ball diameter as a
   limit on access to tight corners. These support using the actual
   profile instead of treating the cutter as a point at its tip.
   Our contact coordinates follow from elementary geometry: a cylindrical
   disc and a narrowing frustum reach their largest radius at the bottom;
   a sphere reaches it at the equator. These coordinates are our
   derivation, not formulas copied from the documentation.
4. Frisken, Perry, Rockwood and Jones, [Adaptively Sampled Distance
   Fields: A General Representation of Shape for Computer Graphics](https://www.merl.com/publications/docs/TR2000-15.pdf),
   SIGGRAPH 2000, pp. 249–254, sections 3 and 4.1. Section 4.1 gives
   Boolean carving of object and tool fields. Section 3 discusses
   sampling resolution and reconstruction error. This supports removing
   swept head and neck solids from a volumetric field that can retain a
   roof. It does not require a one-height stock model or establish our
   sweep search's accuracy.
5. Frisken and Perry, [Designing with Distance Fields](https://www.merl.com/publications/docs/TR2006-054.pdf),
   2006, section 2.2 and Table 1; sections 4.1–4.3. The paper states the
   sign convention for min/max CSG and warns that a combined field need
   not remain a Euclidean distance field. It also describes aliasing and
   resolution limits. Our inference is to sweep each convex head/neck
   piece separately and combine the removed sets, rather than apply a
   convex search to the complete non-convex cutter. This paper supports
   the set representation, not the convex-search convergence claim.
6. Ren, Zhu and Lee, [Feature Conservation and Conversion of Tri-dexel
   Volumetric Models to Polyhedral Surface Models for Product
   Prototyping](https://www.cad-journal.net/files/vol_5/CAD_5%286%29_2008_932-941.pdf),
   Computer-Aided Design & Applications 5(6), 2008, pp. 932–941,
   DOI 10.3722/cadaps.2008.932-941, sections 1 and 6. The paper describes
   tri-dexel Boolean modelling and the work needed to preserve features
   and produce a consistent surface. Tri-dexel is a valid alternative
   for this class of stock representation. We retain the existing field
   engine to avoid adding another representation and reconstruction
   path. A single top height cannot represent air below a roof.

Limits of this search and the retained design:

- These sources support the selected shapes and representation. They do
  not certify our implementation, feeds, engagement, or collision tests.
  The original handbook and convex-search references in module comments
  remain historical provenance; this search does not claim to have
  checked every page of those references.
- A prepared neck opening does not prove that the head can descend into
  it. The outside station still needs the stock check before any Z move.
  Returning along the entry path alone does not prove that path is clear.
- The field is sampled and circular moves are approximated. Small roofs
  and sharp features can depend on cell size and chord tolerance. CSG
  set operations do not provide a global exact-distance guarantee.
- Four ideal parametric heads do not describe arbitrary form cutters,
  edge radii, or every flute/neck transition. Tilted tools and general
  non-convex sweep searches remain outside this operation's scope.
- No source found here supplies the exact straight/circular pass layout
  or proves its safety. That layout is our own construction, checked by
  the generator tests and the independent verification and profile laws.

**Cost to change later**

- The kind words and the op's keys are in saved files from the first
  release that ships them: renaming one is a format step.
- `Mode::Undercut` and `Warning::UndercutLift` (`undercut_lift` over MCP)
  are part of the verify contract and of cached findings.
- Moving to a generic drawn form cutter (issue 896) can keep these four
  kinds as presets: `Undercut::of` is the one place that reads them.

### The lollipop head in the fixture laws (review of `b95c27b2`)

**Chosen:** the fixture laws (c2)/(d2) read
`camcraft_tools::undercut::fixture_geom`. It is `Tool::geom` with the
head band raised to the head's top (the ball's pole at `2r` for a
lollipop). The shank band shrinks by the same amount, so the neck top
and the holder stay at their real heights. The post gate
(`fixture_struck::op_hits`) and the generation steel laws (link routing
and the workholding warnings) use it. The holder stack is still built
from the real `Tool::geom`.

**Why:** a lollipop's `flute_len` is the neck start, below the pole.
The plain stack swept the ball only up to the neck start and the neck
above it, so steel beside the upper hemisphere passed. The band is a
cylinder of the ball's radius, so it encloses the ball and can only
over-report.

**Dropped:** a change of `Tool::geom` itself. Every stack law reads it,
and `Cutter::judged` matches the undercut head to `flute_len`; a change
there moves the gouge and stock laws too. The fixture laws alone had
the gap.
