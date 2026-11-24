# #3865 - CLI UI Busy Window Diagnostic

The UI link serves one client at a time. During a slow call or active session, a second client connecting to the link times out during its handshake after `HELLO_TIMEOUT` (5 seconds).

Previously, the second CLI reported that the link file was stale (exit code 3), misattributing the timeout and prompting users to restart the window or delete active link files.

## Decision

1. **Distinct Error and Exit Code:**
   - When a hello connection or handshake read times out on an active window, the CLI reports `the CamCraft window <who> is busy with another call (nothing ran)`.
   - The CLI exits with code 4 (`LinkError::Busy`), distinct from code 3 (`LinkError::NoWindow`, indicating stale or missing link files).

2. **Usage String Update:**
   - `UI_USAGE` updated: `Exit: 0 stood, 1 refused, 2 usage, 3 no running window, 4 window busy`.
