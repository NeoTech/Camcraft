## #3195 - Hole patterns from numbers (`add_hole_pattern`)

**Decided.**

- The recipes live in `camcraft-geometry` as `hole_pattern::HolePattern`
  (point list, line, grid, bolt-hole circle full or partial) with one
  method, `points()`. It knows nothing of operations. It refuses bad
  numbers by the field name and caps a pattern at 10,000 holes.
- A partial circle takes `angular_pitch` (degrees from hole to hole) or
  `span` (degrees from the first hole to the last hole). A pitch that
  closes the circle on itself, or a span of 360 degrees, is refused: the
  full circle says that.
- The MCP tool `add_hole_pattern` makes the Drill, Bore or Thread
  operation on the stock (`SrcRef::Stock`) with `CreateOperation`. It
  puts the holes in the `holes` field. Each hole starts at `top`. The
  default `top` is the top of the stock of the active setup. With no
  stock and no `top`, the tool refuses by name.
- Heights in the seed that measure from the model (`top_of_model`,
  `bottom_of_model`) measure from the stock (`top_of_stock`,
  `bottom_of_stock`) for a pattern operation. A stock-only setup has no
  model top, and generate refused "Top of Part Z ... no model loaded".
  A height that the `params` patch states still wins.
- The optional spot drill is a Drill operation with `cycle: spot` on
  the same holes, at its own stated depth. There is no default spot
  depth: the depth depends on the spot tool and the chamfer that the
  shop wants.
- The spot drill and the operation are one undo step
  (`History::begin_group` / `commit_group`, a `BatchCommand`). Both
  commands are built (and so checked) before either runs, so a refusal
  changes nothing. A `CreateOperation` takes its program position when it
  is built, so the main operation is built again after the spot drill
  runs. Otherwise the two operations share one position and fall back to
  name order.
- A hole off the stock footprint is made, with a `warning` that names
  it. A pattern can be meant for a fixture or a later stock.

**Dropped.**

- The Stock/Sketch UI form. It is a UI page of its own and goes to a
  follow-up issue. The MCP tool calls the same commands that the form
  will call.
- A new undoable command type (`CreateHolePattern`). The existing
  `CreateOperation` already validates holes, tool and cycle. A group of
  two existing commands gives the one undo step with less new code.
- A change in the height resolver that reads "top of model" as the stock
  top when no model is loaded. That changes every operation kind, not
  only patterns.
- Storing the pattern recipe on the operation. The operation stores the
  holes, as a hand-entered or detected hole list does. An edit of the
  pattern makes a new operation. A stored recipe needs a file-format
  change.

**Cost to change later.** Low. The tool is additive. A stored recipe or
a UI form can call `HolePattern::points()` without a change to it.
