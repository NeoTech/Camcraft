## #1947 - Single-source agent docs for the MCP tools

**What was decided.** The live MCP tool table (`crates/app/src/mcp/tools.rs`)
is the one source. A new module, `mcp::catalog`, makes three outputs from it:

- The Markdown tool reference. `camcraft-cli mcp reference` prints it. It is
  committed at `docs/agents/mcp-tools.md`.
- The compact prompt for a local model. `camcraft-cli mcp prompt` prints it.
- The shared working rules (`catalog::RULES`). The local Prompt-JSON context
  (`provider::prompt_context`) now starts with these rules.

The agent wrappers hold no tool list. `.agents/skills/camcraft-mcp/SKILL.md`
(Codex CLI and Antigravity) and `GEMINI.md` (Gemini CLI) point to the Claude
skill and to the generated reference. Codex also reads `AGENTS.md`, and its
skill table already names `camcraft-mcp`, so `AGENTS.md` does not change.

`crates/app/tests/mcp_doc_drift.rs` pins it:

- The committed reference must equal the generated one. The failure message
  gives the command that writes it again.
- Each tool call (`name {`) and each backticked tool name in a table row of
  the skill, the wrappers and the guide must be a live tool.
- Each live tool must be named in the guide and in the local prompt.
- The local Prompt-JSON context and the local prompt carry the same rules.

**Why.** Before this change, the guide did not name 34 live tools, and the
skill listed shipped tools as gaps. A generated reference cannot drift, and
the tests stop new drift in the hand-written files.

**Dropped.**

- A reference under `.claude/skills/camcraft-mcp/`. This session cannot write
  there without an approval prompt. Also, one neutral path suits all agents.
- Generated wrapper files. They hold only pointers, so the tests that scan
  them are enough.
- A `skill_reference` MCP tool. A new tool changes `tools/list` for all
  clients. The CLI subcommands give the same text without that change.
- A rule that the hand-written skill must name every tool. Many open PRs edit
  that file. The generated reference covers each tool already.

**Held.** The edit of `.claude/skills/camcraft-mcp/SKILL.md` (remove
`set_units` and `set_program` from "Known Remaining Gaps", link the generated
reference) and its pin are posted on the issue as a patch. This session
could not write under `.claude/`.

**Cost to change later.** Low. A tool PR runs one command to write the
reference again. The format of the reference is one function.
