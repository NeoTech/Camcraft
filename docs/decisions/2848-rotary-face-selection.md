## #2848 - rotary operations and a face selection

**What was decided**

- Rotary Parallel keeps its cut over the picked faces. The kernel plans the
  raster on the whole model, as before. Then it keeps each cut point where the
  line from the axis out through the ball centre meets a picked facet (the
  radial shadow of the picks). The radial relink builds the links between the
  kept pieces, as it does for the whole raster.
- The axis fit, the tangency law, the gouge laws and the verify read every
  facet of the picked model. Only the cut is bounded.
- A pick resolves to facet indices of its model, not to points. The indices
  name the same facets after the WCS yaw (#760) turns the material.
- A selection that Rotary Parallel cannot use refuses by name:
  - a pick that is not a face (an edge chain, a vertex);
  - picks on two models;
  - a face that does not look away from the axis (an end face, a bore wall);
  - picks that form two or more groups that do not touch;
  - picks that no pass lies over.
- Rotary Contour and Rotary Offset refuse a face selection by name. Before,
  they cut the whole model without a message.
- The generation stamp of a rotary operation with a selection gets a version
  mark. A path that was cached under the old reading (the whole model) is
  stale until the operation is generated again.

**Why**

- The clip plans nothing new. A kept point is a point that the whole-model
  raster already certified. The pattern is the one `rotary_ends::clip` uses
  for a held end (#2343), and `finish_clip` uses for the 3-axis finish.
- The maintainer's triage comment names the defect: the operation ignores the
  selection without a message, and an unusable selection must refuse.
- The issue's "Done means" list asks for a refusal of disconnected picks. The
  kernel can link two groups (a mutation run proved it), but the list is the
  contract. This is my reading of "disconnected"; it can be relaxed later.
- "Non-cylindrical" in the issue is read as "a face that a radial tool cannot
  stand over". A cone or a sculpted face of the turned surface is legal. This
  is my reading, not the reporter's words.

**Considered and dropped**

- Face containment for Rotary Contour. Its rings need a new link between the
  kept pieces: the waterline's plunge column is certified per ring entry, and
  the kind has no clearance radius to retract to. That is new safety logic.
  Follow-up: #2851.
- Face containment for Rotary Offset. A clip is wrong for a roughing ladder:
  a kept piece would plunge into stock that the dropped passes did not clear.
  The correct form is a picked region in wrap space, with the seam law. That
  is new algorithm work (`effort:max`). Follow-up: #2852.
- Generating on the picked facets only. An open mesh has no axis and no
  solid to check against, and the tool would gouge the faces next to it.
- An MCP pick-time door for the rotary refusals. Generation refuses by name
  already; the door is polish.
- A change to the preview region. It stays the model's shadow.

**Cost to change later**

- Low. The clip is one module (`multiaxis::rotary_pick`) and one call in
  `rotary_parallel_over`. The refusals are in `operation::rotary_picks`. To
  permit separate groups, remove the `groups` check. A rotary kind that learns
  to keep to picks replaces its `not_yet` refusal.
