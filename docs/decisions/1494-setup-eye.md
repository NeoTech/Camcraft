## #1494 - Setup eye: view state, not a command

**Decided:** each setup row in the browser tree wears the same eye the
Models, Stock, WCS and operation rows have. A setup other than the active
one is not drawn by default (its operation paths, its work-offset gizmo and
its fixtures); its eye turns that on without activating it. The active
setup is always drawn and its eye does nothing. The state is
`State::shown_setups`, the names of non-active setups turned on.

**Why view state and not a command:** the existing eyes (`hidden_models`,
`hidden_stocks`, `hidden_offsets`, `hidden_paths`) are ephemeral view state
by decision: not undoable, not saved, like the camera ("Ephemeral UI state
... is immediate-mode only", AGENTS.md "Everything Is a Command"). A setup
eye is the same kind of toggle, so it follows them; a command would also put
a display toggle into the undo history and the project file. Hence no MCP
tool: nothing in the document changes. The list resets on New and Open, like
the stock and offset lists.

**Display only:** posting, the program simulation and the collision checks
read the world, never these lists. Hidden setup operations keep their
cached paths and post in the same order (pinned by
`hiding_a_setup_never_changes_what_posts`). A fixture of a hidden setup is
neither drawn nor pickable in the viewport, the same as a hidden model; it
still counts in the collision judgement.

**Considered and dropped:** a saved per-setup flag (a project-file change for
a view preference); hiding the active setup too (not asked for); also
skipping the operation page's selected-op overlays for hidden setups (an
operation is opened by activating its setup first, so the case does not
arise).
