## #1521 - Plasma cut data on the torch: slice 1 (the recipe record)

**What was decided.** A plasma torch's cutting row can carry a recipe
(`CuttingRow::plasma`, type `tools::plasma_cut::PlasmaCutData`). The recipe
is keyed by the row's material and the recipe's plate thickness. It states
the kerf and, each optional, the cut height, pierce height, pierce delay,
transfer height, puddle jump (percent of pierce height and dwell), amperage,
arc voltage, the pierce capacity (`max_pierce_thickness`) and an edge-start
mark. The record follows the shape that #1881 gave the waterjet recipe: one
optional field on the row, one file key (`plasma`) on the row's DTO, and
the existing `AddTool` / `ChangeTool` / `ImportTools` commands carry it.

This PR is slice 1 of the five slices that the issue lists, and only the
tool-side part of it. The PR says `Refs #1521`, not `Fixes #1521`. The
issue stays open for the other slices.

**Why.**

- The issue is an umbrella (`effort:max`). The effort check comment on the
  issue recommends one piece of work for each slice. Slice 1 is the base
  that the other slices read.
- A recipe is never picked by its material alone. `row_for`,
  `row_in_tool` and the `Unknown` row logic ignore recipe rows
  (`CuttingRow::is_recipe`). The feed of a 3 mm recipe must not seed an
  operation on 12 mm plate. Selection is `plasma_row_for(material,
  thickness)`, with an exact thickness and no nearest-thickness
  substitution.
- A blank value means "the controller's own table", the same meaning as a
  blank on the plasma operation. There is no default for a height or a
  time.
- A recipe whose plate is thicker than its stated pierce capacity is
  refused, unless the recipe states an edge start. The refusal is at the
  tool door (`refusal_for_kind`), so the command, the project file and the
  tool library all refuse it by name.
- Zero is legal for a height and a time, as on the plasma operation
  (`PlasmaParams::validate`). Thickness, kerf, amperage, arc voltage, the
  capacity and the puddle-jump percent must be above zero.
- A puddle jump without a pierce height is refused: it is a percentage of
  that height.
- The tool editor does not draw a torch recipe in the speed-and-feed
  table, and a plain row for the same material can stand beside it. The
  recipe stays in the draft and survives Apply.

**Considered and dropped.**

- The full "Done means" of the issue in one PR. It is four more slices
  across motion, post, sim and two new machine kinds.
- An editor for the recipes on the tool page. PR #2663 changes that page's
  layout at this time. Until the editor exists, a recipe enters through the
  project file, the tool library and the MCP tool definition
  (`cutting_data[].plasma`).
- "The operation picks a row" and the override commands. They change what
  the operation generates and posts, so they belong with slice 2 and its
  goldens.
- Gas type and gas pressure. A gas name is text and `CuttingRow` is
  `Copy`; the field needs its own design (a fixed gas list, or a row that
  is not `Copy`).
- Multi-stage pierce (0 to 4 stages). Plasma uses one stage. The stage
  list arrives with the laser slice.
- Order rules between the heights (transfer below pierce, cut below
  pierce). Maker charts differ, and the issue does not state such a rule.
- A project file version step. No build reads a recipe for generation
  yet, and three open PRs already move `PROJECT_VERSION`. #1881 added the
  `waterjet` key in the same way. An older build drops the `plasma` key
  and reads the row as a plain material row of the torch.

**Cost to change later.** Slice 2 must take the project version step when
a recipe first changes generated or posted output: from then on an old
reader must refuse the file. A new recipe field is one optional key. A
move to a stage list replaces `pierce_height` / `pierce_delay` and needs a
read migration for files written by this slice.
