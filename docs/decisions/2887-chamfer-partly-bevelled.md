## #2887 - A no-pick 2D Chamfer refuses a top loop that is bevelled on only some sides

**What was decided.** A 2D Chamfer with no picks refuses at generation when a top edge loop carries a modelled chamfer on only part of its length. The refusal names a point on the bevelled part and a point on the sharp part. It asks the user to pick the edges. The check is `partly_bevelled` in `crates/motion/src/machining2d/chamfer_ride.rs`, called from `ride_loops`.

The check runs per segment of the top loop. It measures both ends and the middle of each segment against the wall loop at the corner depth:

- A segment on the wall (within 1e-6 mm) is sharp.
- A segment farther than 0.05 mm from the wall is bevelled.
- Between the two is tessellation noise. It counts as neither.

The loop is refused when it has a bevelled segment and more than one chamfer width of sharp length.

**Why.** The issue allows two results. The op can ride only the bevelled segments. Or it can refuse by name and ask for picks. The refusal is the smaller change. It adds no new toolpath shape. Before this fix, the verifier already refused the operation as a gouge. So no program that posted before is refused now. The user gets the reason and the workaround in place of a gouge coordinate.

The per-segment check does not depend on tessellation. An extra collinear vertex on the bevel's top edge changes nothing.

**Considered and dropped.**

- An open (partial) ride along only the bevelled segments. It needs open-path entry and exit moves on a chamfer ride. It needs edge-break credit for part of a loop in the verifier, and new goldens. It is a feature, not this defect fix.
- A two-sided Hausdorff gap for the whole loop. It makes the classification independent of tessellation. But it still classifies the loop whole, so the sharp sides are still cut.
- A change to the verifier (`models_a_bevel`, the edge-break credit). The issue says the verifier's refusal of cuts into sharp edges must not change.
- A separate note that lists the edges left sharp. The refusal message names the sharp part of the loop. A note for whole loops left sharp (#2596) needs a warning channel from `ride_loops` to the operation. That is separate work.

**Unchanged.** An all-sharp part breaks every top edge (#303). A loop bevelled all round rides its wall (#2435). A sharp loop beside a bevelled loop is left alone (#2596).

**Cost to change later.** Low. The rule is one function in `chamfer_ride.rs` and one call in `ride_loops`. The two tolerances (1e-6 mm and 0.05 mm) and the one-width rule are constants in that function.
