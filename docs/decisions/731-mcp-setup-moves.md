## #731 - MCP set_operation_setup and delete_setup

- **Argument name `operation`** (the issue's wording), though `reorder_operation` uses `name`; `setup` is required and may be null, so an omitted value is refused rather than read as "first position".
- **Result of the move** reports `status` read from the inventory after the command, so the staleness law answers, not the tool.
- **PROCESS_VERSION 49**: 46 upstream, 47/46 held by open PRs, 48 possibly #730; renumbered by merge order anyway.
- Dropped: a combined "move then delete" tool; the two commands stay one History step each.
