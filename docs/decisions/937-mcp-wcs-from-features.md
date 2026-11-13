## #937 - MCP: set the WCS from feature picks

**Decided.** A new MCP tool, `set_wcs_from_features { picks: [{model, id,
role}], setup? }`, in its own module `crates/app/src/mcp/wcs_tools.rs`.
It parses the `list_features` ids, runs the GUI's frame law
`pick::align_roles` (#701) unchanged, turns the result into a WCS
placement exactly as the GUI's `CommandKind::SetWcs` does (position as
is, rotation `quat_from_euler` of the law's roll/pitch/yaw in radians),
and executes the existing undoable `SetWcsOrigin` - no new command kind,
one History step. Every refusal is the law's or the command's text.

**Stacked on #706.** The law lives in the open PR #706 (not yet on
`main`), so this branch is cut from #706's head `d2663a3f` and its PR
says "stacked on #706". It merges after #706.

**A fix to the law rides here.** Building the pin showed that
`align_roles` put a Z pick in the Y slot when the frame came from a Z
pick then a Y pick (the `(Z, Y)` arm was a copy of `(Y, X)`): the WCS
landed lying on its back. The arm now keeps Z as Z, squares the Y pick
to it, and closes X = Y x Z. The fix is carried on this branch because
the MCP tool would otherwise post a turned frame; the same hunk is
reported for #706 so both land the same text.

**Details chosen.**
- `role` is a lower-case word, `origin | x | y | z` (case-insensitive on
  input): JSON-friendly, and the same four roles the GUI's role box
  offers.
- `setup` is optional and, when given, must be the active setup:
  `SetWcsOrigin` always places the active setup's WCS, and the GUI greys
  Apply on an inactive setup's row for the same reason. Naming another
  setup is refused by name instead of silently activating it - an
  activation is its own undo step and the agent should take it
  deliberately (`activate_setup`).
- Each pick goes through the same id parse, model check and one-pick rule
  (#692, `check_pickable`) every other MCP pick uses; extra keys in a
  pick are refused. A stale id is refused by the law ("... is no longer
  in the model - re-pick it").
- The result carries what was placed (`origin`, `rotation` read back from
  the WCS entity, `word`, `label`) plus the fitted `axes` and
  `angles_deg`, so the agent can check the frame without a second call.

**Considered and dropped.**
- Extending `set_wcs` with a `features` argument: `set_wcs` already has
  two mutually exclusive origin forms; a third with its own refusals
  would make its contract harder to read. A separate tool names the
  GUI action it mirrors.
- A shared `Alignment -> Location` helper used by both the GUI and MCP:
  the GUI's conversion lives in the generic `CommandKind::SetWcs`
  handler (`crates/ui/src/state.rs`) that every WCS voice uses; touching
  it would widen this PR into the UI. The MCP side repeats the one-line
  conversion and a test pins that its quaternion equals the GUI's
  formula bit for bit.
- A script verb: there is no script verb for `set_wcs` either; adding
  WCS verbs to the script language is its own piece of work.

**Cost to change later.** Small: the tool is a thin adapter; any change
to the law in `pick::roles` reaches it automatically.
