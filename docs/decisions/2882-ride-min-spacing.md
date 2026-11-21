## #2882 - Thin turning profile samples that are closer than half the chord share

**What was decided:**
- Add a private `thin` step in `crates/motion/src/turning/offset.rs`.
- `thin` drops each sample that lies closer than `MIN_GAP_SHARE * share` to the last kept sample. `MIN_GAP_SHARE` is `0.5`.
- `thin` always keeps the first sample and the last sample. The window ends stay.
- `profile_offset` and `profile_offset_in` both run the outside ride and the inside ride through `thin`.

**Why:**
- Near a vertical tangent of the offset curve, the chord bisection places samples less than 1 micrometre apart. The lathe posts then write runs of identical G01 blocks. The Fanuc example wrote 408 identical blocks in one cone-thread program.
- `out.dedup()` removes only exact duplicates, so it does not fix this case.
- Thinning at the source fixes every turning ride. This covers profile rough, profile finish, contour and boring. It also covers every lathe post.
- Each kept sample is still an exact point of the offset curve. No kept sample gouges.
- Each dropped sample lies within the gap of a kept sample. At the window end, a kept sample can give way to the last sample, so a dropped sample lies within two gaps of a kept one. A chord therefore moves by at most two gaps. The chord bound becomes `2 * share`.
- The callers pass a quarter of the user tolerance as the share. The ride therefore stays within half of the user tolerance.
- At a tolerance of 0.01 mm, the share is 0.0025 mm. The least gap is then 0.00125 mm. That gap is more than the posted resolution of 0.001 mm.

**What was considered and dropped:**
- Drop repeated feed blocks in each lathe post (`assets/posts/fanuc-lathe.cps` and `assets/posts/tormach-lathe.cps`), when a block has the same words as the last block.
  - Dropped because it hides the cause in each post.
  - Dropped because it changes posted output in two dialects.
  - Dropped because the project file still keeps the dense samples.
- Use a fixed absolute gap, for example 1 micrometre.
  - Dropped because a fixed gap does not scale with the tolerance that the user sets.

**Cost to change later:**
- Low. The change is one private function and one constant in `offset.rs`. The pinning test is `crates/motion/tests/turning_offset_spacing.rs`.
- Changing `MIN_GAP_SHARE` changes the chord bound. Keep the bound at `2 * share`, or update the tolerance claim in this entry.
- Known limit: at a very small user tolerance, for example 0.001 mm, the gap is below the posted resolution. Two rounded blocks can then still be equal. This is rare and it costs no position.

**Review fix (PR #2897).** The catalog case `turning/cone-thread` reached main after this branch was cut. Its committed `.nc` had the repeated blocks. The branch merges main and rebuilds the case with `cargo xtask catalog --case turning/cone-thread`. The rebuild removes 465 `G01` lines and adds none. The runs of same-X `G01` blocks go from 509 to 82. The case README drops the limit line for #2882. The taper and thread blocks that the README names stay the same.
