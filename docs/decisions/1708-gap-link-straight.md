## #1708 - the finish3d gap link: a straight line across gap openings

**What was decided.** A `gap_link` parameter on the Finish3D parameters,
a dropdown with two options:

- `Jump` (default): the existing retract/clearance hop, byte for byte as
  every older file wrote it.
- `Straight`: one straight Feed link from the end of one run to the start
  of the next, at the two end points' own tip height - no rise, no
  traverse, no plunge.

The law for `Straight` (new file `crates/motion/src/finishing3d/gaplink.rs`):
both run ends stand at tangent centers by the march's own construction, so
the line is fixed and nothing is chosen for it. The exact segment-triangle
sweep of the move at the budget `r - TOUCH_EPS` answers the whole question:
a grazing contact at the rim passes (tangency is the law's own currency),
a real bite refuses. A refused link takes the `Jump` hop. `ramp`'s
certified diagonal still takes the break first where it can be struck.

The same commit fixes the raster serpentine for a line broken by openings:
a back line now reverses WHOLE (`pieces.reverse()` plus each piece's points).
Reversing only the points kept the pieces in grid order, so the chain ran
one piece backwards and then jumped back across the whole field to the next
piece's far end. Under the plain hop that mis-pairing hid inside a high
rapid; a straight link authors the same line at the cut, where it shows.

**Naming and the stored word.** The maintainer's verdict after a GUI
try-out (chat, 2026-11-14): keep only `Jump` and `Straight` in this PR.
An intermediate build had offered three options (`Lift`, `Bridge` - the
lowest facet-certified traverse height - and `Straight`); `Bridge` is
dropped, and the default option's word is `Jump`, not `Lift`. The project
digest takes tag `0x4D` plus a variant byte only when the choice is not
`Jump`, so every op that hops as before keeps the stamp it had. The file
words are `jump` and `straight`, written always, like `ramp` - new words
in an existing grammar, no version step-up.

**Why.** On the maintainer's part (a 171 mm STEP model with many holes)
every hole broke the raster line and each break lifted to the retract
plane. The moves looked wrong in the GUI and the air hops cost cycle time.
A straight rim-to-rim link at cut height is the smoothest answer; the
sweep verdict keeps the Prime Directive - only a certified non-cutting
line is authored, anything else falls back to the hop.

**Considered and dropped.**

- `Bridge` (lowest certified traverse) in the shipped dropdown: the
  maintainer asked for the two-option surface only; the code for it was
  not carried over. A later issue can re-add it under its own law.
- Auto-lowest-hop with no parameter: the maintainer chose the explicit
  dropdown; the default must write the old bytes.
- Land-model lid work (an earlier experiment to close openings for the
  march): dropped, it did not work reliably.
- Splines or curves for the link path: the named future direction
  ("later we can add splines and curves for how to connect them"); a
  future link type under the same certify-or-hop law.

**Cost to change later.** Low to add link types: the ladder in
`emit_runs` is one branch per type, each certify-or-fall.
