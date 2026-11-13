## #1507 - Geometry-tab row hover highlights the pick strongly in the viewport

**Decided.** The row hover (label, remove button, or anywhere on the row) sets
`State.geo_hover` (ephemeral, no command). A new `State.geo_hover_drawn` holds
last frame's value, and the viewport draws the strong highlight
(`interact::row_hover_segments`: 7 px dark halo + 3.5 px hot-magenta
`Overlay::RowHover`, both themes) from it. The thin 1 px pass
(`hover_features`) no longer repeats the row hover. The reverse direction
(viewport hover marks the row) is not done; filed as a follow-up.

**Why.** The old row hover set `geo_hover` while the properties pane drew,
but the shell cleared it at frame start and the viewport drew before the pane,
so it never reached the draw: the highlight did not work at all. The
one-frame lag with a repaint request on change fixes the ordering without
reordering panes. Magenta is apart from the violet pick, the yellow viewport
hover and the red/blue/orange overlay inks.

**Dropped.** Drawing the viewport after the panes (a layout reorder, risky);
a new app-side resource (the existing `State` ephemeral hover is the
precedent and already the UI's own state); tinting the face fill (the loop
outline reads on shaded and wireframe; a fill is a larger GPU change).

**Cost to change.** The ink and widths are two constants plus one `Overlay`
arm.
