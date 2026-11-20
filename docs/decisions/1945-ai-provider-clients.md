## #1945 - AI provider HTTP clients, keychain keys and offline-only mode

**What was decided.** A new `camcraft_app::ai` module holds the provider
transport: `endpoint` (URL parse, loopback test, the offline and https laws),
`presets` (Anthropic, OpenAI Chat, OpenAI Responses, Gemini over https;
Ollama, Unsloth Studio, LM Studio and llama-server on loopback), `wire` (the
request path and body per provider, echoing tool-call envelopes so reasoning
items and Gemini thought signatures survive), `probe` (Ollama `/api/show`
capabilities), `secret` (an opaque `ApiKey`, environment first, then the
keychain), and, native only, `http` (the client) and `keychain`. The AI Chat
dock uses it through `transport_http::HttpTransport`; the browser build keeps
the offline-only default transport and links no socket or keychain code.
`prefs.json` gains `aiOfflineOnly` (written only when on), and the dock's
"Offline only" switch is saved there.

**New third-party crates (need the maintainer's decision).** `reqwest 0.12`
(default features off, `rustls-tls-webpki-roots`) provides HTTP(S).
`tokio 1` uses `rt`, `time`, `net` and `macros` for a current-thread runtime
per request. `keyring 4.2` provides macOS Keychain, Windows Credential
Manager and Secret Service over zbus. `url 2.5` was already in the lock and
is now a direct dependency. These dependencies still need the maintainer's
decision. This change adds no dependency between CamCraft crates.

**Why these.** The first version used blocking `ureq 2.12`. Review #2122
found that its active request could not be cancelled. A silent server could
therefore block chat shutdown for the full request timeout. The client now
selects between the async request and a cancellation check every 20 ms.
Cancellation drops the request and its runtime, which closes the active
connection. The Ollama capability probe uses the same cancellation path.
A detached blocking request was rejected because it would remain active
after cancellation. rustls avoids system TLS dependencies. keyring 4's
Linux store uses Rust over zbus, so the release runners need no `libdbus`
build package.

**Provider errors remove secrets.** JSON and plain-text provider errors
remove the supplied API key before the message is truncated. This also
removes a key that crosses the text limit.

**Offline-only is enforced in the client, twice.** `endpoint::admit` refuses
a non-loopback host before anything resolves; the client's resolver then
refuses any resolved address that is not loopback (a hosts-file `localhost`
pointing elsewhere). No redirect is followed: a local server could otherwise
bounce a call to a remote host. `no_proxy` disables environment proxies,
so a loopback request cannot use a remote proxy. Plain `http://` off the machine is refused
whether or not a key is sent - cloud providers are https only.

**Unsloth Studio and llama-server bases.** The issue names
`http://127.0.0.1:8888` for Unsloth Studio; the base is
`http://127.0.0.1:8888/v1`, the OpenAI-compatible mount every other local
runtime here uses. llama-server's documented default port is 8080; its label
says to start it with `--jinja` for tool calls.

**A model without tools refuses, it does not silently degrade.** When an
Ollama model in the native-tools preset lists no `tools` capability, the
step is refused by name, pointing at the Prompt-JSON and Q&A presets. Falling
back inside the transport would send a request without the prompt-JSON
context (the transport has no project snapshot) and the reply would not parse.

**Considered and dropped.** Blocking `ureq` (no active request cancellation);
`native-tls`
(OpenSSL headers on Linux); keyring's `dbus-secret-service` store (vendored
libdbus C build); storing keys in `prefs.json` encrypted (the issue forbids
any key there); an MCP tool to set a key (an agent should never handle a
user's secret).

**Cost to change later.** Low for presets and wire shapes (data and pure
functions with loopback tests); swapping the HTTP crate touches only
`ai/http.rs`.
