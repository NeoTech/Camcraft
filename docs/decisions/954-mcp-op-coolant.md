# 954 - MCP: set an operation's coolant

- **Tool** `set_operation_coolant { name, coolant }` runs the existing
  `EditOperationParams::with_coolant` (one undo step), in a new
  `mcp/coolant_tools.rs` (mod.rs and tools.rs already near the size cap).
  Considered: extending `edit_operation`'s patch - dropped, it would bypass
  the dedicated command and its stamp-survival law.
- **Words**: the schema enum is the project file's kebab-case
  (`through-tool`, as the issue says); `through_tool`, spaces and any case
  are accepted on input. Anything else is refused naming the word and the list.
- **No machine entry**: the mode is still set and the reply carries a
  `warning` (the post is what refuses at post time; no code is invented).
  Warns only when the machine states coded coolants and none serves the
  mode; Off and an unstated list never warn. Considered refusing at set
  time - dropped, the GUI picker allows it too and the machine may change later.
- `get_job` operations now show `coolant` so a headless agent can read the
  value back (also what the tests assert).
- The camcraft-mcp skill is a process file; its tool table gets the row in a
  separate process PR (noted in the PR body).
