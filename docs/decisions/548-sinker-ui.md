## #548 - Sinker EDM: machine setup and op pages in the UI

**Decided:**
- **The machine page is a section of the existing Machine window**, not a new window: `machine_edm_editor::draw`, shown only when the draft's kind is `sinker_edm`. It edits a COPY of the draft's `edm` and writes it back only when an edit changed it, so opening the window never turns an absent `edm` into an empty one. Sinker keys (max orbit radius, electrode pickup), the orbit list (CamCraft geometry + the control's word), the condition table (id, name, gap, undersize, role) and the seven `edm.codes` roles (#542) are all stated/"not stated" widgets: unticked = absent, never zero. Roles in `edm.codes` the sinker post does not read are left as written and listed read-only.
- **No new command.** Edits ride the existing undoable `EditMachineApply` (-> `LoadMachine` / `SetSetupMachine`), so one Apply is one undo step and the mounted numbers return exactly. Validation is the loader's own law at command creation (`Machine::is_sane` -> `Edm::is_sane`); the page words each cause beside the section ("Apply will refuse: ...") so the refusal is never the generic "states a limit that cannot be meant".
- **`Edm::is_sane` gained one rule**: a blank `pickup` is refused. The file loader already refused it (`parse_edm`), the in-memory door did not, so a GUI draft could have mounted a machine that would not reload. Now the two agree.
- **Op page**: the Sinker Burn page (#541/#543 fields already in the tab pattern) now reads what the mounted machine states for the typed condition - "Condition gap" and "Condition undersize" (main's Side Burn already names its own "Electrode undersize" on Passes, and a label stands on one tab only), or "not stated" and a hint that the op will refuse - so the generator's inputs are visible before Generate.
- **Seeds**: ticking a value on seeds one the field's law accepts (radius 1.0, gap / undersize 0.0, blank pickup / code as a flagged problem) - never a vendor value.
- **Seam left for #544 / #545 / #547**: orbit and side-burn op pages are out of scope (those ops do not exist). The Sinker Burn page's tabs and the Machine section are where they join; the electrode-from-faces button (#547, `ElectrodeFromFaces`) belongs at the top of the Sinker Burn Geometry tab once that PR lands.
- **MCP**: no new command, so no new tool. `EditMachineApply` itself has no MCP verb (an agent mounts a machine file with `load_machine`); filed as an `MCP:` follow-up.

**Why:** #290 D4/D5 (conditions and codes are the user's data, "not stated" never zero); AGENTS.md "Everything is a command" (the history is the single source of undo) and "Validate early".

**Considered and dropped:**
- A separate EDM machine window. The kind picker already lives in the Machine window and a second window would fork the draft/Apply law.
- A new `EditMachineEdm` command with finer-grained undo. The draft law (one Apply = one step, #373 style) is the existing convention for every machine edit.
- Per-field validation messages from `post`. `Edm::is_sane` stays a boolean for the loader; the UI states the same laws in words, pinned against `is_sane` in a test.

- **Refreshed on main after #1141's reworked merge (2026-11-13)**: the section edits `Machine::edm_codes` (main moved the codes off `Edm`), and in main's scrolling Machine window a sinker draws its own section plus axes in place of spindle, tooling and coolant - as main's wire EDM does, since `Machine::edm_sane` refuses those on an EDM. Dropped: keeping the spindle/tooling/coolant rows above the sinker section (rows the machine law refuses, and the section fell below the fold).

**Cost to change later:** low. The section is one file over `Edm`; new keys (#544 orbit limits, #546 wear) add rows.

Not exercised on a real EDM control; pages need the maintainer's hand check.
