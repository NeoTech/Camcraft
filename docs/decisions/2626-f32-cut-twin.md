## #2626 - GPU stock sim S1: the f32 cut twin and the brick work list

**What was decided.**

1. The twin sets the f32 arithmetic that the S2 WGSL kernel must copy.
   No WGSL exists yet, so the twin follows the f64 order of
   `field_sweep::Sweep::dist`: the capsule closed form for the ball; for
   the upright flat, the level closed form, the plunge closed form and a
   32-step golden section on every other move.
2. The tile is the brick (8 cells a side). The CPU subtracts the brick
   origin in f64 and rounds once to f32. A cell center is
   `(local + 0.5) * cell` in f32.
3. The twin adds a margin to each distance before it claims:
   `TWIN_ULPS = 32` f32 epsilons of the tile scale (reach + r + flank +
   8 cells), plus `1e-6` of the path length on the flat path (the f64
   golden-section bracket). The f32 cut is therefore never above the
   f64 cut. The parity test checks this with no tolerance, because the
   f64-to-f32 pack rounding keeps order.
4. The max law runs on the order-preserving key (flip all bits of a
   negative, set the sign bit of a positive). The GPU can then use an
   unsigned atomic max.
5. The work list uses the f64 window of each chord (`Sweep::window`),
   clipped to the lattice box of `pack_cut`. It is made in f64 on the
   CPU. The items are sorted brick-major, then by chord.
6. The twin refuses, by name, every tool other than the ball and the
   flat end mill, an arc move, a tilted axis (also for the ball) and a
   zero-length feed move. The f64 cut accepts a zero-length move as a
   station; the issue asks the twin to refuse it.
7. The twin writes the cut layer only. The ray-entry layers
   (`cut_ups`, `cut_downs`) stay out: the issue makes S1 a display
   reference and S2 writes only the cut layer.
8. The parity test compares values after a clamp at `-cap` on the two
   sides. A cut under `-cap` never shows (the mesh layer is at least
   `-cap`), and a claim that sits at the `cap` threshold on one side
   only must compare by its visible value.

**Why.** The issue asks for one-sided parity ("never more removal") and
for a refusal by name. A margin in the kernel gives the one-sided bound
by construction. A tolerance only in the test would not.

**Dropped.** A ball on a tilted axis (easy, but the issue names upright
tools); a direct copy of `flat_tie` (the twin is conservative on the
tie, and the f64 tie only claims more); a public change to the f64
kernel (the twin reads the f64 window through the crate-internal API).

**Cost to change later.** If S2 needs a different f32 order, change
`cut_twin.rs` and keep the parity tests; the margin then follows the
new error bound.
