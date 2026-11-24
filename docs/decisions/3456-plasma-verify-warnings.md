## #3456 - plasma law warnings are verify findings and operation warnings

**What was decided.** The plasma clause keeps each non-blocking finding
(tip-up, kerf crossing) as data: `posting::GateWarning { op, kind,
message }`. `GateReport` carries them in a new `warnings` field. `verify`
(script and MCP) adds each one to the `warnings` row of its operation and
to a new `gate.warnings` list. MCP `verify` lists each one in `findings`
with `kind` `plasma_tip_up` or `plasma_kerf_crossing`, the op label, no
move, and `blocks_post: false`.

**Why.** The plasma laws found the crossings, but the only place that
said so was a sentence inside `gate.not_judged`. A client that reads
`findings`, the op `warnings` or `ok` saw a clean program. The tip-up and
kerf-crossing laws warn (`PlasmaFinding::refuses` is false for them), so
`ok` and `held_by` do not change: the warning is visible and does not
hold the post.

**Dropped.**
- Remove the warnings from the `not_judged` line. The G-code viewer
  header in the UI shows only that line, so it is the one place where the
  UI shows these warnings. The line stays as it is.
- Make a tip-up block the post. The law states a warning; changing the
  law is a separate decision.
- Change the cut order or the rapid route to avoid the freed slugs. That
  is generator work, not a verify defect; it can be a follow-up.
- Add the warnings to the cached `Toolpath::warnings`. They are found at
  the program gate, not at generation, and the cache is per op.

**Cost to change later.** Low. `GateWarning` is a plain struct; another
program law can push into `GateReport::warnings` and the surfaces show it
with no more change.
