## #3004 - A tilted plane reads its stock cap only in its own frame

**Decided.** The tilted (3+2) verify branch in `operation/generate.rs`
starts its local stock cap at negative infinity. The cap is the maximum
local height of the projected corners of the stock box and the model box.
The world stock top (`stock_top_z - oz`) is no longer a lower bound.

**Why.** The world top is a height along world Z. On a side plane (A +90 or
A -90), the local cap is the extent of the blank toward the tool (30 on a
60 x 60 x 70 blank). The old start value held the cap at 70. The shank law
then named a shank 11 mm over the side face a `ShankHit`. The first-traverse
law named a clearance 10 mm over the face low. The corner projection is
already an over-report of the true cap (an affine map takes its maximum at
a corner), so a real collision still convicts.

**Dropped.**
- Recheck `ShankHit` against the exact solids in `stock_top_law`. That
  keeps the wrong cap for the other laws (first traverse, rapids), and it
  is a second law where the cause is one wrong start value.
- A fallback to the world top when no box exists. The model box is always
  projected, so the branch cannot run without a box.

**Cost to change.** Low. The change is one start value in one branch.
The conservative box cap for an oblique face is a separate limitation
(see the issue) and is not touched here.
