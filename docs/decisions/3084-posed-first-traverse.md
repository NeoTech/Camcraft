## #3084 - The first traverse over work that the machine's rotaries can leave at any pose

- **What was decided** (issue worker, unattended; the maintainer can change it):
  1. **The premise (maintainer, PR #2997).** The unknown start is at the top of the Z travel (the tool-change height), as for #2901, #2965 and #2987. A program that starts with the tool low near the table or the work (for example after a manual jog) is NOT covered.
  2. **The post order is the basis.** The post sends Z, then X/Y at that height, then the move with the rotary words (`fanuc.cps` `traverseZ`, `xyFirst`, `onRapid`). So during the X/Y traverse each rotary that turns the work stands where the previous program left it. The operation does not state that pose.
  3. **Where the lines are.** Each line of a work-turning rotary passes through the WCS origin. This is the indexed post's setup law that the index-swing clause already reads (`posting_index_swing.rs`). No shipped machine file states `pivot` or `offset` (#1962), and no code places nested rotaries from them.
  4. **What the law reads** (new law `sim::verify::first_traverse_low_posed`, seam `operation::first_traverse_posed`):
     - Two or more work-turning rotaries: each turn moves the other's line. The only figure that every pose keeps is the distance from the WCS origin. Each point of the start stock and of the part, and each corner of each tagged fixture, is read on its sphere about the WCS origin (top = origin height + distance).
     - One work-turning rotary that tilts: a full turn about its line through the WCS origin (`posed_top`, the same circle as `swept_top`).
     - A full turn and a full sphere contain every stated turning range. So the read is never lower than the true top. The turning range is not read.
     - Heights are read along the path's frame axis (`+Z` upright; the 3+2 frame's axis and origin in a tilted frame).
  5. **Who answers.** A milling operation (`mill_order`) of a multi-axis or rotary kind (`Section::MultiAxis` or `Mode::Rotary`), on a mounted machine with a tilting rotary that states `turns_part: true`:
     - a rotary kind on a machine where two or more rotaries turn the work (case 1 of the issue);
     - a multi-axis kind on a machine where one or more rotaries turn the work (case 2 of the issue).
     - A rotary kind with one work-turning rotary keeps #2965's swept law about the fitted axis.
  6. **The traverse height.** The higher of the clearance and move 0's own height in the path's frame (`traverseZ`). A rotary kind's move 0 is a radial pose, so it reads the clearance alone: the post never traverses under it. With no clearance, move 0's height.
  7. **Steel.** Each tagged fixture is read as turning with the work (a superset: a box's swept top is never under its plain top). The machine table stays under the rotaries and is not read.
  8. **The verdict.** The same blocking `Warning::FirstTraverseLow`. It replaces each first-traverse verdict at a lower top, so one verdict names the highest top that the clearance must clear.
  9. **Refusal by name.** A lone tilting work-turning rotary with no usable line (no stated axis and no RS-274 letter A or B, or a zero or non-finite axis) refuses generation: "the work-turning rotary Q states no usable axis line, so the first traverse over the work it turns cannot be judged". The sphere case needs no line.
- **Cache.** The op stamp of an operation that the posed law reads (`first_traverse_posed::newly_judged`) takes one more word, `first-traverse-posed-3084`. Such a cache reads Stale once. The salt does not change. No shipped project moves: the only examples on a machine with a work-turning rotary (`first-4axis-part`, `first-continuous-4axis-part`, the microARC 4, one rotary) have slot and rotary-parallel operations, which the law does not read.
- **Pinning tests.** `crates/app/tests/first_traverse_posed.rs`: `two_work_turning_rotaries_read_the_rotary_kinds_stock_on_its_sphere`, `a_tilting_table_reads_a_5_axis_kinds_stock_swept_about_its_line`, `a_work_turning_rotary_with_no_line_refuses_by_name`. Sim unit tests in `crates/sim/src/verify/first_traverse_posed.rs`.
- **Known limits.**
  - A program that starts low (premise, item 1).
  - The setup law of item 3: a work offset that is not on the rotary lines is read as if it were. The index-swing clause has the same premise.
  - The Z word is read as a machine height in the frame of the rotaries at zero, as #2965 reads it. Under a TCP mode that turns the program frame with the table, the post's Z is a height in the turned frame; that reading is not made.
  - A rotary that never stated `turns_part` (a project saved before #804, or a machine built in code) is not read as work-turning. The load names such a machine (#813), and each parsed file states it.
  - A 3+2 operation (`Tilt`) and a cross drill on a work-tilting machine are not read by this law. Their first traverse has the same unknown pose: follow-up #3100.
  - The tool radius is not added (the law reads the tip, as #2901 and #2965 do).
- **What was considered and dropped**:
  - The exact top over the stated turning ranges (branch and bound over the inner angle, with the closed form over the outer one): it needs the order of the rotaries and the position of each line, which no file states. The sphere is exact for a trunnion whose ranges reach 90 degrees, which is the common case.
  - The lines from `pivot` / `offset` / `chain_order`: no shipped file states them.
  - A refusal of every multi-axis operation on a table-tilt machine: it blocks programs that the law shows to be safe.
  - A new warning variant: the #2901 verdict blocks the post and carries both heights; a variant adds four mappings and no information.
  - Edit `first_traverse::judge`: the law stays in its own functions, so the stacked PRs (#3099, #3096) do not conflict.
- **Cost to change later**: low. One law function in `camcraft-sim` and the selection in `operation::first_traverse_posed` (`turn_of`).
