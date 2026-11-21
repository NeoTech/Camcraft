## #2012 - Depth ladder: the remainder ladder stays the default, equal steps is an opt-in parameter

**What was decided**

1. The default ladder does not change. It steps down by the full stepdown
   from Top Z, and the last step keeps the remainder. `top 20, bottom 2,
   stepdown 5` cuts `[15, 10, 5, 2]`.
2. Equal steps is an operation parameter, `equal_steps`, off by default. It
   is a field of the operation's `links.approach` object
   (`camcraft_motion::approach::Approach`). On, the ladder cuts the same
   number of levels, and each step is the span divided by that count. The
   same example cuts `[15.5, 11, 6.5, 2]`.
3. The mode is one function, `machining2d::equal_stepdown(top, bottom,
   stepdown)`. It gives the stepdown that the unchanged level law
   (`cut_levels_under`) then walks. `cut_levels_for(top, &approach, bottom,
   stepdown)` reads the mode and the stock top from the approach. The
   generators call it in place of `cut_levels_under`.
4. With stock over Top Z (#2004) the level at Top Z stays, and the steps
   under it are equal.
5. The stepdown stays the largest step. Code that reads `stepdown` as a
   bound (the 3-D shell's floor under a plane, the contour's first entry
   height) is not changed: with equal steps it reads a bound that is a
   little larger than the real step, which is the safe side.
6. The parameter is on the operations that have a `links` object: 2D
   Contour, 2D Pocket, 2D Adaptive, 2D Offset, Circular, Slot, Bore, 3D
   Pocket and the rotary / multi-axis roughing that run the 3-D ladder.
7. The app's pocket region law (`operation::area::ladder_planes`) reads the
   same mode through `OpParams::ladder_stepdown`, so the region is built at
   the planes that the generator cuts.

**Why**

- The default must not move posted G-code. Each golden file and each saved
  project keeps its programmed depths.
- The project rule "CAM choices are parameters" asks for a parameter, not
  for a silent change of the law.
- The `links.approach` object is already saved field by field (absent means
  default), already rides `EditOperationParams` for undo, and is already
  what the MCP `edit` tools parse. A new field there needs no file version
  step and no new command.
- One function for the mode keeps one level law. No second ladder can drift
  from the first.

**Considered and dropped**

- Equal steps as the new default. Dropped: it changes the posted depths of
  each multi-level operation in each saved project.
- A second level function with its own loop (levels computed as `top - i *
  span / n`). Dropped: two loops with two float behaviours for one law. The
  mode as an adjusted stepdown reuses the tested loop and its refusals.
- A field on each parameter struct of the motion crate. Dropped: Face,
  Trace and Trochoidal Slot have about 30 struct literals in tests and
  callers, and the operations have no `links` object to save the choice in.
- Replacing `stepdown` in each generator's parameters before the run, so
  that each reader sees the equal step. Dropped: Bore and the Circular
  vault have one span for each hole or ring, so only the level law knows
  the span.
- A row beside each Stepdown field on the Passes tabs. Dropped for now: the
  field is saved in `links.approach`, and the one editor of that object is
  the Lead in / Lead out tab. The row is in its Linking group.

**Not done here (follow-up #2832)**

- Face, Trace, Engrave and Trochoidal Slot have no `links` object. They keep
  the remainder ladder. Issue #2832 adds the parameter to them.
- Turning has its own infeed laws (`turning/*`). They are not part of this
  decision.

**Cost to change later**

- To make equal steps the default: change `Approach::DEFAULT.equal_steps`,
  write the field when it is `false`, and regenerate each golden of a
  multi-level operation. Old files without the key would then read as equal
  steps, so that change also needs a file version step.
- To remove the parameter: delete the field and its DTO key. Files that
  have the key still load (unknown keys of `approach` are ignored only if
  the DTO stays tolerant; check before the removal).
