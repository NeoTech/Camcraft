## #1559 - 2D ladders enter over the stock carried above their Top

**Decided.** When a 2D Pocket, 2D Adaptive (Spiral and Trochoidal),
Offset, Circular or Contour has its Top below the material standing over
its region, generation reads that material's top - the declared stock's
highest surface over the region, lowered by what the program ahead cut
(the same reading #104's "Top below stock" note already made) - and lays
it over the generation's copy of the approach (`Approach::material_top`,
never saved, like `center_cutting`). The first descent then starts at the
plunge height over that material and feeds down to the first level; the
op says so in a `Carried stock:` note that also gives the depth the first
level engages. Levels still start at Top (#514 untouched), and an op with
Top on the material posts byte-identical.

**Only where the plain entry dives.** The op is generated as before first;
only if that path's entry rapids down under the material standing under
the tool's disc (the declared stock's own top there, lowered by the
program ahead - the same two re-reads the rapid law makes) is it generated
again over the carried stock. A finish after a rough, whose entry falls
through what the rough cleared, keeps its path byte for byte (the user's
end-cap project regenerates and posts identical). A Contour beside a wall
(Outside / Auto) reads the material in the band one tool diameter around
its region, never the part's own footprint.

**Why.** The issue's ask: a user can set Top to the first real cut and
skip the air pass at the faced top, and the entry must not rapid into the
stock above. Reading the material in the application keeps the motion
crate free of the stock and the simulation; the generators only learn one
height.

**Considered and dropped.**
- A saved "material top" op parameter: the stock and the program ahead
  already say it; a saved number goes stale when an op ahead changes.
- Starting the levels at the material top: that reopens #514's decision.
- Carrying whenever the region-wide reading stands above Top: the first
  build did, and it lifted the entries of the end-cap's bore and profile
  finishes (rough's rind and the wall's own cells in the reading) into a
  feed through air the rough had cleared - safe, but a changed program
  for nothing. The dive test above replaced it.
- Slot, trochoidal slot, chamfer, trace and the 3D kinds are not carried
  in this PR: their region is read after generation or they do not ride
  the planar ladders. They keep the #104 note, and the rapid law still
  refuses their entry into stock.

**Cost to change.** One field read in the ladders' entry height; widening
it to another kind is one `carried_top!` call in its generate arm.

**Severity.** The issue was filed as P7. Read from the code: the rapid law
(`RapidInStock`, which holds the post) convicts this entry, so it is a
legal program refused with a workaround (an air pass), not a silent
collision - no P1. A separate silent case found while pinning it (a
clearance plane set inside the stock is never judged) is filed as #2121.

**Review correction (#2129).** Read every descending rapid in the plain
path before deciding that nothing needs carrying. A first-region feed
below the region-wide maximum can be in air over a lower stock step, so
stopping there hides a later entry into taller stock. Keep the existing
local stock and prior-cut readings at each disc; carrying unconditionally
would discard credit for cleared material. Real Pocket and Contour pins
cover both separate stepped-stock regions and retained prior-cut credit.
