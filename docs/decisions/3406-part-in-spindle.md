## #3406 - Part-in-spindle mode and table tool stations on a mill

**Decided.** A mill setup can state part-in-spindle mode: the mill
spindle holds the bar in a collet, and lathe tools stand on the table.
`SetupDef::part_in_spindle` holds the stock's length offset register
(`H`), the collet stick-out and outer diameter, and a list of table
stations. Each station gives one tool its own 1-based work offset, an
approach side (`-X` or `+X`) and a spindle direction (`M3` or `M4`).
`SetPartInSpindle` and `SetTableStation` are undoable commands. The MCP
tools are `set_part_in_spindle` and `set_table_station`, and `get_job`
shows the record. The project key is additive.

- **The family door.** In an active mode, the setup admits the Turn
  family and a Drill with no cross holes. It refuses milling kinds and
  cross holes by name. The check is one early return at the top of
  `family_gate`, and the same check runs again at the post
  (`posting_family`).
- **The setup's work is judged as a lathe's.** Generation and the
  spindle law read `part_in_spindle::work_kind`, which is `Lathe` for an
  active mode. So the lathe's axial-hole door refuses an off-centre hole,
  and the spindle law refuses a bar whose axis is not world Z through the
  work offset. A mill reading would drill an off-centre hole into a part
  that turns.
- **The collet's rating.** `workholding_max_rpm` is required; an unset
  value blocks the post by name. The work-spindle limit of #1031/#1174
  applies to a part-in-spindle mill as to a lathe.
- **The post is refused by name.** No post section writes the stock's
  `G43 H`, a work offset for each tool, or the station's spindle word yet.
  A turning section through a milling post would come out as milling. So
  a part-in-spindle setup with work is refused at the program gate, by
  name. The post section is a follow-up issue.
- **Clearing and stating.** The mode is not stated over a milling
  operation, and not cleared while the setup has any operation. A
  machine that is not a mill refuses the mode; a record on a setup whose
  machine later changes blocks the post by name.
- **Additive key.** An older reader drops the key. Its family door then
  refuses the turning work on the mill at the post, so the file does not
  post wrongly. No version step.

**Dropped.**
- A version step for the key: the older reader already refuses by name.
- Stacking on #4288 (the laser PR also edits `family_gate`): the hook
  is four lines at the top of the function, far from the laser hunks,
  and #4288 is a conflicted draft feature. Whichever merges second
  refreshes.
- A tool-kind filter on table stations: a drill on the table is a legal
  station for an on-centre hole, so any tool is accepted.
- UI rows on the Setups page: the issue asks for commands and MCP tools.
  A UI follow-up is filed.

**Cost to change.** The record is one additive key and one schema field.
The post section adds code only in the posting path and removes the
program-gate refusal.
