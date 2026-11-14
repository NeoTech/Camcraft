## #1616 - grblHAL, TinyG and Carvera state the clearance Z before the first X/Y traverse

**Decided.** `grblhal.cps`, `tinyg.cps` and `carvera.cps` take the #1614
pattern (PR #1618, `tormach.cps`). The first X/Y traverse after a tool
change, and at the program's start, has its height stated: a Z-only `G0 Z`
to the section's `ClearancePlane`, else its `RetractPlane`, never below the
move's own Z.

A stream that states neither plane is refused by name, with #1618's
wording: "first X/Y traverse with no clearance height stated (no
ClearancePlane or RetractPlane) - the post will not rapid X/Y at an unknown
height". The operation name is put in front where the post knows it.

When a feed is the first move, its rapid down to the start height is
skipped if the traverse already went to that height. TinyG always wrote
that rapid; it now writes it only when the height differs.

This PR closes #1616 for these three posts. The Fanuc-family half (fanuc,
haas, mazak, doosan-dnm, sinumerik, heidenhain) is #1633.

**Carvera: M3's option (c).** The order of the two traverse lines depends
on where the spindle stands:
- **Right after `M6 T..`:** the Carvera keeps main's order. The bare
  `G0 X.. Y..` comes first, at the height the changer leaves the spindle.
  Then comes `G0 Z<ClearancePlane|RetractPlane>` (never below the move's
  Z), and only then the descent.
- **At a program start with no tool change:** Z first, as on grblHAL and
  TinyG.

Why: nothing we have read says where the Carvera firmware parks X/Y after
its automatic tool change. A Z-only rapid at that X/Y down to a thin
stock's work clearance could come down onto the tools stored in the rack.
Traversing X/Y at the changer's exit height is what main always did, so
this PR adds no new hazard. The clearance Z is still stated before any
descent.

Options M3 laid out in the review of 49ec197f:
- (a) Accept Z-first after `M6` too. Dropped: it is a new downward rapid
  at an undocumented X/Y.
- (b) Confirm from the Carvera documentation where `M6` leaves the
  spindle, and say so in the post. Dropped for now: no such source has
  been read. If one is found, the Carvera can go back to Z-first.
- (c) Keep X/Y-first after `M6` for the Carvera only. **Taken.**

grblHAL and TinyG are unchanged: they lift Z themselves before the change
and a manual change does not move a rack under the spindle.

**Why Z first everywhere else.** The lift before a change (`G53 G0
Z<safe_retract>` or the work-offset clearance) does not prove clearance. A
long tool on a high work offset, or an operator jog at TinyG's M0, can
leave the tip below the part top. Stating the height means a setup that
cannot clear stops at the control's Z limit instead of rapiding through
the part. The wording and shape match #1618 so the posts read alike.

**Also considered and dropped.**
- Adding the rule to `assets/posts/AGENTS.md` here. #1618 adds the "first
  traverse states its height" bullet at the same spot, so editing it here
  would conflict. The bullet's list of posts is widened in #1633.
- Skipping the extra `G0 Z` when the lift before the change was already
  the same work-offset height (grblHAL and TinyG without `safe_retract`).
  Kept, because the tool change itself (a manual change, a tool setter, a
  jog at M0) may move Z, and one redundant line costs nothing.

**Cost to change.** One function per post (`xyFirst`/`traverseZ`), plus
the Carvera's `changedAt` flag; the pinning suite
`crates/post/tests/first_traverse_clearance.rs`; and the goldens that
gained the one line.
