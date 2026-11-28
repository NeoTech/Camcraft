## #4032 - Rotary Flow refuses a crossing between any two guides

**What was decided.** A new law, `rotary_flow_cross::refuse_crossed_set`,
reads every pair of guide chains that are not neighbours in the pick
(`j >= i + 2`). It runs once over the complete set, after the set
normalization and before the first band is cut. A meet or a crossing
within the corner-lattice dust is refused by name. The refusal names the
two guides and the crossing point, in the text and as the error location.
The exact test is the existing Ericson 5.1 segment-pair solve
(`rotary_flow_set::seg_seg_at`). An axis-aligned box prefilter, padded by
the dust, skips each chain pair and each segment pair that cannot come
within the dust.

**Why.** The kernel blends only adjacent pairs, and the per-pair meet law
in the band loop read only those pairs. A set where guide 3 crosses
guide 1 machined bands that fold through the same ground, with no
diagnostic (Prime Directive: a crossing that is machined is a wrong
path).

**Considered and dropped.**
- Move the adjacent check into the new set-wide pass too. Dropped: the
  adjacent refusals then fire before the band loop's width and backtrack
  refusals, and the refusal text of an already-refused adjacent set can
  change. The adjacent law stays in the band loop, unchanged.
- A prefilter on the cylindrical `(theta, Z)` spans. Dropped: theta wraps
  and the unwrap aligns pairs by whole turns, so a theta interval is not a
  sound bound across the set. The world-space box is sound and simple.
- A grow of `rotary_flow_set.rs` (already past 400 lines). Dropped for a
  new small module.

**Cost to change later.** Low. The law is one call in `rotary_flow` and
one module. A set with a non-adjacent crossing AND an adjacent defect now
reports the non-adjacent crossing first; each such set was already refused
or wrong.
