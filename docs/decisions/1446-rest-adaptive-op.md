## #1446 - Rest machining on the 2D Adaptive operation

Stacked on #2822 (#2033), which reads the stock of the previous operations
for the 2D Pocket.

**What was decided**

- The 2D Adaptive operation has the rest group (`OpParams::Adaptive2D.rest`).
  Off is the default. A file without the key reads as off.
- The group reads the previous operations only. The stock that they leave
  (`rest_stock_from_columns`, #2033) goes to the rest Adaptive planner
  (`rest_adaptive`, #2031) through the new
  `machining2d::rest_adaptive_previous`.
- The entered previous tool is refused by name on a 2D Adaptive, at the
  edit and at generation. The 2D Pocket keeps that mode.
- No stock ahead is refused by name. The 2D Pocket cuts the whole pocket
  in that case. The 2D Adaptive does not: the rest planner has no
  proved-clear space to enter from, and a full pocket is the work of the
  plain 2D Adaptive.
- A roughing pass that stops above the floor is refused by name ("no
  entry with proved clearance"). The whole pocket is not guessed.
- Beside the group, these are refused by name and not dropped: the
  Trochoidal method, arc turns, corner relief, a tilt.
- Equal steps (`Approach::equal_steps`, #2012 / #2842) run inside
  `adaptive::rest_levels` (`ladder_stepdown`), so both `rest_adaptive`
  and `rest_adaptive_previous` step and read the stock at the same
  equal-step depths.
- Corner relief (`#2400`) beside rest machining on `OpParams::Adaptive2D`
  is refused by name (`REST_ADAPTIVE_CORNER_RELIEF`) at the edit and at
  generation (and skipped in `corner_relief_sim`), because
  `rest_adaptive` plans no inside-corner reliefs. The 2D Pocket keeps
  corner relief in both rest modes (#2822).
- The film of the stock is `cell * (1 + sqrt 2)`. The column reading
  leaves a band of that thickness along each wall that an earlier tool
  cut to. The planner plans no pass for the band. The band still stands
  for each plunge and each load reading.
- The allowance grows only material thicker than the film. It never makes
  the standing material smaller.
- The stamp writes byte `0xD9` and the allowance when the group is on. An
  operation without the group keeps its digest.

**Why**

- #2031 decided that an entered diameter is no proof that material is
  absent. The rest Adaptive planner plunges into cleared space, so its
  input must be proved.
- With the film at one cell diagonal (the value of #2033), the wall band
  read as rest material along each wall. The planner then refused the
  60 x 40 fixture ("would reach 23 passes by a retract"). Measured on a
  0.1 mm grid: the band is 0.14 mm thick, and an opening by 0.12 mm
  leaves the four corners only.
- An allowance of 0.6 mm made the band 0.74 mm thick and the planner
  found no entry. Thus the allowance grows the body of the material only.

**Considered and dropped**

- A stock built from the entered previous tool. Dropped: #2031 forbids it.
- The whole pocket when no earlier operation cut. Dropped: the rest
  planner cannot enter a full pocket, and a silent change to the plain
  planner would hide a missing roughing operation.
- The removal of the wall band from the stock. Dropped: material thinner
  than a cell beside a wall is not proved absent, so it must stand for
  the safety readings.
- The rows of the group on the 2D Adaptive page. Not in this PR: the page
  keeps the value of the draft, and the rows are a follow-up issue (as
  #1815 was for the 2D Pocket).

**Limits**

- Real residue thinner than the film (2.4 cells) along a wall is not cut.
  The path note states the film.
- The `entry` field and the links of the operation are not read by the
  rest planner: each entry is a plunge into proved-clear space.
- The MCP skill row is owed. The session could not write
  `.claude/skills/camcraft-mcp/SKILL.md`.

**Cost to change later**

- A second source for the 2D Adaptive needs a new producer of `RestStock`
  and the removal of one refusal. The file key `rest.source` exists.
- A finer stock grid makes the film smaller with no code change.
