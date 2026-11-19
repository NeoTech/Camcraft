## #2625 - Separate WCS feature picks by role

**What was decided:** Show four rows for Origin, X axis, Y axis and Z axis. Each row has its own Pick and Clear controls. The picks remain ephemeral UI state until Apply. Clear below the table discards the draft without changing the setup.

Apply uses the existing `align_roles` result and one undoable `SetWcs` command. The active setup check stays in place. The existing MCP tool keeps the same command. Arming a role clears `wcs_ui.pick_circle` and `wcs_ui.pick_anchor`, because those modes otherwise take viewport clicks before the role draft.

**Why:** Each button names the role that receives the next pick. A new pick replaces only that role. The table keeps the accepted `Face`, `Patch`, `Chain` and `Vertex` kinds. It also keeps the existing rule: two axes define the frame, with an optional origin.

**Considered and dropped:** Keeping the role dropdown would keep the extra selection step. Do not add a second WCS derivation, an origin-only rule, direct world writes, or a new command or MCP tool. The existing alignment law and command already provide the required validation and undo behavior.

**Cost to change later:** Changes to the controls require UI code and tests. Extensions to the alignment rules require separate app work and tests.
