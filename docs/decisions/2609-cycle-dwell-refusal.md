## #2609 - Refuse unsupported G73 and G85 dwell

**Decided:** Add a positive-dwell guard for G73 and G85 in the LinuxCNC,
Tormach, and grblHAL posts. Keep the existing G83 guard unchanged.
The refusal names the cycle and the requested dwell in seconds.
It runs before a cycle block or a modal X/Y continuation can be written.
A zero dwell keeps its existing output.

**Why:** The public LinuxCNC G-code reference gives G73 and G85 no P word.
The PathPilot and grblHAL sources cited by these posts follow that behavior.
The posts must refuse the requested dwell instead of silently dropping it.

**Dropped:** Do not write an unsupported P word. Do not substitute G82 or
G89, or expand the cycle into motions. Those choices change the cycle
behavior and need a separate design.

**Branch:** Stack on PR #2310 because that PR adds the LinuxCNC post.
Change its implementation only to add the dwell guard.

**Cost to change later:** Controller support would need a documented
decision for each dialect and updated pinning tests.
