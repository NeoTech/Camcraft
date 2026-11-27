## #3196 - Face to size from numbers (command, MCP tool, Stock page group)

**What was decided.**

- One command, `FaceToSize`, holds an optional `SetStock` and a normal
  `CreateOperation` (a Face on `SrcRef::Stock`) in one `BatchCommand`.
  One undo takes back the two. No new operation kind.
- The Z values are program Z. The command adds the world Z of the active
  setup's placed WCS (0 when none is placed) and stores the heights as
  `Fixed` world Z. The top Z is the first stage top (`top_z`), the final
  Z is `cut_z`, and the depth per pass is `stepdown`.
- A stated box stock is centred on the old stock's XY centre (on the WCS
  XY when there is no stock), with its top at the top Z.
- The command refuses by name, at construction:
  - a final Z at or above the top Z (the plain Face door accepts a level
    window, so this guard is new);
  - a depth per pass, stepover, feed or plunge feed not above 0;
  - an empty or unknown tool, or a stepover wider than the tool (ribs
    stay between the rows);
  - no stock and no stock size;
  - a top Z under the stock top (the first pass would cut deeper than
    the depth per pass);
  - a final Z at or under the stock bottom (the face cuts through).
- `CreateOperation` validates against the world. When the command sets a
  new stock, it applies that `SetStock`, builds the operation, and
  inverts the `SetStock` again before it returns. The world is unchanged
  after construction, also on a refusal.
- The UI form is a "Face to size" group on the Stock page, under the
  stock actions. The MCP tool is `face_to_size`.

**Why.** The issue asks for one undo step over the existing Face, with
refusals at construction. Program Z is what a machinist types at the
control. `Fixed` heights keep the cut at the same place on the block if
the WCS moves later. `WorkOffset` heights were dropped: they refuse when
no WCS is placed, which is the empty-project case of the issue.

**Dropped.**

- A new "quick op" kind: it would add a file-format word for no new
  toolpath.
- `TopOfStock`-relative heights: a top Z that differs from the stock top
  would read as a different depth than the user typed.
- A camcraft-mcp skill row for the tool: a process-file change. The tool
  is in the generated reference and the MCP guide. A skill row can come
  with the next skill refresh.

**Cost to change later.** Low. The command is one file. The heights are
ordinary Face fields, so a later change of reference touches only
`face_params`.
