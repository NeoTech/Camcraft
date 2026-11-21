## #3105 - Check the default-machine refusal separately from logs

**Decision:** Check the per-operation refusal assertions against the CLI refusal line. Set `CAMCRAFT_LOG` to `info` for the child process so the test always includes generation logs.

**Why:** The complete stderr stream includes a valid generation log for `Face1`. The refusal correctly names only the unmounted setup `Setup2` and its operation `Face2`. The test must check which operation the refusal names.

**Dropped:** Disabling logging would hide the regression. Production posting changes are unnecessary because the refusal is correct. A broad exclusion from stderr would also reject valid generation logs.

**Cost to change later:** Update the test if the CLI refusal prefix changes. Production posting behavior does not change.
