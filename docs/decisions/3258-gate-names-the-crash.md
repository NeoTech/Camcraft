## #3258 - The gate names how a test executable stopped; no host lock for the UI steps

**What was decided.** The gate writes one line into the step log when a direct test executable
stops without an exit code of its own: the signal on Unix (`killed by signal 11 (SIGSEGV), core
dumped`), or a known crash status on Windows (`status 0xC0000005 (access violation)`). The step note
carries it: `exit -1 (killed by signal 11 (SIGSEGV))`. The gate does not serialize the UI steps across
runs on one host.

**Why.** The four red runs of #3258 do not point at a resource that two runs share in the repository:

- The UI binaries stopped in the middle of their tests, with no panic. The tests that did not
  finish are the GPU ones: `the_gpu_sees_the_box_the_cpu_sees` and
  `deviation_classes_agree_with_the_cpu_verdict` (trace_parity), the three headless-draw tests of
  egui_app. `exit -1` is the gate's code for "no exit code": a signal killed the process.
- The kernel logs of skull and egg show a segfault in `libvulkan.so` at the failure time. The
  runners then pinned Mesa's software Vulkan (about 22:40 UTC, 2026-11-21). Since then, about 30
  post-merge runs, many of them at the same time on one host, show no UI `exit -1`.
- The wopr run (22:16-22:38) ended before that pin. mkinney found no kernel segfault and no OOM on
  wopr. So the wopr abort is attributed to the driver only by its time, not by a log line.
- The uma `bore_core` failure is a different cause: `PermissionDenied` on a PID-named directory in a
  shared `/tmp` that another runner account owned. #3290 fixed it.

So the cause was outside the repository (a hardware Vulkan driver), and the runner environment
fixed it. What the repository lacked is the diagnostic: the log said `exit -1` and nothing else, and
it took a kernel log on the host to find the signal.

**Dropped.** A per-host file lock around the UI steps (the earlier held patch on
`mkinney:issue/3258-ui-host-lock`). It makes each post-merge run slower when runs overlap, and no
evidence shows that two UI runs on one host conflict once the driver is software Vulkan. If the new
line shows a signal again on the software renderer, that patch is the next step.

**Cost to change later.** Low. The lock can be added around the UI steps in `exec.rs` without a
change to this line.
