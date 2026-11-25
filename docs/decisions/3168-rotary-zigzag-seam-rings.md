## #3168 - Rotary Offset Zigzag levels close with a ring that skips the seam

**What was decided.** The Zigzag/Auto level plan now emits its #3129 wall
rings on the rotary wrap too. A boundary loop that spans a whole period
wraps the seam: its two cover-end edges are the seam's two copies at one
azimuth, so the ring drops both edges and keeps one period of boundary
per leftover arc. The arcs arrive open - each ends on the seam's copy of
its start azimuth, closed on the work - and the rows cut the seam
columns. A loop that wraps no azimuth closes exactly as the planar
#3129 ring does.

**Why.** The #3129 plan suppressed the rings on the wrap whole, because
a closed ring there would run down both cover ends and cut the seam
azimuth twice. That left every wall perpendicular to the rows (a
D-flat's edge walls) with cusps between row ends. Dropping only the seam
edges removes the double cut while the ring still finishes the walls.

**Considered and dropped.** Closing a wrapped loop's ring across the
seam through the +period duplicate: the cover data cannot close a
full-period ring without a chord one full period long across the cover,
which the link law would refuse or the wrap would cut backwards. Emitting
no ring on the wrap (the old behaviour) is the cusp defect this fix
removes.

**Cost to change later.** The seam law lives in
`machining2d::pocket_zigzag::ringed_passes` behind one `Option<f64>`
(the cover's period). A future cover that is not an exact duplicate per
period would need the trim to follow the work's own symmetry instead.
