## #1855 - Move multi-pass field parity to the release heavy lane

**What was decided:** Mark `multi_pass_contour_simulates_with_meridian_parity`
as ignored in debug. Name the release heavy lane in its ignore reason.
Keep its fixture and all assertions unchanged. Pin the ignore attribute and
its release plan selection in `xtask/src/heavy_tests.rs`.

**Why:** The issue reports about 107 seconds in debug. The prior issue
attempt measured 182.45 seconds. The testing rule moves tests above about
60 seconds to the heavy lane or reduces their fixtures. The release planner
already selects ignored app tests, so this change needs no new routing logic.

**Considered and dropped:** A smaller fixture could reduce debug time, but
could change the sample density or the path. Splitting this
single field-engine job would not reduce its cost. Neither option is needed
to keep the current coverage in the release lane.

**Cost to change later:** Remove the ignore attribute and update its routing
pin after a measured fixture or engine improvement brings debug time below
60 seconds. Keep the parity assertions in either lane.
