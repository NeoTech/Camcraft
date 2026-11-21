## #3127 - A work-turning rotary line that does not pass through the WCS origin

- **What was decided** (issue worker, unattended; the maintainer can change it):
  1. **Refuse, do not read.** A work-turning rotary that states a non-zero (or non-finite) `pivot` or `offset` is refused by name. The posed first-traverse law (`operation::first_traverse_posed::turn_of`) refuses generation. The index-swing clause (`posting_index_swing::verdict`) refuses the post.
  2. **One test for both.** `posting::index_swing::off_origin` reads the two fields. A component with an absolute value of 1e-9 mm or less reads as zero. A zero vector (`[0, 0, 0]`) is judged as before.
  3. **Which lines.** A vertical line keeps every height, wherever it stands. So one vertical work-turning rotary with a pivot is not refused. With two or more work-turning rotaries, the sphere about the WCS origin needs every line through that origin, so any off-origin rotary refuses.
  4. **The sibling clause.** `posting_swing_standing` reads the same `Verdict`. It reads only `Verdict::Judged`, so it skips the new `OffOrigin` verdict as it skips `Unlined` and `Leaning`. The index-swing clause has already refused the post.
- **Why.** `pivot` is stated in machine coordinates. The program does not know where the WCS origin is in machine coordinates. So the program cannot place the stated line in the WCS. A read about a guessed line can be too low, and the post is then not blocked. A refusal is visible and safe. No shipped machine file states `pivot` or `offset`.
- **Pinning tests.**
  - `crates/app/tests/first_traverse_posed.rs`: `a_table_line_off_the_wcs_origin_refuses_by_name`, `a_zero_pivot_reads_the_table_line_through_the_wcs_origin`.
  - `crates/app/tests/index_swing_offset_line.rs`: `a_pivot_off_the_wcs_origin_refuses_the_index_swing_by_name`, `an_offset_off_the_wcs_origin_refuses_the_index_swing_by_name`, `a_zero_pivot_and_offset_judge_the_index_swing_as_before`.
- **What was considered and dropped**:
  - Read the swept set about the stated line: it needs the machine position of the WCS origin, which the project does not state. A guess makes a silent wrong answer.
  - Treat `pivot` as a WCS position: the field is documented as machine coordinates (#1962). This reading is wrong for a typical machine.
  - Refuse every rotary with a stated `pivot`, also a zero one: a file that states the default must not lose its checks.
- **Cost to change later**: low. When the project can place the machine frame in the WCS, replace the refusal with a read about the moved line in `off_origin`'s two callers.
