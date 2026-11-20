## #2567 - Discover known failures from test markers

**What was decided:** Use one structured ignore marker for a test that is
red on purpose. Discover the marker from the test source. Store owned
known-failure descriptors instead of a fixed list. Put a marker immediately
before the expected failing assertion. Check the panic location and message
line prefix. Check the upstream issue state in both release commands.

**Why:** The release gate must distinguish an expected failure from a crash
or another failed assertion. Source markers keep the exception with its
test. A passing test or a closed issue must fail the gate so the stale
exception is removed. Keep the existing warning if `gh` cannot read the
issue state. The expected failure check still runs in that case.

**Considered and dropped:** Keep the three-test static list. It makes each
new exception require gate code changes. Add a custom procedural macro or a
dependency. The source scanner can read this small marker without either.
Use a blanket ignore. It hides new failures and tests that now pass.

**Cost to change later:** A marker format change must update its parser,
marked tests, gate pins and review rule together. Requiring an issue state
when `gh` is unavailable would also require reliable credentials and network
access in each release environment.
