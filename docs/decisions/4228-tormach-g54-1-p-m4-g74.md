## #4228 - Tormach mill post: G54.1 P offsets, M4 and G74, and control cutter compensation

**What was decided.**

- Raise `workOffsetMax` to 500 in `assets/posts/tormach.cps`. Format offsets 10 through 500 as `G54.1 P<n>`. Keep offsets 1 through 6 as `G54` through `G59`, and offsets 7 through 9 as `G59.1` through `G59.3`. Raise `MAX_BOUND` in `camcraft_post::post::work_offset` to 1000.
- Declare `millSpindleReverse = true;` in `assets/posts/tormach.cps`. Start the spindle with `M4` when `PARAM MillSpindle` is `reverse`. Reset the direction to forward (`M3`) for subsequent operations. Remove `Tormach PathPilot` from `M3_ONLY_POSTS` in `tool_gate.rs`.
- Refuse left-hand rigid tapping (G74) by name; Tormach's post processor guidelines list G74 left-hand tapping as unsupported. Recommend thread milling instead. Right-hand rigid tapping continues to emit G84.
- Implement control cutter compensation (`G41`, `G42`, `G40`) with the `D` word in `assets/posts/tormach.cps`. Disallow cutter compensation engagement on `G0` moves or arcs.
- Pin the behavior with unit tests in `crates/post/tests/tormach_post.rs`.

**Why.**

- Tormach PathPilot is built on LinuxCNC. LinuxCNC supports 500 coordinate systems through `G54.1 P1` through `G54.1 P500`. The post previously refused offsets above 9.
- Left-hand cutting mills require reverse spindle rotation (`M4`). The post previously started the spindle with `M3` for all operations. The engine refused reverse mill spindle operations because `millSpindleReverse` was undeclared.
- While LinuxCNC supports G74, Tormach's PathPilot post guidelines explicitly classify G74 left-hand tapping as unsupported. The post refuses the request by name to prevent unsupported G-code execution.
- PathPilot supports cutter radius compensation with `G41` and `G42` on straight lines using `D<tool_number>`. The post previously refused control compensation.
- Programs with standard right-hand tools and standard offsets 1 through 9 produce the exact same output as before.

**Considered and dropped.**

- Drop control cutter compensation and record a refusal. Implementing `G41`, `G42`, and `G40` follows the proven Haas and Fanuc post patterns and satisfies user needs on PathPilot machines.
- Format offsets 10 through 500 as `G54.1 P1` through `G54.1 P491`. PathPilot and LinuxCNC map `G54.1 P<n>` directly to coordinate system index `n`.

**Cost to change later.** The implementation follows the standard LinuxCNC and PathPilot programming manual. The changes are local to `assets/posts/tormach.cps` and `tormach_post.rs` with the global `MAX_BOUND` raised to 1000.
