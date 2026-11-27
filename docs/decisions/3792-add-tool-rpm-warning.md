## #3792 - A tool added outside the machine's spindle range says so at the add

**What was decided.**

- `AddTool::with_holder` reads the mounted machine. If the tool's posted
  speed is above `spindle_max` or below `spindle_min`, the command carries a
  warning (`AddTool::warning`).
- MCP `add_tool` returns it in a new reply key, `warning` (null inside the
  range). The UI's add shows it in the status line, after the holder note.
- The text is: `<tool> runs <rpm> rpm, above the <machine model>'s <limit> -
  the post will be held; set the tool's speed (edit_tool) or
  apply_cutting_data`.
- The machine check's problem line for an operation ends with the same fix:
  `... above the machine's 10000 - set the tool's speed (edit_tool) or
  apply_cutting_data`. The default-machine check (#2321) uses the same words.
- The fix text is one constant, `camcraft_post::machine::SPINDLE_FIX`. The
  warning text is built by `spindle_range_warning` in the post crate, next
  to the check. It works on domain values only, so it is not in `app`.
- The add does not change the tool's speed. The machine check and the post
  gate do not change. No posted G-code changes.

**Why.** The other paths that set a speed (`apply_cutting_data`, the Feeds
tab spindle row) already say when the machine holds it. Only the add was
silent, so the first word came at verify, after the work was generated. A
library speed is what the user stated; a silent clamp at the add would
change posted `S` words, which is under the safety floor and outside this
issue.

**Considered and dropped.**

- Clamp the speed at the add. Dropped: it changes posted G-code
  (`effort:high`), and the issue asks for a warning.
- Put the warning in the existing `note` key. Dropped: `note` is the holder
  line; an agent that reads `warning` must not have to parse a holder
  sentence.
- Add the fix one time to the joined gate message, not to each problem
  line. Dropped: callers that read `MachineCheck::problems` one by one
  (verify findings, the UI) would not see it.
- Name the fix on an NC-line finding (`SourceLocation::NcLine`). Dropped:
  a posted NC file has no document tool to edit, so that line keeps its
  old text.
- A separate `warning` for the tool-library import and for `edit_tool`.
  Dropped: not in the issue. `edit_tool` and a machine change stay judged
  by the machine check.

**Cost to change later.** Low. The words are in one constant and one
function. The tests that pin them are
`crates/app/tests/mcp/add_tool_spindle_warning.rs`,
`crates/post/tests/machine_check.rs`,
`crates/app/tests/default_machine_limits.rs` and
`crates/ui/tests/machine.rs`.
