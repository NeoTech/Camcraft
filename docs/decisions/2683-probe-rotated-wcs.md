## #2683 - LinuxCNC and Tormach probe posts: a rotated work offset stops the offset write

**What was decided.** Before each `G10 L20` that writes X or Y,
`tormach.cps` and `linuxcnc.cps` write a run-time test of the XY rotation
of the section's work offset and of the target work offset. A rotation
that is not zero shows `(MSG, PROBE: G54 OR G55 XY ROTATION NOT ZERO -
NO OFFSET WRITTEN)` and ends the program with `M2`. The offset is not written.

```
o903 if [[#5230 NE 0] OR [#5250 NE 0]]
(MSG, PROBE: G54 OR G55 XY ROTATION NOT ZERO - NO OFFSET WRITTEN)
M2
o903 endif
G10 L20 P2 X[#5420-#190] Y[#5421-#191]
```

When the target is the section's own work offset, the test names one
register (`o903 if [#5230 NE 0]`). A result that sets only Z has no test.
A measure-only result has no test.

**Why.**

- The rotation is a register of the control (`R` of `G10 L2`, parameter
  5210 + 20 x the system number: `#5230` for G54). The post cannot read it
  at post time. Only the program can test it.
- The rule "target = active offset + deviation" (#2610) is proved for
  unrotated systems. With all rotations zero, the guard never fires and the
  proof holds.
- Case 1, the target has a rotation that differs from the active system's.
  This is wrong by algebra, with no doubt about the control. Let the active
  system have origin `a` and rotation `α`, the target rotation `τ`, the
  machine position `m` and the deviation `d`. The write gives the target
  origin `t' = m - R(τ-α)(m-a) - R(τ)d`. This depends on `m`. The target is
  correct at the probe position only. At a distance `r` from that position
  the error is `2 r sin(|α-τ|/2)`. `G10 L20` does not write `R`.
- Case 2, the two rotations are equal (the target can be the active
  system). The LinuxCNC reference says that `G10 L20` makes "the current
  coordinates become the given value". For the written axes this gives the
  correct result. For an axis that has no word (an edge probe writes X
  only), the reference does not say if the control keeps the coordinate
  along the rotated axis or keeps the stored offset along the machine axis.
  With the second rule, an X deviation `d` also moves Y by `d tan(R)`.
  Nothing here was run on a control. The Prime Directive permits a
  visible refusal and does not permit an unproved offset.
- A rotation about Z does not change Z. A face result (Z only) stays
  unguarded, so that a rotated system can still get its Z from a probe.

**What was considered and dropped.**

- Guard only case 1 (`[#5230 NE #5250]`), and trust case 2. Dropped: case 2
  depends on a control rule that the public reference does not state.
- Copy the rotation to the target (`G10 L2 P2 R[#5230]`) before the write.
  Dropped: it is a new function, it changes a register that the operator
  set, and it still depends on the rule of case 2.
- Refuse at post time. Not possible: the post does not know the rotation.
- Test before the first probe move. Dropped as not necessary: the probe
  moves and the trip values are in the rotated frame and are correct. Only
  the write is in doubt. The stop comes after the deviation reports, so the
  operator still sees the measured values.
- A LinuxCNC controller model with rotation in the tests. Dropped here:
  #2682 owns the model, and a model that we write cannot prove the rule of
  a control.

**Cost to change later.** If a run on LinuxCNC or PathPilot shows that
`G10 L20` keeps the rotated-frame coordinate of an axis without a word,
the guard can become "the two rotations differ" (`[#5230 NE #5250]`, and
no test when the target is the section's system). That is one expression
in each post, the five tests of `probe_rotated_wcs.rs` and the six
goldens that this PR moved.

**Workaround for the operator.** Clear the rotation (`G10 L2 P<n> R0`),
probe, then set the rotation again. Or probe with a measure-only operation
and read `#190`.. .
