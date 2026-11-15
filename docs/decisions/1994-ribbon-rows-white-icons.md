## #1994 - The ribbon's rows get their own heights, and the glyphs go white

**What was decided.** The ribbon has three rows, and each has its own height
and gap (`crates/ui/src/egui_app/theme.rs`):

- the workspace tab row: 24 px (was 16 px, which clipped the tab text);
- a gap of 4 px, then the cell box (30 px);
- a gap of 8 px (was 3 px, and the button labels ran into it), then the
  group caption row: 18 px;
- a bottom gap of 4 px to the ribbon edge.

The ribbon strip is 88 px at the defaults (was 66 px). The tab row is
centred on its full band, so the tab text has equal clear space above and
below.

Every ribbon glyph uses `painted::glyph_ink`: the on-fill ink of its cell
fill, or of the strip ground when the cell is plain. The per-family hue
(`family_tint`) is removed. On the dark ground the glyphs are white. On the
light workbench ground the glyphs keep the dark on-fill ink, because white
does not show on the near-white strip.

**Why.** The maintainer's screenshot (2026-11-15) showed the tab row
without room for its text, the blue glyphs on the gray bar, and the caption
row touching the button labels and the ribbon edge.

**The commit that tinted the glyphs.** `37e7594ec` ("feat(ui): a light
workbench theme and workspace tabs", 2026-11-14) added `RIBBON_ICON_INK`
(steel blue), `family_tint` (the per-family hue of the 2D, 3D, Drilling,
Multi-axis, Turning, Plasma, Wire EDM and Sinker doors) and the `icon_tint`
argument of `painted::cell`. The same commit set `RIBBON_TAB_H` to 16 px and
the caption row to start 4 px below the cell box. Before that commit the
glyph ink was `on_fill(strip_fill)`, which is white on the dark ground.

**Considered and dropped.**
- A white glyph on the light ground: dropped. It would vanish on the
  near-white strip (`#F6F7F9`). The light theme keeps its dark ink.
- A taller cell box (more than 30 px): dropped. It changes the cell
  token in both columns and the clamp range. The 2 px label overflow is
  absorbed by the caption gap instead.

**Cost to change later.** The row heights are tokens. A theme that grows
the cells grows the ribbon (`ribbon_h`). The glyph ink rule is one function
(`painted::glyph_ink`); a new per-family colour would need a new decision.
