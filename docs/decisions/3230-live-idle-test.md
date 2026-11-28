## #3230 - Synchronize the active-client idle test

**Decided:** Keep the listener unchanged. Use its existing request/answer channel to synchronize eight real TCP ping exchanges. Queue each next complete message before answering the current request. Delay the owner's answer by one quarter of the idle window. Assert all eight successful replies and an elapsed duration of at least two windows. Then stop sending on the same connection and require an idle notice followed by EOF.

**Why:** The old client slept for 150 ms after each reply with a 400 ms idle window. Reply delivery and client scheduling consumed the remaining budget. A temporary Linux probe added a 300 ms delay after each reply. The old test failed with `BrokenPipe`. The synchronized test passed with that same delay and the original 400 ms window. Thus synchronization, not just a larger deadline, removes the demonstrated gap. The final two-second window allows bounded scheduling slack; all socket and request waits are bounded to three windows. The elapsed assertion scales with this window, so increasing it cannot hide a missing refresh.

**Regression checks:** Temporarily removing the complete-message activity refresh made the corrected test fail with `ConnectionReset` during its active replies. Temporarily disabling idle expiry made it fail while waiting for the final idle notice. Both mutations were removed before final verification. The existing silent-client, slow-loris, configuration, and hello/security tests remain unchanged.

**Dropped:** Retry loops, disabled expiry, accepted connection errors during active exchanges, production clock or listener changes, new dependencies, and a timing framework. These would weaken the contract or exceed this test-only change.

**Cost to change later:** Only this test needs adjustment if the request/answer synchronization contract changes. This is Linux evidence for portable standard-library code, not Windows reproduction or certification.
