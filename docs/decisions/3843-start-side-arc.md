## #3843 - 2D Offset: a start-side edit of the seed Helix takes Arc

**Decided.** The seed of a new 2D Offset does not change: Auto, Helix.
The edit door changes instead. When one `EditOperationParams` moves a 2D
Offset from "Pass starts at" Auto to Inside or Outside, and the entry is
Helix before and after that edit, the same edit sets Arc
(`OpParams::with_start_side_entry`, next to the tilt basis step). The GUI
page, MCP `edit_operation` and the cutting-data and stock edits all pass
through that one constructor. One undo restores Auto and Helix together.

**Why.** The maintainer's words on #3741: "an outside cut would default to
a ARC or straight approach in the plane of its cut". A user who sets a
start side and does not touch the entry gets Arc, so #3748's
`EntryNotRecommended` warning does not show for an operation that the
user did not change. The law sits in the command, so every door gets it
and undo stays one step.

**Kept as the user set it.**
- An entry that the same edit changes (for example Plunge to Helix).
- A Plunge or Ramp set before the start side moves.
- A Helix on an Inside pass that moves to Outside (the user kept it).
- A Helix that the user selects again on an Inside or Outside pass.
- The entry when the start side goes back to Auto.

**Dropped.**
- Seed Arc for every new 2D Offset. The seed is also the default of every
  Auto pocket area, and Arc is opt-in there. It moves about 17 test files
  of the app and UI crates that make a 2D Offset with no entry, and it
  changes the Auto toolpath of every new operation.
- A UI-only change on the Lead in page. MCP and the other edit doors would
  not get it.
- A stored "entry is the seed" flag. It changes the project file for one
  default.

**Cost to change later.** Small: one function and its call in
`EditOperationParams::edit`. A seed change can replace it at any time.
