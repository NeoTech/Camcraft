## #1616 - grblHAL, TinyG and Carvera state the clearance Z before the first X/Y traverse

**Decided.** `grblhal.cps`, `tinyg.cps` and `carvera.cps` take the #1614
(PR #1618, `tormach.cps`) pattern unchanged: before the first X/Y traverse
after a tool change and at the program's start, a Z-only `G0 Z` to the
section's `ClearancePlane`, else its `RetractPlane`, never below the move's
own Z. A stream that states neither plane is refused by name, with #1618's
wording ("first X/Y traverse with no clearance height stated (no
ClearancePlane or RetractPlane) - the post will not rapid X/Y at an unknown
height"), prefixed by the operation name where the post knows it. A feed's
rapid down to its start height is skipped when the traverse already stated
that height (TinyG always wrote it; it now does so only when it differs).

**Why.** The lift before the change (`G53 G0 Z<safe_retract>` or the
work-offset clearance) is not proof of clearance: a long tool on a high
work offset, an operator jog at TinyG's M0, or a changer that leaves the
spindle elsewhere can put the tip below the part top. Stating the height
makes a setup that cannot clear stop at the control's Z limit instead of
rapiding through the part. Same wording and shape as #1618 so the posts
read alike.

**Considered and dropped.**
- Doing the Fanuc family (fanuc, haas, mazak, doosan-dnm, sinumerik,
  heidenhain) in this PR: they have no `xyUnknown` traverse, carry
  G43.4/rotary paths and about 100 goldens and two dozen test files
  between them; split to a second PR so each golden move can be read by
  diff. This PR is `Refs #1616`.
- Adding the rule to `assets/posts/AGENTS.md` here: #1618 adds the
  "first traverse states its height" bullet at the same spot; editing it
  here would conflict. The bullet's post list is widened when the Fanuc
  PR closes #1616.
- Skipping the extra `G0 Z` when the lift before the change was the same
  work-offset height (grblHAL/TinyG without `safe_retract`): kept, because
  the tool change itself (a manual change, a tool setter, a jog at M0) may
  move Z, and one redundant line costs nothing.

**Cost to change.** One function per post (`xyFirst`/`traverseZ`), the
pinning suite `crates/post/tests/first_traverse_clearance.rs`, and the
goldens that gained the one line.
