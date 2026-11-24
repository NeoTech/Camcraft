## #3751 - `camcraft-cli --ui`: the UI link (part 1)

The maintainer asked for a `--ui` flag that sends CLI commands to the
running UI. The issue left five design questions open. Each answer below
was taken unattended, as the recommended option.

**1. Transport.** Each window opens a local socket at start.
- Unix (Linux, macOS): a Unix domain socket `<pid>.sock` in the link
  folder (std `UnixListener`). No network port.
- Windows: std has no named pipe and no local socket. The window binds
  TCP on `127.0.0.1` with a port that the OS selects. No other interface
  is bound. A named pipe needs a new dependency (`windows-sys` with
  `Win32_System_Pipes` in `camcraft-app`, or the `interprocess` crate,
  which is not in `Cargo.lock`). That is the maintainer's decision; it is
  asked on #3751. The token (point 2) closes the port to other users and
  to web pages until then.
- Dropped: loopback TCP on every platform (one code path, but a port
  where std has a better door); a raw named pipe through FFI (unsafe
  code and overlapped I/O for the deadlines, for a first slice).
- The listener is the one that `--mcp-port` uses (`listen.rs`), with a
  stream type that hides TCP or Unix (`wire.rs`). The same bounds hold:
  one client at a time, a 16 MiB line cap, write deadlines, the idle
  drop, and a dropped client is logged, never a panic.

**2. Discovery and security.** The window writes `<pid>.json` in the
link folder: pid, transport, address, a token, the open project, the app
version. The window removes it when it closes.
- The folder: `CAMCRAFT_UI_LINK_DIR`, else `%APPDATA%\camcraft\ui-link`
  (Windows), `~/Library/Application Support/camcraft/ui-link` (macOS),
  `$XDG_RUNTIME_DIR/camcraft/ui-link` or the XDG data folder (Linux).
  On Unix the folder is 0700 and the file and socket are 0600.
- The first line of each connection must be
  `{"camcraft_link":1,"token":"<token>"}`. Any other line closes the
  connection before a line reaches the app. The compare takes the same
  time for each token of the same length.
- The token is 256 bits of SipHash output under std's `RandomState` keys,
  which std seeds from the OS random source, mixed with the clock and
  the pid. Dropped: a new `rand` / `getrandom` dependency for
  `camcraft-app` (both are only in `Cargo.lock` through other crates).
- The client never sends a token to an address that is not loopback,
  whatever the file says.
- The link is on by default, as the issue asks. `CAMCRAFT_UI_LINK=off`
  starts a window without it. A window that cannot open its link still
  runs, and its status line says why.

**3. Protocol.** After the hello, the session speaks the MCP JSON-RPC of
`camcraft-cli mcp`. The window answers each line in `mcp_host::pump`
with `answer_live_with`, exactly as for `--mcp-port`: on the UI thread,
on the window's History, behind the same live gate. So `post`, file
writes and machine edits still need the approval in the window. There is
no second command set.
- CLI verbs: `--ui call <tool> [<json-args>]` (one tool call; prints the
  tool's JSON; a refused tool exits 1), `--ui mcp` (a stdio MCP bridge
  to the window), `--ui windows` (the link files).
- `script` and `process` with `--ui` are not in this slice. A script
  runs its verbs in a headless world of its own; to run them on a
  window needs the script host to send each verb over the link.

**4. No window.** The CLI says "no running CamCraft window ... (nothing
ran)" and exits 3, a code of its own, so a test runner can tell "the UI
is not up" from a refusal (1) and a usage error (2). A stale file of a
crashed window is skipped (its socket does not answer). The CLI never
falls back to a headless document.

**5. Several windows.** `--window <pid>` or `--window <project path>`
selects one. The window rewrites its file when its project changes.
With one window answering, the CLI uses it. With several and no
`--window`, the CLI lists them and exits 2.

**Cost to change later.** The file format has `format: 1`; a reader
skips other versions. A named pipe on Windows replaces only the
`not(unix)` arms of `link.rs`, `wire.rs` and `link_client.rs`.
