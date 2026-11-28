## #4347 - Doosan DNM: no G5.1 word inside a canned-cycle group

**What was decided.** The Doosan DNM post writes no `G5.1 Q1` while a
canned-cycle group is open. `onLinear` and `onCircular` call
`enableSmoothing()` only when `inCycle` is false. `endPreamble` writes `G80`
before `disableSmoothing()`, so a `G5.1 Q0` never comes while G81-G89 is
modal (FANUC 0i-F B-64604EN-2/01, 3.3.2).

**Why.** The hole chain of a cycle (plunge, retract to R) reaches
`onLinear`. The old code armed smoothing before `chainCut()` dropped the
move. Thus each hole wrote `G5.1 Q1 R<n>`, and the next cycle block or the
section end wrote `G5.1 Q0`, all inside the group. The control alarms on a
G code that AICC mode does not permit.

**Considered and dropped.**
- Move `enableSmoothing()` after `tilt()` and `chainCut()`. Dropped: `tilt()`
  can write blocks, so the order of `G5.1 Q1` against those blocks would
  change in every smoothed program outside a cycle. The `!inCycle` guard is
  the same test that `chainCut()` uses and keeps all other output
  byte-identical.
- Arm smoothing again after the `G80` that a hop rapid writes. Dropped: the
  next cutting move arms it, as before.

**Limits.** After the `onLinear` guard, smoothing cannot be active while a
group is open, so the `endPreamble` reorder has no independent pin. It is a
second guard for a future path that arms smoothing inside a group. The
mutation run with the old `onLinear` shows that the reorder alone writes
`G80` before `G5.1 Q0`. No stream in the tests puts an arc inside a cycle
group, so the `onCircular` guard is not pinned on its own.

**Cost to change later.** Low: three lines in one post script.
