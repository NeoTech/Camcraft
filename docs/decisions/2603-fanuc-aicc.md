## #2603 - Fanuc post: per-operation AI contour control (G05.1)

**Decided.** The built-in Fanuc post has two new options in the "Smoothing"
group:

- `aiContourControl` (boolean, default off).
- `aiContourLevel` (integer 1-10, default 5).

With the option on, each operation writes `G05.1 Q1 R<level>` as the last
preamble block (after the tool change, `G43 H`, spindle, coolant and any
`G68`). It writes `G05.1 Q0` at the end of the operation: after `G80`,
before `G69`, `M9` and `M5`. The words use the two-digit form `G05.1`, as
the machine owner writes them and as this post writes `G00` / `G01`.

**Why.** The owner of the 31i-A5 Robodrill gave the form: "G05.1 Q1 R5 in
the start of op and then G05.1 Q0 at the end". The R level is a machine
setting, so it is an option with the owner's default R5. Other Fanuc
controls can lack the AICC function, so the option is off by default and
the default output does not change (no golden moves).

**Guards.**

- A canned cycle writes `G05.1 Q0` before the cycle block. The Doosan DNM
  manual cited in `doosan-dnm.cps` requires Q0 before G73-G89. We use the
  same conservative rule on all Fanuc controls.
- A section with the spindle stopped (a touch probe with `G31`) does not
  arm AICC.
- A level outside 1-10 is refused by name once (`message()`), and no
  `G05.1` is written. The host also refuses it from the declared range.
- With the option on, the `#616` note "SMOOTHING NOT WRITTEN" is not
  written, because the post now writes the control's smoothing words.
  The op's Smoothing tolerance is still not held by `G05.1`: the deviation
  is the control's own parameter.

**Dropped.**

- Arm AICC only for operations that ask a Smoothing tolerance: the owner
  arms it on every operation, and `G05.1` states no tolerance.
- Keep AICC on through canned cycles: not confirmed for the 31i-A5
  (question 1 on rigid tapping is still open).
- `G5.1` (one-digit form): equivalent on the control, but the owner writes
  `G05.1`.

**Still open on the issue.** Questions 1 and 2 (M29 before G84, G73 or G83,
the R plane) wait for the owner's answers. They are not part of this
change.

**Cost to change later.** One option block and three functions in
`assets/posts/fanuc.cps`, pinned by `crates/post/tests/fanuc_aicc.rs`.
