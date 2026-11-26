## #3085 - Match the current bore counts on unmeasured platforms

**Decided:** Center the test-only fallback band on `(4688, 4660, 1980)`.
Keep the previous half-widths: 15 moves, 15 planar feeds and 10 arcs.
The inclusive ranges are `4673..=4703`, `4645..=4675` and `1970..=1990`.
Extract a pure matcher so portable unit tests exercise the fallback on Linux.
Keep the exact platform pins, mesh assertion and safety assertions unchanged.

**Why:** Commit `2407d9b09` records the measurements after PR #3802:
5232 facets and `(4688, 4660, 1980)` on Linux x86_64, with the same counts
on Windows x86_64. The old band still describes the truck mesh.
The issue's proposed `(3158, 3130, 1141)` predates this solidcore bump.
The existing narrow widths retain the portability allowance without accepting
either previous mesh generation. They are not new platform measurements.

**Considered and dropped:** Do not restore the issue's outdated triple.
Do not widen the exact pins or the fallback widths. Do not change the planner
or its safety checks. Use the issue's unit-test acceptance alternative instead
of running heavy generation or claiming a measurement on a fourth platform.
The macOS pin still needs measurement after PR #3802, as its existing comment says.

**Cost to change later:** Measure an unpinned platform and add its exact triple
and regression test. A later mesh change must justify new count expectations.
The focused tests check the current pin, each inclusive boundary, each adjacent
outside count, and both truck-mesh readings plus the previous solidcore reading.
