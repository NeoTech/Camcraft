## #3841 - Keep the source move on an AC travel refusal

**Decided.** Add the zero-based input move index at the path solver boundary.
Keep the original travel error as the cause, including its axis, required
angle, limits, and hint. Standalone pose solves keep their existing error.

**Why.** The original index lets a post identify the input move even after
the solver subdivides earlier moves. The post can use the typed cause to
identify the travel constraint without parsing a message. In a future
post-driven retry, a post could revise the identified input and ask the
engine to solve and validate the new path. For example, it could consider an
opposite tilt solution when both signs fit the machine and a C rotation can
be reduced. The engine still owns reachability and continuity checks.

**Dropped.** A message-only index would require parsing for a retry.
Automatic sign switching would change which rotary solutions the solver
selects. That requires its own design and tests. This change reports the
first travel refusal. Reports for large rotations within travel require
their own diagnostic.
