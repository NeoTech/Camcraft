# 4021 - make the ai_chat cancel test deterministic

Chosen: the blocked test worker (`BlockingTransport`) now ends only when the
cancel flag is set. The 5 s clock in the worker, in `pump_until_idle` and in the
start wait of the cancel test is now `HANG_GUARD` (120 s). It only stops a
hung suite. No test outcome depends on it.

Why: the worker started its 5 s clock at the start of the turn. If the UI thread
stalled for more than 5 s before the Cancel click (a slow Windows box), the worker
gave up first and `saw_cancel` stayed false. A new pin sleeps 6 s before the
click and failed before the fix.

Dropped: a channel for `started`. The `AtomicBool` already signals it, and the
cancel flag itself is an `AtomicBool` by design. The assertions of the test are
unchanged.

Review correction: both cancellation tests use one startup helper with
`HANG_GUARD`. A missing transport start fails with a clear diagnostic. This
keeps the startup wait bounded even when the request ends before `complete`
runs. The pin uses a short deadline to check the missing-start case.

Dropped: separate startup loops. They allowed the stall test to wait forever.
Changing this choice later only changes the test helper and its callers.
