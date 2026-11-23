## #3475 - A tool wider than its slot is refused at the command

**What was decided.**

- Slot: the generator's fit rule (`h = width / 2 - r - leave`, with
  `h = 0` permitted) moved into one pure function,
  `camcraft_motion::machining2d::slot_centre_band`. The slot generator
  and the commands call it. There is no second copy of the rule.
- A new `slot_gate` in `commands/operation/gates.rs` runs in
  `CreateOperation`, `EditOperationLinks` (MCP `set_tool`, the UI's links
  edit, only when the tool changes) and `ChangeOperationKind`. It refuses
  a plain Slot whose tool is too wide. The text is one sentence that names
  both sizes and the largest tool that fits, in the document unit:
  "the tool 'EM-WIDE' is 10 mm wide and the slot is 6 mm wide: choose a
  tool of 6 mm or less" (after the usual "invalid parameters: " prefix).
  The UI shows the same text after its "links not updated: " prefix.
- The default width (review of PR #3635, maintainer's standing order to
  take the recommended option). A Slot whose width comes from the seed
  (4 mm) takes the larger of 4 mm and its tool's diameter:
  `operation::seed_slot_for_tool` (`operation/slot_pick.rs`), called
  beside `seed_helix_for_tool` at the ribbon's quick-add
  (`ui/src/actions.rs`) and at MCP `add_operation` before its params
  patch (`mcp/edit.rs`), and in `ChangeOperationKind` after
  `retype_params`. A tool of 4 mm or less gives exactly the old operation.
  A width that the caller or the picks state is not changed, and the gate
  judges it.
- Picks that seed no slot (`slot_from_picks` is `None`): the width is only
  the seed's fallback, so the gate skips it
  (`operation::slot_width_is_stated`) and generation still refuses with the
  pick text. The `ChangeTool` check skips such operations too. The
  alternative (seed the fallback width from the tool) was dropped: it
  writes a width that nobody stated.
- The second door: `ChangeTool` refuses an edit that makes a tool wider
  when a plain Slot that uses the tool can then not hold it
  ("S1 uses it: the tool ..."). It reads the operations that name the tool,
  as `retype_refusal` does.
- The generator keeps its own refusal. A project file from an older build
  can still hold the bad pair.
- `slot_dxf_sketch::picks_that_are_no_slot_are_refused_by_name`: the plain
  Slot on the 3 mm outline under the 4 mm tool was accepted at create and
  refused at generate. Now the create refuses it. The trochoidal slot is
  not changed.

**Why.** AGENTS.md "Validate early". The slot rule is a function of three
numbers that the command has, so the check is exact. It cannot refuse a
slot that the generator cuts.

**Dropped.**

- A check at `set_tool` only (not at creation): an AI caller could then
  make the bad pair with `add_operation`.

- 2D Pocket. Its fit depends on the region: the containment (an open
  side lets a wider tool cut from outside), the stock and the part-sliced
  region. The command does not have that region without the part slice
  and the area controls. A bounding-box or width estimate can refuse legal
  work, so none was added. A warning was not added, because it would need
  that same region to be exact. A `source:agent` follow-up issue holds the
  pocket work.
- A refusal in `EditOperationParams` when the slot width is edited below
  the tool. It was not in this issue's scope (the tool assignment); it
  goes in the same follow-up issue.
- Trochoidal Slot: its fit has a different rule (the trochoid needs a tool
  smaller than the width) and its generator already names it.

**Cost to change later.** Small: `slot_gate` and `resize_refusal` are one
call each; a pocket check can join `tool_gate` the same way.
