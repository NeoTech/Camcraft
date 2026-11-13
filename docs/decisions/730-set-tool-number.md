## #730 - MCP: set_tool_number

**Decided.** One tool, `set_tool_number { tool, number | null }`, over `SetToolNumber` through `env.history` (one undo step); `number` is required so a missing field is refused rather than read as a clear. `get_job`'s tool rows gain `number` (own, or null) and `posts_as`. The tool lives in a new `mcp/tool_number_tools.rs` (tools.rs and job.rs stay under the 400-line law).

**Why.** Reuses the command's refusals verbatim (unknown/non-tool, range, duplicate, no-op); a JSON number that is negative or fractional is refused by the tool with "positive integer".

**Dropped.** A separate read tool (get_job already lists tools); a script verb (not asked).

**Cost to change.** Low: additive JSON fields and one tool row.
