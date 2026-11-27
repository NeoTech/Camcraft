## #3940 - Configure the live MCP idle window

**Decided:** Accept `--mcp-idle-timeout <seconds>` and
`CAMCRAFT_MCP_IDLE_TIMEOUT` at app startup. The option wins. Use whole
seconds that fit `u64`, like the existing integer port parser. An absent
or blank environment value keeps 300 seconds. Invalid values stop startup
with a named usage error. Zero disables idle expiry.

**Why:** The stdio bridge exits when the app drops its client. A user can
keep a long CAM session connected without changing the default for others.
Use the same window for the live TCP listener and the authenticated UI
link. Keep the five-second pre-hello deadline, framing limits, cancellation
and one-client ownership unchanged. Poll for shutdown even with no idle
expiry. No document or machining behavior changes.

**Dropped:** Automatic bridge reconnection is a separate protocol change.
Floating-point seconds add parsing rules that this startup option does not
need. An infinite duration sentinel can overflow deadline arithmetic; use
an explicit zero check and keep elapsed-duration subtraction instead.

**Cost to change later:** Small. Parsing stays with the MCP startup options.
The listener already accepts a duration. Fractional seconds could be added
without changing the wire protocol.
