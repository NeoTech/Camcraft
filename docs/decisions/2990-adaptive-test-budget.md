## #2990 - Split adaptive trochoid targets and reduce fixture work

**What was decided.** Move six public shape tests into three independent
integration targets. Use smaller fixtures and a coarser debug measurement
grid. Keep all 22 tests, the one existing ignored test, and all 61 original
assertion lines.

- `adaptive_troch_parallel` holds the wide-pocket and corner-cleanup tests.
- `adaptive_troch_island` holds the island and ring tests.
- `adaptive_troch_neck` holds the triangle-and-neck test and the ignored
  L-shape test.

Shared helpers in `tests/common/adaptive_troch.rs` use public APIs.
The remaining 16 tests stay in the original unit module, which has
388 lines. Private tests keep access to private helpers there. No
production API, dependency, or gate routing changes are needed. Cargo
discovers the new targets. The batch scheduler can run them independently.

The D6 rectangle is 30 x 24 mm. The D10 rectangle is 44 x 36 mm. Both
retain enough width to exercise parallel lanes. The D6 island uses a
30 x 30 mm outer boundary and a 6 x 6 mm hole. The D10 island uses a
48 x 40 mm outer boundary and a 12 x 12 mm hole.

The ring starts with a 12 x 10 mm rectangle. Its offsets remain 12 mm
and 2 mm, so the ring keeps its 10 mm band. The triangle keeps its
original 60 x 60 mm dimensions. The neck keeps its original 30 x 30 mm
wide lobe and 10 mm neck width. Only its narrow end is shorter: its final
X coordinate changes from 60 mm to 48 mm. The explicit-parameter test
uses a 30 x 16 mm face and keeps its rapid-count assertion.

Use `QualitySpec.cell = D / 60` in debug runs of `check_cleared`, as in
the existing project pattern. Release runs keep the default `D / 80`
cell size. Both modes run all geometry assertions. The existing ignored
L-shape test moves without a fixture change. Its release run keeps
the `D / 80` cell size.

Add a 90-second execution budget to the existing wide-rectangle test in
the `adaptive_troch_parallel` target.
Apply this assertion in every run. The initial 60-second budget failed
at 61.93 seconds in a concurrent suite run. The 90-second budget permits
contention. The measurement excludes compilation. The unchanged fixtures
took 178.44 seconds. A mutation that restored those fixtures took
167.53 seconds and failed the 90-second budget.

**Why.** Smaller fixtures and the debug cell size reduce repeated
geometric work. Independent targets let the scheduler distribute that
work. The tests still check their behavior in the batch gate. Release
runs retain the finer measurement. Each test file stays below about
400 lines.

The local motion lib baseline took 207.11 seconds. Fixture changes alone
reduced it to 191.12 seconds, a reduction of about 7.7 percent. This was
not enough to address the slow target. Add the target split rather than
rely on smaller fixtures alone.

**Considered and dropped.** Moving these tests to the heavy lane would
remove their checks from the batch gate. The proposed light tier in
PR #2787 has not merged. Moving all seven reported slow tests would need
access to private helpers. Move the public shape tests instead. Keep the
private tests in the unit module without exposing more production APIs.

The smaller 38 x 33 mm D10 rectangle failed the existing rest-area
assertion: 31.03 mm2 exceeded 21.50 mm2 plus the 8.27 mm2 tolerance.
Enlarge the fixture to 44 x 36 mm and keep that assertion.

Smaller D10 island fixtures failed the existing checks. A 36 x 36 mm
outer boundary with a 6 x 6 mm hole failed the rest comparison.
A 54 x 36 mm outer boundary with a 10 x 10 mm hole measured about
128 degrees of engagement at `D / 60` and failed the engagement check.
Keep the 48 x 40 mm outer boundary and the 12 x 12 mm hole. Do not
relax either assertion.

The 30 x 30 mm and 45 x 45 mm triangles failed the existing rest
comparison. Keep the original 60 x 60 mm triangle. The neck fixture with
a 24 mm wide lobe also failed the rest comparison. Keep its original
30 x 30 mm lobe and shorten only the narrow end.

**Cost to change later.** Restore or enlarge the fixtures if a defect
needs a larger case. Change the execution budget or debug cell size only
with measured evidence. A different target split needs only test moves
and helper imports. Keep the assertions and test count when changing
the fixtures or target layout.

## PR #3032 - Refresh the reached Linux bore pin

**What was decided.** Pin the Linux x86_64 bore path to 3158 moves,
3130 planar feeds and 1141 arcs. Require the imported mesh to contain
6228 facets. Require the four cutting levels at Z 0, 5, 10 and 15.
Add a quick test that rejects the old tuple and adjacent count tuples.
Keep the existing verification, fallback and planning-budget checks.

**Why.** The review found a stale baseline in the reached release heavy
lane. PR #2904 (`bac4a96c7`) replaced the STEP importer. Decision #917
records that `Milling-test.step` changed from 952 truck facets to 6228
solidcore facets, with measured deviation within the requested tolerance.
The changed mesh affects the cutting regions and their arc fits.
The review measured the new tuple on both this PR and upstream main.
The mesh and cutting-level assertions bind the count to that input and
require every intended level to remain present.

**Considered and dropped.** Do not widen the count range, ignore the
failure, or move the test out of its lane. Do not change production code
to recreate the old truck-mesh count. Keep the safety assertions and
measure the corrected pin in the release heavy lane.

**Cost to change later.** A later mesh or planner change must explain
and measure a new tuple. Keep the mesh, cutting-level and safety checks
when the count changes.
