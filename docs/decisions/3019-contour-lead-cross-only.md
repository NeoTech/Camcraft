## #3019 - A seam lead turns only when it crosses its own ring

**Decided.** At the seam of a closed contour ring, the lead-in and the
lead-out keep the walk's own end tangents, except when the full lead
along that tangent crosses a feed move of the ring. The check uses the
verifier's crossing law (`arc_cross::moves_cross_prior`: proper crossing,
a shared end is a touch). Only then is the tangent mirrored across the
other end's tangent. The mirrored lead is used only if it does not cross.
The change is in the new module
`crates/motion/src/machining2d/lead_seam.rs` (`seam_tangents`).
`lead_fit.rs` calls it from `both_fit` and `plan_leads`.

**Why.** The first fix (#3053) mirrored the tangent on each closed ring
that bent to the air side at the seam. That condition is also true at a
sharp corner of a plain rectangle. There it moved the reseated start,
reversed the vertical entry arc and dropped the chord-fan entry (#3111,
reverted in #3114). A gate on the real fault, "this lead crosses its own
ring", changes nothing where no crossing exists. So all other rings keep
their seam, leads, entry and posted G-code.

The issue's crossing does not come from the tabs. The same crossing
occurs with no tabs on the R6 plate (pinned). So the gate is on the
crossing, not on the tab count. The maintainer's reopen comment asked for
"only where a tab needs it". The crossing gate is narrower than "each ring"
and covers the untabbed case of the same fault.

**Dropped.**
- Gate on `tab_count > 0`. It leaves the same crossing on an untabbed
  round-cornered plate.
- #3053's bend-direction law. It changed rings that had no crossing.
- Relax the crossing law for leads. It would hide real lead crossings.
- Shrink a crossing lead. The crossing is next to the seam, so each
  halving still crosses, and the lead would be refused.

**Cost to change.** Local to one function. The check builds the ring's
moves once for each call. A lead that crosses after the mirror too keeps
its tangent, and verify still holds the post (no silent change).
