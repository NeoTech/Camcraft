## #797 - the machine check reads a 4th-axis table's pose through the tool axis

**What was decided.**

- A machine counts as a 4th-axis table when it has exactly one rotary and that rotary states a direction along X. "Along X" uses the PathPilot post's own test: x² ≥ 0.998·|d|². On such a machine, the pose check (`machine/judge.rs`, `pose_problems`) no longer judges the canonical letters one by one.
  - It reads the tool angle about X from the letters with the post's formula (`tormach.cps` `toolAngle`) and the same tolerances.
  - A pose whose recomposed axis has an X component over 1e-9, or a C over 1e-6, is refused by name: "...'s only rotary, A about X, cannot reach that pose".
  - Otherwise the angle, negated for a `[-1,0,0]` direction the way the post writes the word, is judged against the rotary's travel. So the face underneath (A0 B180) is accepted as A180 and refused only by the stated travel.
- Every other machine keeps the letter law unchanged: no rotary, two or more rotaries, or one whose direction is unstated or not along X.
- `fanuc.cps` now refuses a turning letter that has no rotary of that name ("tilted move commands B - the machine states no B rotary"). It writes the stream's letters as they come. Without this refusal, the A0 B180 the check now passes would reach an A-only control as a B word. The check used to stop that, and now the post does.

**Why.** The check and the post must agree on what a pose is. The post had already learned to read A0 B180 as A180 (#785), but the check still refused it, so a 4th-axis job could not cut the face under the bar. For a single rotation about X, reading the pose through the axis is exact: such a table reaches exactly the axes with no X component. That makes it a reachability fact, not IK.

**Considered and dropped.**

- Making the check aware of which post is selected, so it would accept A0 B180 only under PathPilot. That ties the machine gate to a post name, and the gate is machine-only by design. Making the letter-writing post refuse letters it cannot honour is the honest place for that wall.
- Requiring the single rotary to be named "A". Reachability is about the direction, not the letter. A post that needs the name (PathPilot does) refuses by name anyway.
- Unwrapping the angle across moves for a travel like 0..360. The check judges each pose's angle in (-180, 180], exactly as the post judges its A word, so a pose at -90 on a 0..360 table is refused by both. That is conservative and consistent; unwrapping belongs with the simultaneous-4-axis follow-up (#796).

**Cost to change later.** Small. The reading lives in `judge.rs` (`only_x_rotary`, `x_tool_angle`, `x_rotary_problem`) and is pinned by `crates/app/tests/fourth_axis_pose.rs`. A machine with an A about X and a second rotary keeps the letter law by test.
