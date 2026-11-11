## #511 - Detect pockets / slots in the GUI

**Decided:** a "Pockets and slots" section on the Model page. It has a **Detect pockets / slots** button, one row per recognised feature (kind, slot width/length/ends, floor Z, depth, tightest corner, islands) and a **Create Pocket / Create Slot** button per row. Create queues one `CreateOperation`, selects the new operation and opens its page; one Undo removes it.

- **Shared plan:** the name, the seed, the tool-sized helix and the feature mapping moved into `app::recognize::plan_operation`, with the seed in `operation::seed_params`. The MCP `detect_pockets {create}` tool calls the same code, so a click and an agent create the same operation. `ui` holds no recognition logic.
- **Tool:** the last document tool that fits the kind, the rule the ribbon's quick-add uses. The row's tooltip names it, and Create is disabled with no tool.
- **Stale list:** the scan is view state keyed by model and document version. Any document change hides the list and Create refuses ("detect pockets again"), so no row ever offers coordinates the part no longer has.

**Considered and dropped:**
- Hover-highlighting each floor in the viewport. It would need new viewport overlay plumbing, so it is left for a follow-up if wanted.
- A model context-menu entry. The page keeps the list visible next to the model's other actions (placement, repair).
- A tool picker per row. The operation's page already changes the tool after creation, one undo step apart.

**Cost to change later:** low. The page is a single module, and the state is a single file.
