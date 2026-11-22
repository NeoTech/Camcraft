## #3486 - Correct the expected opening pose in multiaxis tests

**What was decided:** Correct only the tests. Keep the production behavior
from #3300, which states the upright pose before the first tilted feed.
Initialize the expected held pose to upright in all three affected tests.
Assert the upright opening pose explicitly. Keep the exact rapid pose
comparisons and the existing ball and cutting checks. Rename the roughing
test to `rapids_hold_inherited_pose_and_flanks_lean` to describe its checks.

**Why:** All three tests failed because the opening rapid stated
`Some([0.0, 0.0, 1.0])`, while the tests expected `None`. The explicit opening
pose check passes. The stated upright pose establishes the rotary position
before a tilted feed. The test expectations must include that safety behavior.

**Considered and dropped:** Reverting the stamp would remove the rotary
safety behavior from #3300. Looser pose comparisons could hide a rapid that
changes its inherited pose. Neither change is needed.

**Cost to change later:** A change to the opening pose contract must update
these three tests and the opening rotary pose tests. It must also prove that
the post establishes the rotary position before a tilted feed.
