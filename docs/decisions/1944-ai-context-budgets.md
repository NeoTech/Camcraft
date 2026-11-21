## #1944 - AI context budgets and compact MCP read modes

**Decided.**

- `get_job` takes `summary: true`. The summary removes `machine.definition`
  and keeps the machine `path` and `name`, with `definition_omitted: true`.
  Each operation's `params` keeps only the fields that differ from the
  default that `add_operation` seeds for its kind. Null and empty-list
  fields of operation, setup, tool and model rows are removed. Inside
  `params`, a `null` stays, because there it is a stated value.
- `list_operation_types` takes `kind` (a key or a label, any case) and
  `section` (`Mill2D`, `Mill3D`, `Drilling`, `MultiAxis`, `Turning`,
  `Plasma`, `WireEdm`, `Sinker`). A name that matches no kind or section
  is refused with the valid names.
- `list_features` takes `compact: true`. Compact rows keep the id, the
  name and the facts that a pick is chosen by. Counts replace the loop and
  face lists. A face whose outside is unknown still says so. The compact
  default `limit` is 25 rows for each kind. The full listing keeps 500, so
  that no current caller changes.
- `mcp::provider::tools_in(provider, ToolSet)` gives the full table or a
  core set of 25 workflow tools (`CORE_TOOLS`). `tools(provider)` stays the
  full table.
- The chat worker (`crates/ui/src/egui_app/ai_chat/budget.rs`):
  - Cuts each tool reply that goes back to the model at 12000 JSON
    characters. The cut keeps whole characters and ends with a hint that
    names the compact read modes. The tool card in the panel keeps the
    full text.
  - Sends at most one viewport image in each request (the newest).
  - Stops a turn after 12 model rounds. This cap existed as
    `MAX_TURN_STEPS`; it moved to `budget::MAX_TOOL_ROUNDS` and has a test.
  - Sends at most 6 earlier turns. When it drops older turns, it puts the
    latest successful `get_job` reply of all earlier turns (cut to the
    reply cap) in front of the first kept prompt.
  - Offers the core tool set to a local endpoint and the full table to a
    cloud endpoint.

**Why.** Small local models have context windows of 8k to 32k tokens. The
full tool table and one full `get_job` or `list_features` reply can fill
that alone. Each cap is a constant with a test, so a later change is one
line.

**Dropped.**

- A smaller default `limit` for every `list_features` call. It would change
  the reply of each current MCP client. The small default applies only with
  `compact`.
- A panel control to choose the tool set. The endpoint kind already tells
  local from cloud, and a new control needs layout and snapshot work. A
  follow-up can add it if a user wants the full table on a local model.
- Removing old tool results inside one turn. Provider formats pair each
  tool call with its result; to remove one safely needs the call record
  in each message. The reply cap and the round cap bound one turn.
- A tool refusal for a model that calls a tool outside the core set. The
  call goes through the same MCP door and approvals as any other call.

**Cost to change.** Each limit is one constant in `budget.rs` or
`compact.rs`. The core set is one list in `mcp/provider.rs`.

**PR #3033 review fix.** Keep the upstream importer warning for edges that
lie off their surfaces. Keep the compact `get_job` schema from this branch.
These are separate tool entries. Taking either complete conflict side would
remove the other change, so neither side is discarded. This resolution adds
no behavior. The screenshot check uses a temporary harness that is not
committed. Its images stay on the fork's `pr-screenshots` branch. The full
chat tool card remains visible while the model reply is capped.
