## #3024 - Clip pick outlines before screen projection

**What was decided:** Clip each highlight segment in homogeneous clip space before the perspective divide. Use the renderer's near plane, `z >= 0`, and a positive `w` limit. Keep the visible portion of each crossing segment. Join only adjacent surviving segments. Preserve the seam of a fully visible closed outline. The selected, viewport-hover and row-hover highlights use the same path.

**Why:** Point rejection removes visible portions of edges that cross the near plane. A point close to the eye can also project far outside the viewport. Segment clipping keeps the outline complete and prevents a false connection across its hidden portion.

**Considered and dropped:** Stroke width and anti-aliasing changes do not correct the projection. Dropping a whole crossing edge leaves gaps. Changing the camera matrix or the GPU projection extends the scope beyond pick highlights.

**Evidence:** Seven tests cover clipped rectangle edges, stroke bounds, the near plane, separate runs, closed seams, invalid points and the positive `w` boundary. Five fail on the unchanged baseline; all seven pass with the fix. Restoring the old code, removing the near-plane guard and removing the positive `w` guard each make the relevant pin fail. Restoring the fix makes them pass. The quick gate and UI clippy pass. No golden files change.

**Cost to change later:** A different renderer depth convention requires a matching clip-plane change and updated analytic projection tests. The display cache and the camera API need no change.
