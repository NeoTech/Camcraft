## #729 - MCP: lay_part_on_face

**Decided:** `lay_part_on_face { face, setup?, plane?, flip?, place? }` takes
`face` as one `{model, id}` object (an array is accepted and passed on, so the
command's own "pick one flat face" refusal names the more-than-one case). It
returns `outside` (the command's own `plane_side` words) beside the stored
alignment and description, as `lay_part_on_points` does.

**Why:** the single-pick shape matches the issue text ("one face id") and the
Setups page; reusing the command's refusals keeps GUI and MCP wording equal.
The `plane` option parser was factored into `plane_of` for the new tool; the
older tool is left untouched to keep the diff small.

**Considered and dropped:** a `frame` option (a face gives no in-plane
direction); a curved-face refusal test (no curved fixture in the MCP tests;
the command's refusal is pinned in `points_align.rs`/pick tests).

**Cost to change:** low; a tool schema and one function.
