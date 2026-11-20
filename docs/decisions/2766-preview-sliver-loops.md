## #2766 - The pocket region on a part drops loops of near-zero area

**What was decided.** `minus_standing` (`crates/app/src/operation/area.rs`)
removes each loop whose absolute area is below `ISLAND_AREA_MIN` (1e-4 mm2)
from the difference "enclosed area minus standing part material". The removal
is `Region::without_slivers` (`crates/geometry/src/geo2d/slivers.rs`), a new
method that only filters loops. It does not change a kept loop.

**Why.** The enclosed area and the part section share a wall. The two
tessellations of that wall disagree by about one grid unit (1 um). The
difference then has 3-point loops of 5e-9 to 1.5e-7 mm2. A clockwise loop of
that kind stands outside each boundary, and `Region::triangles` refuses the
whole region ("hole outside any boundary"). Thus the area preview of the
biggest fig8 patch had no fill with the default Containment. `minus_standing`
already uses `ISLAND_AREA_MIN` as the measure of tessellation noise for the
standing material. The same measure now applies to its result.

The region is the one that generation cuts (`selection_region`). Thus the
slivers are removed for generation too, and the outline, the fill and the
toolpath agree.

**Considered and dropped.**

- Make `Region::triangles` ignore a hole without a parent. Dropped: the
  refusal is correct for a real hole, and each other reader of the region
  would still get the slivers.
- Build the result again with `Region::new` from the kept loops. Dropped:
  the constructor normalises each loop again and can refuse a loop that
  touches itself. A filter cannot change or refuse a real loop.
- Put the filter in `Region::difference`. Dropped: that changes each boolean
  in the product and each golden that depends on one. The defect is in this
  one difference of two tessellations of the same wall.
- Stack on #2767 (the #2762 fix). Dropped: the pin is red on `upstream/main`
  and green with this change alone. The two PRs are independent.

**Cost to change later.** One call in `minus_standing` and one 20-line
module. A different threshold is one constant.
