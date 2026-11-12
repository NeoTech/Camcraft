## #1305 - The posed fixture sweep reads a band as a flat-ended cylinder

**What was decided.** The posed sweep of fixture laws (c2)/(d2)
(`crates/sim/src/verify/posed_steel.rs`) keeps its capsule test as the
cheap first reading and adds a second one for every box the capsule
reaches: the band as the flat-ended cylinder it is. Per interval the
second enclosure is

    chord p0..p1 (swept exactly)  +  cylinder lo..hi, radius r, about the midpoint pose
                                  +  ball(pad),  pad = bulge + swing * hypot(max(|lo|,|hi|), r)

and, for fixed steel under a rotary, that set carried rigidly by the
midpoint turn and grown by `dth * (rho_max + r + pad)`, `rho_max` the
farthest of the parallelogram's four corners from the axis line. A box
is struck only when BOTH enclosures reach it; it is cleared only on a
GJK proof of a separating plane more than `pad + FIXTURE_HIT_TOL` wide
(`fixture_mesh::band_clear_of_box`, the mesh law's own narrow phase
pointed at a box). Leaning by θ the rim now reaches R sin θ past the
band's end plane instead of R, and the verdict turns at the lean where
the true band touches (pinned to 0.006 degrees on the issue's holder).

**Why it stays conservative.** (1) Every point of the true band at any
instant of the interval is `tip + Q y`: the tip within `bulge` of the
chord (`arc_bulge::deviation`), `y` in the cylinder about the midpoint
pose, `Q` the least rotation onto a pose within `swing` of it
(`midpose`'s proven bound), and a rotation by φ moves a point at most
φ times its distance from the centre (arc >= chord) - so the point lies
in the Minkowski sum above. The capsule's old growth used `|hi|` as the
arm because it reasons about the centreline; a cylinder's rim stands
`hypot(hi, r)` from the tip, so the flat reading takes the longer arm.
(2) The intersection of two sets that each contain the swept band
contains it. (3) "Clear" is only ever the supporting-hyperplane bound
`v.w / |v| > margin` over exact support mappings; simplex rounding, a
degenerate simplex or an exhausted search answer "not proven", and the
capsule's hit stands. Non-finite numbers anywhere are never a proof.
The property test places a speck of steel at 4800 sampled points of the
true band (rims and end faces weighted) on moves whose pose swings, in
the path frame and under a rotary: every one is struck. Dropping the
swing term from `pad` fails that test.

**Sources.** Gilbert, Johnson, Keerthi, "A fast procedure for computing
the distance between complex objects in three-dimensional space", IEEE
J. Robotics and Automation 4(2), 1988; van den Bergen, "A Fast and
Robust GJK Implementation for Collision Detection of Convex Objects",
J. Graphics Tools 4(2), 1999 (the lower-bound termination that is the
proof); Ericson, "Real-Time Collision Detection", 2005 (support
mappings of a cylinder and a box, closest point on a simplex, swept
enclosures by bounded rotation). The enclosure itself is the one
`verify/mesh_posed.rs` already proves for the fixture mesh law (#339
slice 3) - the box law now reads the same geometry. No vendor material.

**Considered and dropped.**
- *Replace the capsule outright.* Kept as the broad phase: it is a
  ternary search with a box reject, GJK runs only where it hits, and
  every verdict that was clear stays clear by the same arithmetic. No
  golden moved.
- *Capsule ∩ end-plane slab* (two half-spaces at the band ends). Sound
  and tiny, but not the cylinder: it over-reports a box diagonal to the
  rim by up to √2 of the gap and measures the slab against the whole
  box, so a long clamp is convicted by a lean of gap/length. A smaller
  cliff, not none.
- *A closed-form cylinder-box distance by feature cases*, or nested
  ternary search of the (convex) point-cylinder distance over the box.
  The first is a new case analysis to get wrong in a collision law; the
  second costs ~10^5 evaluations per test. The repo already carries a
  GJK whose answer is a proof.
- *Keep the tip's travel as a ball* (the capsule's `travel` growth,
  up to a quarter millimetre). The swept band takes the chord exactly,
  so only the swing and the bulge ride the margin.

**Not in this PR.**
- `main` has neither #938 (`UPRIGHT_TILT_RAD`, PR #973, open) nor #1303
  (open, no branch pushed), so this was built on `upstream/main` alone.
  In the PATH frame law (c2) also runs its own column enclosure
  (`stack_span::band_struck`), which still drops a leaning band by its
  whole radius - #1303. Until that merges, `bands_vs_fixtures` on the
  issue's evidence case is still convicted at 0.01 degrees by the
  column, not by the sweep; the sweep's own verdict is pinned at
  `posed_hits`, and end to end where the sweep alone decides (fixed
  steel under a rotary). PR #973 edits one line of `posed_steel.rs`
  (`is_vertical` -> `is_upright`), away from this change.
- The mesh twin (`mesh_posed.rs`) already sweeps flat-ended bands
  (read in full: no capsule). The part-solid probe (`probe.rs`) reads a
  radius-at-height profile and names no capsule; it was skimmed, not
  audited, and is unchanged.

**Cost to change later.** One predicate (`flat_band_clear`) and one
crate-private entry (`band_clear_of_box`); removing the `&&` restores
the capsule law exactly.
