# #4081 slice 2b: the Laser Profile operation in the app

Stacked on slice 1 (PR #4270, the laser machine kind and the laser head)
and slice 2a (PR #4273, `motion::laser`). This slice adds the operation
that the user creates, edits, saves and generates. The laser post is
slice 3.

## Decisions

- **Chosen: one operation, `OpType::LaserProfile`, in its own `Laser`
  section and `Laser` family.** Stored kind id 49 and digest tag 49,
  appended (stored ids never move). `Family::Laser` admits only
  `Kind::Laser`, and a laser cutter admits only that family. The ribbon
  shows the Laser section on a laser cutter only.
- **Chosen: `LaserOp` copies the waterjet's stored shape**, with `power`
  (percent of the source's maximum) in place of the abrasive delay. It
  reuses the waterjet's lead, side and tab types (`JetLead`, `JetSide`,
  `JetTabsOp`) and their project words, because the rules are the same.
  Dropped: a shared "sheet op" struct for the two kinds (it would move
  the reviewed waterjet params and DTO for no change in behaviour).
- **Chosen: the program check shares the waterjet's X/Y gantry law.**
  `machine::waterjet::check_xy_ops` holds the family, tool kind, rotary,
  feed-ceiling and stroke checks once; the waterjet and the laser call
  it with their own family, tool kind and word. Every waterjet sentence
  is unchanged.
- **Chosen: posting and the stock sim refuse laser work by name until
  slice 3.** The verification report says "laser posting is
  unavailable: beam on/off records, power records and pierce dwell
  records are not implemented", the same safety rule as the waterjet
  (#3637). The stock sim says "laser stock removal is not judged: the
  stock sim has no laser model". No built-in post serves a laser.
- **Chosen: a starter laser head in the catalog**, `LZ-KERF-020`
  (`laser_tool(0.2)`), so the empty-rack refusal names a real tool (the
  M13.4 law) and the MCP flow can add one by name.
- **Chosen: an original op glyph** (`assets/ui/svg/op-laser-profile.svg`):
  an outlined head, a thin beam, sparks and the kerf gap in the sheet. It
  differs from the waterjet nozzle and from the solid laser head tool
  glyph.
- **The MCP reach needs no new tool.** `add_operation` and
  `edit_operation` read the kind and its params from the registry and
  the project DTO, as for the waterjet. Only the `add_operation`
  description names the laser kind.
- **Tests join directory suites.** The op tests are in
  `crates/app/tests/machine/laser_profile.rs` (the `machine` suite) and
  `crates/app/tests/mcp/laser_profile.rs` (the `mcp` suite); the ribbon
  and page tests are in `crates/ui/src/egui_app/tests/laser_ribbon.rs`.
