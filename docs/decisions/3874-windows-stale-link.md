## #3874 - Windows UI link mutual token proof and bounded answer reads

On Windows, the UI link discovery file (`<pid>.json`) stores a loopback TCP port (`127.0.0.1`) and a random token. If a window crashes, its file remains and another local process can bind that port. Before this fix, the CLI sent the raw token in the hello line, accepted any peer that answered `HELLO_OK` without proving knowledge of the token, and read answer lines with no size limit.

**What was decided**

1. **Mutual token proof handshake** (`crates/app/src/mcp/link.rs`, `listen.rs`, `link_client.rs`):
   - The CLI generates a fresh nonce (`new_nonce`) for each session.
   - The CLI sends `hello_line` with `token_proof(token, nonce, "client")` and `nonce`, without sending the raw token over the socket.
   - The window checks the client proof in constant time (`hello_matches`) and replies with `hello_ok` carrying `token_proof(token, nonce, "server")`, using pure-Rust SipHash-2-4.
   - The CLI checks the server proof in constant time and refuses any peer that fails to prove the token before sending any tool call.
2. **Bounded answer line reads** (`crates/app/src/mcp/link_client.rs`):
   - `read_answer_line` reads at most `MAX_ANSWER_LINE + 1` bytes (16 MiB limit) and returns `ReadLineError::TooLong` immediately when a line exceeds the limit, without parsing remaining bytes as another message.

**Why**

Mutual token proof verifies that the listening peer actually read the user-only discovery file without exposing the raw token to an untrusted listener. Bounded answer reads prevent a false or broken peer from causing unbounded memory growth or partial-line framing splits. Both stay in pure Rust with no new dependencies and no `unsafe` code.

**What was considered and dropped**

- **PID liveness probing (`pid_alive`) and deleting stale files in `link::entries`:** Dropped. Probing PID liveness requires direct platform FFI dependencies (`windows-sys` on Windows, `libc` on Unix) and `unsafe` calls, remains vulnerable to PID reuse and TOCTOU races, and does not prove that the process listening on the port owns the token. Keeping `link::entries` read-only also preserves stale-file reporting in `connect` and `--ui windows`.
- **Windows named pipes with user-only ACLs:** Deferred as noted in `docs/decisions/3751-cli-ui-link.md`.

**Cost to change later**

Low. The hello protocol stays on `camcraft_link: 1` JSON lines, and `hello_matches` continues to accept legacy `token` hellos alongside `token_proof`. Moving Windows transport from loopback TCP to a named pipe later still changes only the `not(unix)` transport arms of `link.rs`, `wire.rs`, and `link_client.rs`.
