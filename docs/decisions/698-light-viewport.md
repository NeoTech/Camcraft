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
- **Colour-vision modes** (review of PR #838): protan/deutan and tritan get
  light-ground sets too (`RED_GREEN_LIGHT_GROUND`, `TRITAN_LIGHT_GROUND`),
  changing only the line inks. Their dark-tuned lines read 1.01-2.8:1 on the
  light ground (pale yellow feed, white "open" boundary). Each light set keeps
  its eye's surviving axis (blue-yellow; red-cyan) and moves the pale end to
  near-black, so lightness still carries the pairs. Every must-distinguish
  pair passes the M13.4 CIEDE2000 floor (weakest: protan 25.7, deutan 27.7,
  tritan 24.4), and every line ink clears 3:1 (3.5-14.6:1). The values were
  found with a port of `cvd.rs`'s maths and are pinned by
  `the_light_colour_vision_sets_pass_the_cvd_gate_and_clear_3_to_1`. Stock
  verdict and selection inks are unchanged: they paint the part, not lines on
  the ground.
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
- **Window title bars** (light only): `widgets.open.weak_bg_fill` is the
  window fill and `open.fg_stroke` the text. They were egui's dark defaults
  in every theme, so a light window had a black title bar. Dark keeps egui's
  own values, pinned by `the_dark_title_bar_keeps_eguis_own_open_fill`.
- **Debug info window** draws its own title row (`window_chrome`): the
  collapse arrow hovers grey and close hovers red with a white cross, like
  the OS window's caption buttons. egui's built-in title bar paints these
  glyphs with no hover fill and cannot be styled to do so. It is in this PR
  because the reporter hit it during the light-theme check: on the light
  ground the window's arrow and close glyphs were invisible, then vanished
  on hover. It is a light-theme readability fix, not a new feature.

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
2. The light colour-vision sets were checked by the simulation gate, not by a
   colour-vision-deficient viewer.
3. Part shading and the selected-part amber on the light ground were not
   re-tuned.
4. The other four floating windows still use egui's title bar (now themed,
   but with no caption hover fill). They can adopt `window_chrome::title_row`.
5. The light pixel snapshot golden (`egui_app_light`) will differ. It is
   regenerated only at a cut (pixel-snapshot law); this PR touches no PNG.
6. `theme.rs` is at 402 lines.
7. The light-set literals live in `ground_ink.rs`, not in theme tokens, so a
   theme file cannot change them yet (review note 4: a follow-up).

**Cost to change later.** Each light ink is one value in `ground_ink.rs` or
the light `Tokens` column, and its test.
