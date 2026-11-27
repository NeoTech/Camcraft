## #4151 - LinuxCNC post header: clear G92 offsets and state G97

**Decided.** The generic LinuxCNC milling post (`assets/posts/linuxcnc.cps`)
writes `G92.1` and `G97` in its startup safety block in `onOpen()`:

- `G92.1` clears any `G92` coordinate system offset active on the LinuxCNC
  control from previous jobs or manual operations. A boolean post option
  `clearG92` (default `true`) in `declaredOptions` controls emission of
  `G92.1`. When `clearG92` is set to `false`, `G92.1` is omitted.
- `G97` explicitly sets constant spindle RPM mode before the first spindle
  start (`S.. M3`), ensuring no modal `G96` (constant surface speed) mode
  left on the control remains active.
- Explain keyword `"G92.1"` is added to `explainWords` with definition
  `"CLEAR G92 OFFSETS"`.
- Pinned by dedicated integration tests in `crates/post/tests/linuxcnc_post.rs`.

**Why.**

- On LinuxCNC, `G92` offsets persist until cleared with `G92.1` (or `G92.2` /
  `G92.3`). An uncleared `G92` offset shifts all work coordinates and can cause
  tool crashes or scrap parts.
- Similarly, if the control previously executed a turning or lathe cycle with
  modal `G96`, LinuxCNC may remain in constant surface speed mode. Emitting `G97`
  in the header ensures the spindle runs at the programmed RPM.
- Adding `clearG92` to `declaredOptions` provides an escape hatch if a user
  intentionally establishes an external `G92` fixture offset before running the
  program.

**Dropped.**

- Emitting `G92.1` in every section preamble (`startPreamble()`): unnecessary and
  potentially disruptive to setups that intentionally modify offsets mid-program.
  Clearing `G92` at program start is sufficient and conforms to RS-274 / LinuxCNC
  standards.
- Emitting `G97` on every tool change: milling operations do not change spindle
  mode mid-program in `linuxcnc.cps`; setting `G97` once in the header safety block
  guarantees constant RPM mode for the whole program.

**Cost to change later.**

- Low. The option schema and safety block are isolated to `assets/posts/linuxcnc.cps`
  and the corresponding test fixtures.
