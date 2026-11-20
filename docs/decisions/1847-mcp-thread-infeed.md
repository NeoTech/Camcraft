## #1847 - Thread infeed tools for MCP

**What was decided:** Add `get_thread_infeed` and `set_thread_infeed` over the existing `EditOperationParams` command. Return every plan field, including defaults. Accept partial updates. Refuse unknown fields. Use the project DTO to decode values. Let the existing command validate the machining parameters.

**Why:** Project JSON omits the default plan and default flags. The generic editor checks whether a patch survives project serialization. That check refuses some resets to default values. Explicit read values let an MCP caller inspect and restore the complete plan. The command preserves validation and undo behavior.

**Considered and dropped:** A broad change to the generic editor would affect unrelated operation types. Direct ECS writes would bypass command validation and undo. A new command or dependency would duplicate existing support. Separate machining validators would repeat domain rules.

**Cost to change later:** A new plan field requires an update to the tool schema, explicit JSON view and tests. A shared explicit representation could replace this view if more callers need it.
