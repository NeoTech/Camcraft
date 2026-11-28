## #4149 - Mach3 post: extended work offsets with G59 P

**Decision:** Increase `workOffsetMax` to 254 in `assets/posts/mach3.cps` per Using Mach3Mill section 10.7.18.
Format work offset numbers 1 to 6 as standard G-code words `G54` to `G59`.
Format work offset numbers 7 to 254 as extended G-code words `G59 P7` to `G59 P254`.
Refuse work offset numbers less than 1 or greater than 254 with an explicit error.
Update operation comment notes to display `G59 P<n>` for extended offsets.

**Why:** Using Mach3Mill section 10.7.18 specifies work offsets 1 to 254. Offsets 1 to 6 map to `G54` through `G59`.
Offsets 7 to 254 use the `G59 P<n>` syntax up to `P254`.
The post previously stopped at `workOffsetMax = 6`. This prevented users from using extended fixtures.

**Considered and dropped:**
- Do not accept offset 255. Using Mach3Mill section 10.7.18 defines offsets 1 through 254.
- Do not emit `G54.1 P<n>`. Fanuc controls use `G54.1 P<n>`, but standard Mach3 expects `G59 P<n>`.
- Do not wrap around or clamp offset numbers greater than 254. Refusing invalid values prevents machining collisions.
- Do not change engine behavior. `camcraft_post` generic bounds allow control-specific limits in post scripts.

**Cost to change later:** Minimal. `mach3.cps` isolates the formatting logic in helper functions.
Updating the syntax or range requires only local changes in `mach3.cps` and its unit tests.
