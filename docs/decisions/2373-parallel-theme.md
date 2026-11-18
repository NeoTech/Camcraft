## #2373 - Isolate the live theme between parallel unit tests

**Decided:** Keep the production live theme in its process-wide
`OnceLock<RwLock<Tokens>>`. Under `cfg(test)`, use a thread-local
`Cell<Tokens>` initialized to `Tokens::DARK`. Extract the storage helper to
`theme/live.rs` so the theme table stays near the 400-line limit.

**Why:** Rust's unit-test harness runs tests on separate threads. A test
that selects Light must not change the ground colors another test uses
when scanning overlay colors. Color-vision selection already belongs to
each world, but `palette_of` also reads the live theme's ground. Isolating
that shared input lets the existing tests run in parallel without changing
the application's runtime theme behavior.

**Considered and dropped:** Adding `snapshot_guard()` around every theme
reader and writer would serialize the tests and rely on every future
consumer remembering the lock. Using thread-local storage in production
would change runtime behavior across threads. Running all tests serially
would hide the interference instead of isolating their state.

**Limits and later cost:** This test-only isolation does not prove that
production theme access is safe across a sequence of operations on several
threads. A future unit test that renders on another thread must explicitly
install its tokens there. Changing the storage policy later is a small,
local helper change; callers keep the same `current` and `set_current`
interface.
