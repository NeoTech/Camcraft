## #3289 - Reserve separate bore fixture directories

**Decided:** Reserve each bore fixture directory with atomic `create_dir`.
Number the candidates within the test tag and process ID, skip
`AlreadyExists`, and report other errors with the directory name. Use the
same helper for STL fixtures and the save/reopen test's output directory.

**Why:** The saved CI log shows four tests failing at `write stl` with
`PermissionDenied` on line 116. The old helper reused a PID-based path and
ignored errors when deleting it. A reused PID can select another runner
user's leftover files. Reserving a new directory avoids touching those
files and isolates repeated uses of a test tag. The reopen test must
reserve its output directory explicitly because fixture creation no longer
creates the old predictable path.

**Considered and dropped:** Retrying whole tests would hide the fixture
failure. Changing permissions or deleting another owner's files would
keep the ownership problem. Timing assertions would not pin directory
isolation. An extra dependency is unnecessary for atomic directory
reservation. No product code or geometry behavior changes.

**Validation:** `bore_fixtures_preserve_existing_pid_paths_and_isolate_repeated_tags`
fails on unchanged upstream code and passes with the fix. Restoring the
upstream file while retaining the pin makes it fail again; restoring the
fix makes it pass. All six `bore_core` tests, the final quick gate, and
`camcraft-app` clippy pass.

**Cost to change later:** Update only the test directory helper and its
pinning test. No project format, public API, or machining behavior depends
on the directory naming scheme.
