## #3347 - Protected faces and edges on 3D Finish

**What was decided.** Enable protected face and edge picks on Finish3D.
Use the existing `AvoidSpec` to project a full-tool keep-out across the
tip-height interval from the cut bottom to the clearance plane.
Include `AVOID_PAD` in the projection.
Subtract an intersecting keep-out from the existing cutter-centre region.
Reuse `finish_clip::clip_path` for every finishing pattern.
Rotate the keep-out into the generation frame with the inverse WCS yaw.
Use the existing `WcsYaw` API for that rotation.

Keep the existing clipper's refusal when no cut remains.
Add the protected item names to that diagnostic.
Add a path note that protection can leave material unfinished.
Extend the parent PR's final Trace safety guard to Finish3D.
Run this guard after WCS restoration and fixture routing.
Route links and check the full tool against the protected geometry.
Refuse a remaining contact by name.
Preserve the existing clip when protection does not intersect the
cutter-centre region.
Preserve the original moves when protection is absent.

Stack one level on open PR #3462 at `abed47739`.
That PR supplies the shared final clearance guard.
PR #3413 also changes the shared allow-list and support text.
The PR that merges second must refresh those overlapping lines.

Disable the inherited sccache wrapper for this work after it stalled.
Keep cache and process repairs outside this issue.
Keep the branch's upstream snapshot at `f14097855`.
Upstream advanced to `c31384ae4` during the gate.
Refresh at merge time so this change retains its checked tree.
Do not copy reverse-comparison differences from the newer upstream tree
into the issue change.

**Why.** The projected interval protects flutes, shank and holder during
cutting, entry and exit moves with the existing clipping machinery.
The final guard covers links and any moves changed by fixture routing.
The inverse yaw keeps protected geometry in the same frame as the cut.
Named refusals identify the item that prevents safe generation.
Reusing these laws avoids separate safety implementations for each pattern.

**Considered and dropped.** Per-height replanning and new pattern kernels
would require larger changes to the generators.
A blanket refusal would prevent useful cuts beside protected items.
A separate empty-area guard would duplicate the clipper's existing refusal.
Keep that refusal and add the protected names instead.

**Cost to change later.** The conservative height interval can leave extra
material beside a protected item.
A later per-height optimization can reduce that material.
It must preserve the same full-tool, link, yaw and refusal safety pins.
