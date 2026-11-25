## #3414 - The approach rotates before it descends

**What was decided.** The first five-axis approach splits into two moves:
a zero-travel Rapid whose axis is the only change (the rotary index -
the machine swings at a standstill, tip fixed, on the hop plane or at
the tip it already holds), then the Plunge/cut descend under the pose
already stated. Applied in `curve.rs` (every chain's approach) and in
`stamp.rs` (every posed kind, at every rapid-to-cut re-entry where the
pose changes). No new `MoveKind`.

**Why.** A pose change riding the move that reaches first contact
turns the rotaries while the tool descends - the ball sweeps through
unknown air and through the rim it approaches.

**What was considered and dropped.**
- A short travel rapid along the hop plane under the new pose: it
  drags the tip sideways while the head turns, and it blurs the law
  that a travel rapid inherits its pose.
- A new `MoveKind::Rotate`: the post already commands a zero-travel
  rapid's rotary words on `G00` (`fanuc.cps`'s `onRapid` through
  `tilt()`; `solve_g93_segment` hands rapids to the rapid path before
  its feed branches), and the builders already keep a zero-travel move
  whose pose changed - no new verb is needed.

**Cost to change later.** The law tests (`rapids_hold_the_pose_they_inherit`,
`rapid_pose_contract`, the air-link ratchet, `balls_are_the_plan`) read
the swing-free stream through `no_swings`/`sans_swings` helpers. If a
real rotate verb ever lands, those helpers collapse into one place.

## Review round (2026-11-23, unattended)

**What was decided.** The standstill swing now runs in BOTH upright
arms of the stamp (`stamp.rs`), on the reviewer's cut line: reached
out of air, the move is not a Rapid, and a held pose exists and
differs from the upright one. An opening upright statement from no
held pose stays as its own move (#3300); a pose that already matches
drops in the builder. The `no_swings`/`sans_swings` filters keep an
opening upright statement from no held pose: the app ladder's first
move is a zero-travel Rapid, and stripping its #3300 statement
desynced the plain-vs-posed pairing. The branch also merged the
newest `origin/main` (9e5af7b91), past the first refresh, so the
conflicts verdict reads against the main of the moment.

**Why.** The review of ad1e5acba reproduced a 90-degree rotary turn
riding an upright approach at move 261 of the slab-with-boss rough
ladder. The merged #3300 opening statement is a zero-travel Rapid
with axis `Some([0, 0, 1])`; the filters read every zero-travel
rapid with a changed axis as a rotary index, and the pairing assert
`same_balls` failed at move 0.

**What was considered and dropped.**
- A generator-scale pin for the arc arm: `the_approach_rotates_
  before_it_descends` gained `multiaxis_rough` (slab_with_boss) and
  `contour5d`, but no case of that suite reaches the arc arm out of
  air under a lean; the arc arm is pinned by the stamp unit test
  `an_upright_arc_out_of_air_swings_at_a_standstill_first` instead.
- Fixing the Windows-checkout CRLF failure of the drill-overfeed
  refusal golden inside this PR: filed as #3628 (a gate fix that
  blocks Windows contributors), out of this PR's scope.
