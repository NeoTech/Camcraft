## #1552 - Document the fit_stock MCP tool

**Decided:** Add the shipped `fit_stock` tool beside `set_stock` in the MCP skill. Document the schema, defaults, millimetres, side order, parts-only scope, preserved rotation, single undo step, response and named refusals.

**Why:** Agents need the same stock-fit contract the MCP implementation already exposes. The row follows `stock_fit_tools.rs`, its parameter parsers and `stock_fit.rs`.

**Considered and dropped:** No runtime changes or new dependencies are needed. A separate guide would split the tool reference. Keep the process version and #681 merge announcement with the merging session, as the issue requires.

**Validation:** A focused xtask documentation contract pin fails without the row and passes with it. Restoring upstream's skill reproduces the failure. This avoids rebuilding the app for a documentation change.

**Cost to change later:** Edit the row and its documentation contract pin together if the tool contract changes.
