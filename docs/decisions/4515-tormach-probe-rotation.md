## #4515 - Refuse Tormach XY probe writes from extended work offsets

**Decision.** Fail the post by name when an XY probe result would write a
work offset from an active `G54.1 P10` through `G54.1 P500`. Keep the
rotation checks for systems 1–9. Keep Z-only and measure-only probing.

**Why.** The public [PathPilot programming manual][programming] documents
extended work offsets, but the [numbered parameter reference][parameters]
documents work-offset rotation only through `#5390` for `G59.3`. The next
value in the old formula, `#5410`, is the current tool diameter. An XY
write needs a rotation check for both the active and target systems.
The result stream limits the target to `G54` through `G59`. Rotation
about Z does not change a Z-only result.

**Alternatives.** Extrapolating the parameter formula reads unrelated data.
Skipping the rotation check can write a wrong origin. A counted warning
alone can still leave a standalone CLI program that omits the requested
write and continues. A post error prevents that program from being saved.

**Cost to change.** If Tormach documents an extended rotation parameter,
replace the refusal with the documented lookup. Update the post tests to
pin the parameter and guard before `G10 L20`.

[programming]: https://knowledgebase.tormach.com/1100m/programming-1100m
[parameters]: https://tormach.com/machine-codes/numbered-parameter-reference
