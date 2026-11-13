# 1226 - painted tool-icon wiring test

- Chosen: read the frame's painted shapes (textured `Shape::Rect`/`Mesh`) and scroll each row/item into view with `scroll_to_me` before reading it.
- Why: the earlier attempt's "BN-0600 paints no glyph" was a harness artefact. Image widgets only paint when inside the clip rect, and the catalog sits in a clipped scroll area; epaint 0.36 paints images as a textured `RectShape`, not `Shape::Image`. The product wiring was fine.
- Considered, dropped: a taller harness window (the page height is the scroll area's, it did not help).
