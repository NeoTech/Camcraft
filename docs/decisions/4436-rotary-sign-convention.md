## #4436 - The posted rotary sign and the travel check agree

**Decision.** Fanuc and Doosan DNM write the opposite angle for a rotary
that states `table_rh`. They keep the angle for `tool_rh`. The existing
machine mirror resolves each axis's convention, then the machine's
convention, then the `tool_rh` default. The post applies that resolved
convention once, when it writes the angle word. Modal angle memory and
the feed cap keep their current angle representation.

The machine check applies the same resolved convention before it compares
an angle with travel. The lone-X check applies the convention after the
stated axis direction. The existing refusal for a reversed lone-X rotary
remains in both G43.4 posts. Its text now distinguishes the axis direction
from the sign convention. The separate explicit `Move::rotary` C-angle
contract does not change.

**Why.** The machine schema defines `tool_rh` as a tool rotation relative
to the work and `table_rh` as a work rotation about the stated axis. Those
rotations have opposite signs. The old posts ignored the convention, and
the check compared the wrong signed angle with asymmetric travel. An
original user-machine fixture pins inherited and per-axis conventions,
initial and modal A/B/C words, A/B pose checks, and lone-X direction.
The control test keeps the unstated default byte-identical to `tool_rh`.
The Doosan fixture explicitly declares its M10/M11 brake codes.

The shared lone-X check also serves Tormach. Its existing post guard
checks its own angle word against travel before it writes that word.
A pin supplies a `table_rh` machine with travel -25..-15 degrees. The
generic check accepts the signed -20, and Tormach refuses its own +20
by name before any A20 command. This adds no Tormach convention support.

**Alternatives dropped.** Refusing every `table_rh` machine would prevent
wrong output, but would reject a convention that the schema already
supports. A new sign framework or dependency would add no needed
behavior. Mach4 keeps its own inverse-kinematics representation and sign
handling. No Mach4 code changes here.

**Related work.** PR #4371 at `ce23fdaa8d18ae3784604f2ad1198316cdf7277d`
changes the retract and section-preamble functions of these post files.
It does not change `angleWord`, `tilt`, or the machine pose check. This
change does not adopt its separate retract work.

The #4417 branch at `2ede3d05710529d59bb1e0e88761977de23ea057`
converts pose angles to joint angles before `angleWord` and before the
machine check's per-axis loop. It leaves `angleWord` unchanged. This sign
conversion belongs after that calculation and does not duplicate its
chain solver. The two issues retain their separate purposes.

**Cost to change later.** A change to the machine sign contract must update
the two word writers, the pose checks and these regression assertions
together. There is no file-format change or new dependency.

**Sources.** The project's `RotarySignConvention` schema and the existing
machine mirror contract; issue #4436. No proprietary implementation or
vendor post text was consulted.
