## #3776 - Keep machine assignment visible

**Decided:** Put "Use for this setup" and its "Cannot assign" reason below
the machine list, outside the details and editor scroll areas. Read the
selected entry after this frame's selection and filter changes. Show
"Selected: <name> - not yet this setup's machine" when its source mount path
differs from the active machine. A double-click checks the clicked entry's
`assign_block` and queues the existing undoable `UseCatalogueMachine` command.

**Why:** The action remains discoverable when details are scrolled to the
bottom. A single click still browses without changing the setup. Both ways
to assign preserve the existing validation and undo behavior. Keeping the
action below the list places it beside the selection message and avoids
using a previous selection's refusal state.

**Considered and dropped:** Assigning on a single click would remove safe
browsing. A second assignment implementation would duplicate validation and
undo routing. Moving the action to the earlier editor toolbar would separate
it from the selection message and require extra care with same-frame picks.

**Cost to change later:** Another layout position is a small UI move. An
intentional gesture change also requires updating the structural pins.
