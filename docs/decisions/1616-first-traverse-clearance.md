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

This PR does not close #1616. It does the work for grblHAL, TinyG and
the Carvera at a program start. Two parts of #1616's "Done means" stay
open: the Fanuc-family posts (fanuc, haas, mazak, doosan-dnm, sinumerik,
heidenhain, #1633), and the Carvera right after `M6` (below). The PR body
says `Refs #1616`.

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
The clearance Z is still stated before any descent.

**The #1614 hazard stays on the Carvera after `M6`.** The bare X/Y
traverse right after `M6` is made at the changer's exit height. Nothing
we have read confirms that height. A long tool can leave the tip below
the part top there, and the traverse then crosses the part. Option (c)
does not remove this hazard; it only does not add the new one (a
downward rapid over the rack). The hazard stays open until a public
source confirms the firmware's `M6` exit position (option b). The
comment in `carvera.cps` says so, and the pin
`the_carvera_script_names_the_open_hazard_after_its_tool_change` keeps
that note in the script.

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

**Review of a1833e52 (takeover).** Two changes were requested.
- The Carvera claim: now stated plainly as above. Considered and dropped:
  refuse every Carvera program with a tool change until option (b) is
  done. That stops all Carvera multi-tool work, and the review did not
  ask for it.
- The closing keyword: two answers are safe. (1) Keep the keyword out
  (`Refs #1616`). (2) Narrow #1616 and file the two remainders as new
  issues before it closes. Taken: (1). It changes no issue, and #1616
  stays open as the record of the work still owed, so nothing is lost
  if a remainder is not filed. Note: commit 49ec197f says `Refs #1616`,
  but commit a1833e52 ends `Fixes #1616`. That commit is already pushed
  and is not rewritten (no amend, no force-push). GitHub can close #1616
  from that commit message when the PR merges. If it does, reopen #1616.
