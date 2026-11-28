## #4542 - A made pcurve is mapped by its own knot domain

**Decided.** `on_our_interval` (`crates/kernel/src/solidcore_trims/checks.rs`)
reads the knot domain of the pcurve that the kernel made and maps that domain
onto our edge interval. It no longer uses the edge interval that the kernel
reports. The reach check stays: each end of the domain must be within the
vertex tolerance of its end along the edge curve. A pcurve with no knot
domain is refused by name.

**Why.** A Fusion inch STEP had an ellipse edge on a cylinder. The kernel
reported the edge interval `[0, 0.5000000000001025]`, equal to ours, but its
pcurve's knots ended at `0.5`. The old check kept the pcurve as made. The
lift check's last sample then fell 1e-13 past the knot domain, past the
kernel's rounding allowance (about 1.8e-15), and the whole solid was refused.
The pcurve's knots are what `eval` reads, so they are the domain to map.

**Dropped.**
- Clamp the lift samples into the pcurve domain. This hides only the last
  sample and leaves a pcurve that a later reader evaluates off its end.
- Widen the end allowance of `find_span` in solidcore. That is an upstream
  change, and it changes every evaluation in the kernel.
- Raise the fixed vertex gap limit for the same file. That is a separate
  check with an open maintainer decision (#3252). The readings of this file
  are in a comment there.

**Cost to change.** Local to `on_our_interval` and its one caller. The unit
tests in `checks.rs` pin the mapped domain and the refusal past the reach.
