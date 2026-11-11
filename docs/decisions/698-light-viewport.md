## #698 - Light theme gets a light viewport

**What was decided.** The viewport ground is a theme token (`Tokens::viewport`:
the dark column keeps today's clear colour, the light column is `#DDE1E7`).
Theme files can set it as `viewport`. The 3D pass clears to the live theme's
ground. Everything drawn over the viewport that assumed a dark ground now has
a light-ground value. The dark column is unchanged in value; each dark ink is
asserted equal to its old constant.

The light values were tuned on screen with the reporter, in a running build:

- **Toolpath moves** (`Semantic::OFF_LIGHT_GROUND`): feed is a deep green
  `[0.08, 0.36, 0.12]`. Rapid `[0.01, 0.31, 0.90]` and plunge
  `[0.90, 0.13, 0.13]` were measured off strokes the reporter drew on a
  screenshot. All three clear 3:1 on the ground.
- **Boundary roles**: saturated include/exclude/open
  (`[0.10, 0.80, 1.00]`, `[1.00, 0.20, 0.70]`, `[1.00, 0.65, 0.05]`).
  These do not clear 3:1, a trade for hue (see Open).
- **Dimmed paths** (unselected, under "Show all toolpaths"): on a light ground
  they fade toward the ground, keeping 60% of their colour
  (`ground_ink::dimmed`). Scaling toward black (the dark ground's rule,
  `OTHERS_DIM` 0.45) turned every hue into near-black ink on a light ground.
  This was the real reason the first colour rounds all "looked the same".
- **View cube and its four buttons** (`ground_ink::cube_ink`): faces are mid
  grey 165-205, a step under the ground. Edges and button outlines are grey
  90. Labels and glyphs are near-black. The Axes button's triad is an
  isometric triad (equal arms, 120 degrees apart), centred, in saturated
  X/Y/Z colours (it was off-centre and pastel in both themes).
- **Corner control hint and playback tool marker**: dark grey on a light
  ground. Both were fixed white.
- **Hover** (light only): `field_hover` equals the viewport grey `#DDE1E7`,
  and egui's own hover uses it too, with dark ink. The dark-column hover
  (steel-blue accent, white ink) is unchanged.
- **Window title bars**: `widgets.open.weak_bg_fill` is now the window fill.
  It was egui's dark default in every theme, so a light window had a black
  title bar.
- **Debug info window** draws its own title row (`window_chrome`): the
  collapse arrow hovers grey and close hovers red with a white cross, like
  the OS window's caption buttons. egui's built-in title bar paints these
  glyphs with no hover fill and cannot be styled to do so.

**Why.** The reporter asked for a light viewport in the light theme. A first
pass derived from the WCAG 3:1 floor read muddy on 1 px lines. The reporter's
on-screen picks are what ships, with the 3:1 floor kept where it held (moves,
overlays, wire, axes, cube ink, hint, marker, hover ink).

**Considered and dropped.**
- Keep the viewport dark in every theme: rejected, it is the bug as reported.
- Thicker toolpath lines on a light ground: not needed once the dim rule was
  fixed. A 1 px line carries the reporter's colours exactly.
- A darker light ground (`#C4C8CE`): tried and reverted at the reporter's call.
- A pale-blue light hover (`selection_bg`), then the cube's grey 185 and 205:
  tried in turn, and the viewport grey was chosen.
- Re-styling egui's built-in window title bar: not possible in egui 0.36
  (the caption glyphs have no hover fill), hence our own title row.

**Open (for the maintainer).**
1. Boundary role inks on the light ground are under 3:1 (hue over contrast).
   They are pinned as chosen by `the_light_palette_is_the_hand_tuned_set`.
2. Colour-vision remaps (`RED_GREEN`, `TRITAN`) on a light ground keep their
   dark-tuned inks. Pinned by `the_light_ground_swaps_only_the_house_palette`.
3. Part shading and the selected-part amber on the light ground were not
   re-tuned.
4. The other four floating windows still use egui's title bar (now themed,
   but with no caption hover fill). They can adopt `window_chrome::title_row`.
5. The light pixel snapshot golden (`egui_app_light`) will differ. It is
   regenerated only at a cut (pixel-snapshot law); this PR touches no PNG.
6. `theme.rs` is at 401 lines.

**Cost to change later.** Each light ink is one value in `ground_ink.rs` or
the light `Tokens` column, and its test.
