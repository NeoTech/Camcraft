# #3873 - UI link pre-hello limits

The UI link accepts one client at a time and requires token authentication on the first line. Prior to this change, the full 5-minute `IDLE_TIMEOUT` and 16 MiB line limit were applied before authentication, allowing an unauthenticated connection to hold the session for up to 5 minutes without sending a valid hello line.

## Decision

1. **Pre-hello deadline:**
   - Prior to token verification, connections are bounded by `HELLO_TIMEOUT` (5 seconds).
   - If a client fails to send a complete, matching hello line within this window, the connection is closed immediately, and subsequent waiting clients are served.
   - `Inbox::link_with_timeouts` permits custom deadlines for deterministic testing.

2. **Pre-hello buffer cap:**
   - Prior to token verification, incoming data is capped at `MAX_HELLO_LINE` (4 KiB) instead of `MAX_LINE` (16 MiB).
   - Any client exceeding 4 KiB before sending a newline is dropped immediately.

3. **Post-authentication limits:**
   - Once a valid hello line with the matching token is received, the normal `IDLE_TIMEOUT` (5 minutes) and `MAX_LINE` (16 MiB) take effect for the session.
