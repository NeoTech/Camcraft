## #3879 - Report GPU coverage without requiring an adapter

**Decided:** Keep the post-merge policy at "skip, but report". Do not set
`CAMCRAFT_REQUIRE_GPU` or change the runner configuration in this change.
A successful process with a GPU skip has a `SKIP` step row. Its note lists
each skipped test and the number of GPU tests that did not execute.
The process can still return zero. This is not a claim of GPU coverage.
CPU assertions in a mixed test can still execute.

**Why:** A runner can lack an adapter or use a software adapter. A green
process exit does not prove that a GPU assertion ran. Keep the existing
fallback and refusal policies, but make the missing coverage visible.
`CAMCRAFT_REQUIRE_GPU=1` still fails where the existing harness requires it.
A switch-off or compute refusal is reported as `GPU unavailable`, not as a
missing adapter. A missing adapter uses `SKIPPED (no adapter): <test>`.

Use libtest's `--show-output`, not `--nocapture`. Captured output keeps the
test name when tests run in parallel. Both cargo and direct gate steps use
these arguments. Repeated skip messages in one test count once. The parser
accepts the existing GPU skip messages, including child-process fault tests.
An ungrouped message has an unknown test identity; it is not an inferred pass.

The adapter report uses wgpu's returned name, backend and device type.
CPU devices are software. Integrated and discrete GPUs are hardware.
Virtual and other devices have unknown hardware identity. A device report
alone does not prove that a compute or parity assertion executed. Software
execution does not prove hardware parity. Adapter selection is unchanged.

**Dropped:** Requiring hardware or changing the post-merge environment.
Those options need a separate runner policy and evidence from its owner.
Also dropped uncaptured parallel output: adjacent lines cannot reliably
identify the test that skipped.

**Cost to change later:** The runner owner can enable the existing require
flag separately. No numerical tolerance, compute kernel, parity assertion,
CPU fallback, workflow, trigger or runner label changes here.
