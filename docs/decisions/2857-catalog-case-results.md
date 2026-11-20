## #2857 - Catalog case results

**Decided:** Give each of the five catalog cases separate `regenerates`
and `rebuilds` tests. The ten tests have independent libtest results.
A discovery test requires both tests for each catalog directory and keeps
the required seed case. Move the shared assertions to a support module
to keep each file below about 400 lines. Keep the existing golden assertions.

Each case has a 60-second wall clock budget. Use the worker and channel
pattern from `import_step_guard`. Keep the original panic when a case fails.
Write each case's elapsed time directly to stderr on success and failure.
The timeout bounds the test result; it cannot cancel the worker thread.
Each rebuild removes only its own temporary case directory, so parallel
tests cannot remove another case's files.

**Why:** A failure must name its case without hiding another case's failure.
All ten case workflows stay below the budget in debug mode. Keep them in
the default lane. A future slow case must use a smaller model or move to
`app heavy: example_catalog`, with that lane in its ignore reason.

**Dropped:** Keep no aggregate loop, because it gives one result and adds
all case times. Add no test harness dependency, because std and libtest
provide the required results. Move no current case to the heavy lane,
because each case is below 60 seconds.

**Cost to change later:** Add two test wrappers for each new catalog case.
The discovery test reports a missing wrapper. A different timeout or
cancellation mechanism changes the shared support module.
