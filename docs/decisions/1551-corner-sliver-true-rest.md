## #1551 - The carried corner slivers after an outside contour are true rest

**What was decided.** No sim or carry change. The full-height slivers at
the stock's outer corners (50 x 50 part, 60 x 60 stock, Setup1 contour with
`EM-0600-4FL`, carried into a flipped Setup2) are material the contour
never touches. The PR adds a pinning suite, `crates/app/tests/carried_corner_rest.rs`,
that checks the carried body against the swept-disc geometry column by
column, and closes the issue.

**Why.** The issue's analysis read `EM-0600-4FL` as a 0.6 in (15.24 mm)
end mill. It is a 6 mm end mill. The contour runs on the part's offset at
r = 3 mm with round corners (arcs about each part corner), so it sweeps
every point within 2 r = 6 mm of a part corner. The stock corner is
5 * sqrt(2) = 7.07 mm from the part corner. The region of each 5 x 5 corner
square outside the 6 mm quarter disc (legs 5 - sqrt(11) = 1.68 mm, area
1.227 mm^2) stays at full height. The probe found 40 full-height brim
columns (10 per corner) in the carried body, all 6.02 to 6.81 mm from the
part corner, and none inside the swept disc. With `EM-0800-4FL` (reach
8 mm) the carried body keeps no full-height brim column.

**Considered and dropped.**
- A sim or carry fix: there is no gap to fix; a change would remove true
  rest material from the carried body (a fixture check against less
  material than the machine holds: gouge-class).
- An extended (sharp) corner on the contour: that is a strategy choice of
  the contour, not this defect, and it would change posted G-code. A user
  who wants the corners cleared picks a larger tool or a second pass.
- Changing #1549's pins back to 60 x 60 stock: they test fixture shape,
  and the 56 x 56 stock keeps them clear of the sliver on purpose.

**Cost to change later.** None; the suite is test-only.
