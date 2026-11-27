## #3030 - Shared Help tips

**Decided:** Keep the four tips in `assets/tips/tips.json`. Embed and parse
that file once through `app::mcp::tips`. Help, the start page and `list_tips`
read that same source. Resolve `{undo}` to Cmd+Z on macOS and Ctrl+Z on
Windows/Linux. The MCP server uses its own OS.

**Why:** JSON and serde are already used. Embedding avoids an installation
asset-path requirement. The app module owns the MCP response, not CAM
behaviour. No new crate or dependency is needed.

**Decided:** Help opens an ordinary, closable window. The start-page tip
sits below the normal actions. It is hidden during generation, import, AI
work and stock precalculation. Native builds select a tip by UTC day;
browser builds use the first tip without a native clock call.

**Decided:** Store the start-tip choice in the existing per-user
`prefs.json`. An absent `showStartTip` means on. Write it only when off.
The checkbox in Help can restore the start tip. Neither view state nor
this user preference changes the project or undo history.

**Dropped:** A site Tips page is optional and is not added. There is no
second text store. No new dialog action is invented: each Show me action
opens an existing guide page or the example catalog. The example tip does
not promise a project for every operation. The undo tip distinguishes an
attached agent's shared history from a headless document's history.

**Cost to change:** A tip or link changes one JSON row and its content
pins. A new action type would need a separate design and tests. Browser
daily rotation would need a browser clock source; it is not needed for
the optional start-page tip.
