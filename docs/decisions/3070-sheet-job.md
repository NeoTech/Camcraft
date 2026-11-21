## #3070 - Guided sheet job: one checklist over stock, WCS and a post floor

**What was decided.**

- One undoable command, `SetSheetJob`, holds the existing `SetStock` and
  `SetWcsOrigin` commands and writes a new `SetupDef::sheet_job` record.
  It applies them in order and inverts them in reverse. One undo restores
  the stock, the origin and the record.
- The record keeps these values: length (X), width (Y), thickness, Z zero
  (`top` or `spoilboard`), XY zero (four corners or `centre`, default
  `front_left`), the through-cut allowance, and the clamp clearance. All
  values are in mm. The record saves additively (no version step). The
  loader refuses a value that the command would refuse.
- The post gate gets one clause, `sheet_job::program_problems`. For each
  operation of a sheet-job setup, it refuses the post by name when:
  - a move goes below the sheet bottom minus the allowance;
  - a rapid that moves in XY is below the sheet top plus the clamp
    clearance;
  - the stock thickness or the WCS Z no longer match the job (an edit
    after the checklist).
- The stock stays where the setup's stock was: the same XY centre and the
  same bottom face, or the origin if the setup has no stock. The WCS
  rotation becomes identity, because a sheet lies flat on the bed.
- The command acts on the active setup, because `SetStock` acts on the
  active setup. The UI refuses Apply on the page of an inactive setup and
  names the setup to activate.
- The MCP tool `set_sheet_job` requires `allowance` and `z_zero`, so an
  agent always states them. `{"clear": true}` drops only the record.
  `get_job` shows the job. The live gate puts the tool in `LimitEdit`,
  because the allowance is a safety floor.
- The UI is a "Sheet job" group on the Settings page of a milling setup.
  It shows the allowance row every time, with the default 0.3 mm, and one
  summary line ("deepest Z...").
- The allowance is 0..5 mm. The clamp clearance is 0..200 mm. These limits
  catch typing errors.

**Why.** The issue asks for a checklist over the existing undoable
commands, not a second copy of the settings. A post-time floor is the
only place that sees each operation's real depth. Per-operation heights
stay with their operations, and the gate refuses a depth that is not
safe. It does not change the depth silently.

**Considered and dropped.**

- Rewriting the bottom and clearance heights of each operation from the
  checklist. Dropped: this is a hidden edit of many operations. The user
  could not see it, and a later edit of the operation would undo it
  silently. The gate refusal says which height to raise.
- A judgment of every rapid, including vertical plunges, against the
  clamp plane. Dropped: a vertical rapid down to the retract height over
  the part does not cross a clamp. Only XY travel can hit one.
- A job on the implicit (undeclared) setup. Dropped: the record needs a
  `SetupDef`, so the command asks for a setup first.
- A machine-bed limit from the machine file. Dropped: no machine file
  gives a bed Z. The spoilboard is the bed that the user can see.

**Cost to change later.** The record is one optional setup field. More
answers (for example, a spoilboard thickness) are additive keys. A move
from a post refusal to height rewriting would need a new command for each
operation kind.
