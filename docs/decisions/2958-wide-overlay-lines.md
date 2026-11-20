## #2958 - Viewport overlay as wide anti-aliased lines, with MSAA

**What was decided.**

- The viewport overlay draws as screen-facing quads, one 32-byte instance for each segment (two f32x3 points and two RGBA8 colours). The vertex shader makes the six corners. The fragment shader fades the sides and the ends over one pixel. The old 1 px `LineList` overlay pipeline is removed.
- The overlay includes toolpaths, height planes, region outlines, triads, the ground grid and the other overlay lines. All of these use the one width.
- The default width is 1.5 pt, multiplied by the display's pixels per point. The choices are 1, 1.5, 2 and 3 pt.
- Each wide-line vertex moves toward the eye by `LIFT_WIDTHS` = 3 line widths of world distance. The world size of one pixel comes from the `mvp` matrix, so the lift is correct in perspective and in orthographic projection.
- Each end of a line extends half the line width past its point. Two lines that meet thus close the corner and do not overlap past it.
- MSAA is a setting: Off, 2x or 4x. The default is 4x. All the viewport pipelines and the traced-stock composite use the same sample count. A change rebuilds the pipelines and the multisample and depth targets. The resolved colour target stays, so the egui texture stays bound.
- The windowed shell requests `TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES` when the adapter has it, and reads the adapter's sample counts for the colour and depth formats. Without the feature, only WebGPU's guaranteed counts (1 and 4) are offered. The browser build always offers only 1 and 4.
- An anti-aliasing entry for a count that the device does not offer is disabled, and its hover text gives the reason. A pick of such a count is refused by name.
- A kept count that the device does not offer falls back to the highest offered count below it. The status line says so once.
- The two settings are per-user view settings in `view.json` (`app::view`), with the origin-name switch. They are not in `prefs.json`. They are not on the undo history and do not make the document unsaved, the same as the theme.
- The two settings are in the viewport's Display menu (#2818), in its Graphics group, after the projection. They were first in the Theme door. A manual test found that place wrong: line width and anti-aliasing are not part of a theme. #2818 had added the Display menu with a Graphics group, so they moved there. A general Settings window is proposed in #3025.
- Like the other Display menu entries, a pick is a direct `State` method call (`select_line_width`, `select_antialias`), not a queued command.

**Why.**

- A `wgpu` line is always one hardware pixel wide and has no anti-aliasing. Toolpaths were hard to see next to the 3 px egui triad (#195).
- 2 pt and more made the dense paths into one solid area when the camera is far. Thus the default is 1.5 pt.
- Without the lift, half of a wide line that lies on a face failed the depth test. In the pinning test, 292 px were visible on the face against 628 px for the same line in the air.
- An instanced segment uses 32 bytes against 48 for the old vertex pair. At the overlay limit (512k segments) this is 16 MiB against 24 MiB.
- For the fallback, a lower count never uses more memory than the user asked for.
- `view.json` already holds the viewport's look (the origin names). `prefs.json` was already past the 400-line file limit.

**Considered and dropped.**

- Draw the paths with the egui painter, as the triad is drawn. This loses the depth test and is slow on large toolpaths.
- Six full vertices for each segment (the trial). This needed approximately 135 MB at the overlay limit.
- A per-line width class for the ground grid. The grid already fades at its rim, so it uses the one width. This can be added later with a byte in the instance colour.
- A fallback to a higher sample count. This uses more memory than the user asked for.

**Cost to change later.**

- The width and the lift are uniforms and constants in `viewport/wide.rs`.
- A new sample count is one row in `Antialias` and one check in `offered_samples`.
- The file words are `"1"`, `"1.5"`, `"2"`, `"3"` and `"Off"`, `"2x"`, `"4x"`. A word that the build does not know reads as the default.
