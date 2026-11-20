## #2818 - Viewport Display menu: display modes, Look at, graphics switches

**What was decided**

- The viewport toolbar has a 13th button, "Display", after the wireframe
  switch. It opens a menu above the toolbar with three groups: Display mode,
  Look at, Graphics.
- The four display modes are those of #2818, in the maintainer's words:
  "Shaded, with edges", "Shaded", "Wireframe", "Shaded, with hidden edges".
  #2636 names a different fourth mode ("Wireframe, see-through"). This PR
  does not add it. #2636 stays open for that mode.
- The display mode choice moved out of the Browser pane ("Solids" radios).
  #2818 says that the choice moves to the viewport menu. The Projection
  radios stay in the Browser pane and the menu has their twin.
- The draw order is one pure function, `draw_list::plan`. `View3D::render_with`
  follows the list. The lists of "Shaded, with edges" and "Wireframe" are the
  same as before.
- "Shaded" drops the wire of the model and of the stock. #2636 says that a
  mode applies to the model, the stock and the fixtures in the same way. The
  stock still shows as its translucent faces.
- "Shaded, with hidden edges" draws the cached model edge list a second time
  with the depth test reversed and 50% alpha. No new buffer and no rebuild
  on a frame.
- "Look at a face" arms one click. The click takes the first visible
  triangle under the cursor and turns the view to its normal with
  `Camera::toward`. The view does not pan or zoom. The click selects
  nothing. A miss changes nothing and says so. Each case disarms.
- The Graphics group has: "Grid" (new field `show_grid`), "Origin axes"
  (the existing `show_world_axes`), and the projection (one of two).
- The display mode and the grid switch are saved in the per-user `view.json`
  with the gizmo names (`ViewPrefs.display`, `ViewPrefs.show_grid`). The
  display mode is a string key, so `camcraft-app` does not know the UI
  type. A missing key or an unknown key loads the default.
- The wireframe switch stays. One press gives wireframe from each mode. The
  next press gives "Shaded, with edges".

**Why**

- One pure draw list lets a test read the passes of each mode without a GPU
  ("Done means" of #2818), and the renderer cannot differ from the test.
- The hidden edges reuse the cached edge list, so the frame cost rule holds
  (`crates/ui/AGENTS.md`, "Frame cost").
- Each setting draws through the one wgpu pass that the native window and
  the browser shell share. No entry needs to be hidden on wasm.
- The menu was written from the group list in the issue text. No SolidCraft
  screenshot, asset, text or look was used (`crates/ui/AGENTS.md`, "Icons
  and assets"). The Display glyph is our own three slider rows.

**Considered and dropped**

- Replace the wireframe switch with the Display button (the ribbon keeps 12
  buttons). Dropped: it removes a one-press switch that users have.
- Keep the "Solids" radios in the Browser pane with four modes. Dropped: two
  doors for one choice, and the issue says the choice moves.
- "Look at" on the current selection in place of an armed click. Dropped:
  the selection is a body, not a face, outside an open operation.
- A `DisplayMode` type in `camcraft-app`. Dropped: the app crate holds the
  document; a display mode is a UI value. The file stores a key.
- Ground shadow, ambient occlusion and the quality preset in this PR.
  Dropped: each needs a new render pass and a frame cost measurement. The
  issue permits the split. The follow-up issue is named in the PR body.
- "Look at" for a picked plane (a work plane that is not a model face). It
  is in the same follow-up issue.

**Cost to change later**

- A new display mode is one enum variant, one `draw_list::plan` branch and
  its words in `display_mode.rs`. The stored keys must stay stable.
- Moving the menu to a different anchor changes `display_menu::draw` and
  `layout_window_display_menu` only.
