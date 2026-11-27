## #2341 - Test corner blends and solid restarts in one pocket

**Decided:** Add one test to the existing `adaptive_entry_pieces` binary.
Use Offset rings with a 30 mm square and a separate 10.1 mm square.
Request a helix, enable corner blends, and cut at Z -4 and -8 mm.
Compare the result with the same pocket with blends disabled.

**Why:** The large piece supplies planar corner arcs and helix entries.
The small piece fits the 10 mm tool, but cannot fit the helix or a ramp.
The test checks added planar feed arcs at each level, not just an arc count.
It also checks finite chained motion, the large piece's helices, and unchanged solid plunge sites and depths.
Each solid plunge must have a diagnostic that names its XY site and Z level.
The coverage passes on unchanged production code, as expected for this issue.

**Considered and dropped:** Separate blend and restart tests do not exercise their shared emission path.
A new top-level test binary adds no useful isolation and increases build costs.
The existing restart binary supplies the fixture and motion-reading helpers without a new binary or shared helper change.
The existing directory suites cover other domains or have active overlapping work.
Changing production code to obtain a failing baseline would invent a defect and exceed this test-only scope.

**Verification:** Disabling blends temporarily fails the added planar arc assertion at Z -4.
Removing `with_solid_restart()` separately fails the named restart assertion at (55.100, 5.000), Z -4.
Both mutations are restored before the final test and lint runs.
The touched binary and strict motion clippy supply the documented direct quick-gate alternative.
The release light lane is not run: the task excludes release builds for this test-only change.
This conflicts with the general review policy that requests the light lane; no pass is claimed.
No machine safety certification follows from these software tests.

**Cost to change later:** Only this test and its fixture readings need to change.
There is no production, format, dependency, or public API change.
