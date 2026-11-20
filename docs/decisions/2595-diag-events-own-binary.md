## #2595 - the diag event-level test runs in a test binary of its own

**Decided:** `diag_lines_are_tracing_events_at_their_level` moves from the
`diag` unit tests in the `camcraft-ui` lib to `crates/ui/tests/diag_events.rs`.
That binary holds this one test only. The test calls `diag::warn` and
`diag::info` directly. The lib keeps `the_macros_take_println_shaped_arguments`,
which compiles the two macros.

**Why:** `tracing` caches the interest of each callsite for the whole
process. The test installs a thread-local subscriber with
`with_default`. In the lib binary, other tests on other threads call
`diag_info!` through the same callsite. When another thread hits the
callsite first, it registers the callsite with the dispatchers that it
sees at that time. If that registration ends after the rebuild that the
new subscriber started, the cached interest is "never". The INFO event is
then lost, and the WARN event (another callsite) stays. In a binary of
its own, no other thread uses these callsites.

**Dropped:**

- `tracing::callsite::rebuild_interest_cache()` before each event. This
  makes the window smaller, but another thread can still write a stale
  interest after the rebuild.
- A lock shared with each test that emits a `diag` line. Nearly each UI
  test emits one, so the lock would serialize the lib suite.
- A global default subscriber for the lib tests. That changes the output
  of each other test.

**Cost to change later:** low. The test is one file. A second test in
`diag_events.rs` brings the race back; the file header says so.
