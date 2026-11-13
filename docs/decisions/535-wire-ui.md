## #535 - Wire EDM machine setup and op pages in the UI

**Decided:**

- **The machine page is the machine editor window** (draft + Apply = `EditMachineApply`, one undo step through `LoadMachine`/`SetSetupMachine`). For a `wire_edm` kind it shows a Wire EDM section in place of Spindle / Tooling / Coolants: the `edm` keys (guide planes, max taper, AWT as not-stated/yes/no, wire diameters), the `edm.codes` rows (the five names #533's post asks for, each "not stated" until written, plus any other name already on the machine) and the condition table (id, name, gap, wire diameter, role). Every stated/unstated value is a checkbox + "not stated", as the rest of the editor. An untouched absent `edm` section is not made present by looking at it.
- **Refusal by name at the machine door.** `machine::edm_refusal`, chained into `tooling_refusal` (so LoadMachine, SetSetupMachine and the project reader all ask it): a `condition` code without `{id}`, a blank or bracketed code, a condition with no id, a duplicate id. The same `{id}` law the wire post refuses by (#533), now asked when the edit is made, not at post time.
- **No new command or MCP tool.** The edits ride the existing machine-edit command (`EditMachineApply` -> `LoadMachine`/`SetSetupMachine`), which the machine MCP tools already reach; nothing new is added, so no `MCP:` follow-up.
- **The op page keeps #532's page** (side, start hole, planes, condition, one-step Apply through `EditOperationParams`); this issue removes what did not belong to a wire cut: the Tool row (it silently adopted the rack's first tool into the draft) and the Coolant row. It stays additive for #1109's kerf side / edge clearance fields and #534's.
- **Quick-add names the first condition.** The ribbon cell of a tool-less kind no longer waits on an empty rack, and a quick-added Wire Contour takes the machine's first stated condition; with none stated the door's existing refusal names the table to fill.
- **The editor's fields scroll** with Apply / Revert kept in reach: the wire section makes the window taller than a short screen.

**Considered and dropped:** a dedicated "Wire EDM" machine page apart from the editor (a second place to edit the same draft); defaulting any code (D5); a viewport pick for the start hole (still typed; not built); sinker keys (#541's page).

**Not exercised:** a real wire control; the PNG snapshots (release lane, never changed here).
