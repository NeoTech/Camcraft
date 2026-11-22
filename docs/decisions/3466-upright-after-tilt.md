## #3466 - An upright section after a tilted section states the return of the rotaries

**What was decided.** The Fanuc and Doosan DNM posts command the rotaries
back to zero at the start of each section that follows a section which
left a rotary away from zero. The new function `uprightAfterTilt()` runs
in `startPreamble`, directly after the `G90 ... G49 G80` line. It:

1. reads the angles that the control holds (`rotAt`, the memory of #885);
2. if one of A, B or C is not zero, retracts to the reference height
   (`retractHome(false)`: `G91 G28 Z0` then `G90`, or `G53 G00 Z` with the
   `safeRetract` option);
3. writes `G00` with a zero word for each held rotary (Doosan: `M11` before
   and `M10` after when A turns, as `onRapid` does);
4. sets `xyUnknown`, so the first X/Y traverse states its height again.

This occurs before the tool change, so a tool change and the same tool get
the same return.

**Why.** The angle words are modal. A tilted section ends with its exit
rapid at the lean, and `endPreamble` reset only the script's own memory
(`lastA/B/C`), not the control. A 3-axis section states no angles, so it
cut under the old lean. Reading `rotAt` (the angles really written) gives
a return only when a rotary stands away from zero. An upright program is
byte for byte as before, and no golden moved.

**Considered and dropped.**

- *A fix in `post::dump`* (zero angles on the first move of the next
  operation). The dump is per operation and does not know the previous
  section. The scripts already hold the control's angles in `rotAt`.
- *The return only when the next section is upright.* The issue asks that
  each section after a tilted section states the rotary position. A return
  at the reference height before a tilted section is also safe: that
  section then arms G43.4 and turns from a known zero. The review of
  PR #3478 asked for this choice on purpose; it is kept, and now pinned
  by `fanuc_returns_to_zero_between_two_tilted_sections` and
  `doosan_returns_to_zero_between_two_tilted_sections` in
  `crates/post/tests/upright_after_tilt.rs`. The catalog case
  `examples/catalog/multiaxis/indexed` has two tilted sections in a row
  (Back A-90 after Front A90, Tilted A30 B30 after that); its `.nc`
  gained the return blocks and moved in this PR.
- *A return at the end of the tilted section.* A program that ends after
  the tilted section is then also upright, but the end of a section has no
  reference to the next section's clearance, and `onClose` already homes.
  The start of the next section keeps the return next to the move that
  needs it.

**Not done here (follow-up #3477).** Item 3 of the issue: a check in the
app that refuses an upright operation after a tilted operation when the
post does not state the return. This needs a manifest flag in the engine
and a decision on the posts that a user loads. The shipped posts that
write tilted moves (Fanuc, Doosan DNM, Tormach PathPilot, Mach4) state the
return; Fanuc, Doosan and Tormach are pinned by
`crates/post/tests/upright_after_tilt.rs`. Mach4 is not pinned: its AC
trunnion fixture is not in a shared helper.

**Cost to change later.** Low: one function in each script, and one test
file.
