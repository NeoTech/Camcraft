## #2445 - Import the Solidcraft extension alias

**What was decided.** Treat `.solidcraft` as a case-insensitive alias of
`.sc` in `format_from_path` and the native `MODEL_EXTENSIONS` list shared
by the model dialog and viewport drop. List both names in the MCP import
path description. Both extensions use the existing checked cached-body
importer. The browser filter continues to omit Solidcraft designs because
the browser has no native Solidcraft importer.

**Why.** The two names contain the same design container. Routing both to
one importer preserves its validation and avoids duplicate parsing logic.
The successful MCP alias pin imports the existing sample, including its
thumbnail JSON and PNG entries, which the importer ignores.

**Considered and dropped.** A separate parser would duplicate the same
format. Renaming user files would retain the reported workaround instead
of supporting the saved name. Enabling browser imports requires a browser
reader and is outside this extension-alias fix.

**Cost to change later.** If the formats diverge, update the routing,
native filter, MCP description and regression pins together. Four pins
cover extension routing, named MCP refusal, successful MCP import and GUI
drop classification. Restoring either old routing path makes its pins
fail; restoring the fix makes them pass.
