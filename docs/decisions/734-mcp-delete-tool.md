## #734 - MCP delete_tool refuses a tool an operation still uses

**Decided:** `delete_tool` runs the app's existing undoable `DeleteTool` (one undo step). On top of that, the MCP layer refuses a tool that any operation still uses. The refusal names the tool, the first operation using it, and "(and N more)", and it points at `set_tool` to move those operations first.

**Why:** an agent driving CamCraft headlessly can't see the tree. Silently stranding operations that would then refuse to generate is worse than an explicit refusal. `DeleteTool::new` itself refuses only an unknown name or a non-tool. The GUI and existing tests delete in-use tools on purpose (the user sees the stranded ops at once), so changing the command would change GUI behaviour.

**Considered and dropped:**
- Moving the in-use refusal into `DeleteTool` for every caller. That changes the GUI's delete, so it is a separate decision for the maintainer.
- A `force` flag on the MCP tool to delete anyway. Not needed yet; `set_tool` then `delete_tool` covers it.

**Cost to change later:** low. Lifting the check into `DeleteTool` would remove the MCP-side guard and update the GUI tests that delete in-use tools.
